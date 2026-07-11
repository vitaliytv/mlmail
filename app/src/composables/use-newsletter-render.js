import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { createOpenAiChat } from '@7n/tauri-components'
import { useOmlx } from '@7n/tauri-components/vue'
import { SYSTEM_PROMPT } from '../services/newsletter-template.js'

/**
 * Renders a newsletter message using the given template prompt.
 * Returns a parsed array of articles [{title, url, description}] or null on failure.
 * @returns {{ render: (message: object, prompt: string) => Promise<import('../services/newsletter-template.js').NewsletterArticle[] | null> }}
 */
export function useNewsletterRender() {
  const { baseUrl, model, apiKey, loadEnv } = useOmlx({ storagePrefix: 'mlmail' })

  /**
   *
   * @param message
   * @param prompt
   */
  async function render(message, prompt) {
    const body = (message?.body ?? '').trim()
    if (!body) return []
    try {
      await loadEnv()
      const chat = createOpenAiChat({
        baseUrl: baseUrl.value,
        model: model.value,
        apiKey: apiKey.value || undefined,
        fetchFn: tauriFetch
      })
      const userContent = `${prompt}\n\n---\n${body}`
      const reply = await chat({
        messages: [
          { role: 'system', content: SYSTEM_PROMPT },
          { role: 'user', content: userContent }
        ],
        tools: []
      })
      const text = (reply?.content ?? '').trim()
      // Strip markdown code fences if the model wrapped the JSON
      const json = text
        .replace(/^```(?:json)?\n?/, '')
        .replace(/\n?```$/, '')
        .trim()
      return JSON.parse(json)
    } catch {
      return null
    }
  }

  return { render }
}
