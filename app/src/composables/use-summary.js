import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { createOpenAiChat } from '@7n/tauri-components'
import { useOmlx } from '@7n/tauri-components/vue'
import {
  buildSummaryPrompt,
  translateHtmlEmail,
  SUMMARY_SYSTEM,
  TRANSLATE_BATCH_SYSTEM,
} from '../services/summary.js'

// Local-LLM helper for the two-column reader: summarize the current email in
// Ukrainian via the on-device omlx model. One-shot, no tools. Best-effort —
// returns null on any failure (omlx down, network) so the UI can show a notice
// instead of crashing, and '' for an empty body.

// omlx can stall indefinitely on a large batch instead of erroring (observed:
// an 80-string batch received no data for 120s+), so every call gets its own
// abort timeout — otherwise a stuck request leaves the UI spinning forever.
const LLM_TIMEOUT_MS = 60_000

/**
 * @param {typeof fetch} fetchFn fetch implementation to wrap
 * @param {number} timeoutMs abort the request after this many milliseconds
 * @returns {typeof fetch} fetch wrapped with a default abort timeout
 */
function withTimeout(fetchFn, timeoutMs) {
  return (url, init) => fetchFn(url, { ...init, signal: init?.signal ?? AbortSignal.timeout(timeoutMs) })
}

/**
 * @returns {{ summarize: (message: object) => Promise<string|null> }} summary helper
 */
export function useSummary() {
  const { baseUrl, model, apiKey, loadEnv } = useOmlx({ storagePrefix: 'mlmail' })
  const timedFetch = withTimeout(tauriFetch, LLM_TIMEOUT_MS)

  /**
   * Summarize a message in Ukrainian.
   * @param {{ from?: string, subject?: string, body?: string }} message the email
   * @returns {Promise<string|null>} the summary, '' when the body is empty, or null on failure
   */
  async function summarize(message) {
    if (!(message?.body ?? '').trim()) return ''
    try {
      await loadEnv()
      const chat = createOpenAiChat({
        baseUrl: baseUrl.value,
        model: model.value,
        apiKey: apiKey.value || undefined,
        fetchFn: timedFetch,
      })
      const reply = await chat({
        messages: [
          { role: 'system', content: SUMMARY_SYSTEM },
          { role: 'user', content: buildSummaryPrompt(message) },
        ],
        tools: [],
      })
      return (reply?.content ?? '').trim() || null
    }
    catch {
      return null
    }
  }

  /**
   * Translate an email's HTML in-place: extract text nodes → batch-translate → reinsert.
   * Falls back to translating the plain body when html_body is absent.
   * @param {{ html_body?: string, body?: string }} message
   * @returns {Promise<{ html: string } | null>}
   */
  async function translateHtml(message) {
    const html = message?.html_body
    if (!html && !(message?.body ?? '').trim()) return { html: '' }
    try {
      await loadEnv()
      const chat = createOpenAiChat({
        baseUrl: baseUrl.value,
        model: model.value,
        apiKey: apiKey.value || undefined,
        fetchFn: timedFetch,
      })

      /** @param {string[]} texts */
      async function translateBatch(texts) {
        // omlx quality/latency degrades sharply past ~35 items per batch (a
        // 35-item batch already took 41s with a mangled translation; an
        // 80-item batch hung indefinitely) — keep chunks well under that.
        const CHUNK = 15
        const result = []
        for (let i = 0; i < texts.length; i += CHUNK) {
          const chunk = texts.slice(i, i + CHUNK)
          const reply = await chat({
            messages: [
              { role: 'system', content: TRANSLATE_BATCH_SYSTEM },
              { role: 'user', content: JSON.stringify(chunk) },
            ],
            tools: [],
          })
          const raw = (reply?.content ?? '').trim()
          // Strip potential markdown code fences
          const jsonStr = raw.replace(/^```[^\n]*\n?/, '').replace(/\n?```$/, '').trim()
          let parsed
          try { parsed = JSON.parse(jsonStr) } catch { parsed = chunk }
          result.push(...(Array.isArray(parsed) ? parsed : chunk))
        }
        return result
      }

      if (html) {
        const translatedHtml = await translateHtmlEmail(html, translateBatch)
        return { html: translatedHtml }
      }
      // Plain-text fallback: translate as single-item batch
      const [translated] = await translateBatch([message.body])
      return { html: `<pre style="white-space:pre-wrap;font-family:inherit">${translated}</pre>` }
    }
    catch {
      return null
    }
  }

  return { summarize, translateHtml }
}
