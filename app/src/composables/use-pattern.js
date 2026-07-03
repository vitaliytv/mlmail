import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { createOpenAiChat } from '@7n/tauri-components'
import { useOmlx } from '@7n/tauri-components/vue'
import { sanitizeSubjectSuggestion } from '../services/pattern.js'

// Local-LLM helper for the "rule from this email" panel: given a concrete
// subject line, ask the on-device omlx model for the stable, sender-generated
// prefix to match similar automated mail on (dropping the variable tail —
// versions, names, ids, dates). One-shot, no tools. Best-effort: any failure
// (omlx down, network, odd output) falls back to the original subject, so the
// panel always has a usable value.

const SYSTEM = [
  'You extract a reusable matching pattern from an automated email subject line.',
  'Return ONLY the stable, sender-generated leading phrase that repeats across',
  'similar notifications — drop the variable tail (version numbers, package or',
  'user names, ids, dates, amounts).',
  'Example: "Successfully published @nitra/cursor@12.13.0" → "Successfully published".',
  'Reply with just that phrase: no quotes, no labels, no explanation.',
].join(' ')

/**
 * @returns {{ suggestSubjectPattern: (subject: string) => Promise<string> }} pattern helpers
 */
export function usePattern() {
  const { baseUrl, model, apiKey, loadEnv } = useOmlx({ storagePrefix: 'mlmail' })

  /**
   * Suggest a stable subject pattern for `subject` via the local LLM.
   * @param {string} subject the concrete subject line of the viewed message
   * @returns {Promise<string>} the suggested phrase, or the trimmed subject on any failure
   */
  async function suggestSubjectPattern(subject) {
    const fallback = (subject ?? '').trim()
    if (!fallback) return ''
    try {
      await loadEnv()
      const chat = createOpenAiChat({
        baseUrl: baseUrl.value,
        model: model.value,
        apiKey: apiKey.value || undefined,
        fetchFn: tauriFetch,
      })
      const reply = await chat({
        messages: [
          { role: 'system', content: SYSTEM },
          { role: 'user', content: fallback },
        ],
        tools: [],
      })
      return sanitizeSubjectSuggestion(reply?.content, fallback)
    }
    catch {
      return fallback
    }
  }

  return { suggestSubjectPattern }
}
