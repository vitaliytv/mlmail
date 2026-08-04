import { useLlm } from '../llm.js'
import { SYSTEM_PROMPT } from '../services/newsletter-template.js'

const CODE_FENCE_OPEN_RE = /^```(?:json)?\n?/
const CODE_FENCE_CLOSE_RE = /\n?```$/

/**
 * Renders a newsletter message using the given template prompt.
 * Returns a parsed array of articles [{title, url, description}] or null on failure.
 * @returns {{ render: (message: object, prompt: string) => Promise<import('../services/newsletter-template.js').NewsletterArticle[] | null> }} the render composable
 */
export function useNewsletterRender() {
  const { loadEnv, chat } = useLlm({ storagePrefix: 'mlmail' })

  /**
   * @param {object} message the source message to render a newsletter from
   * @param {string} prompt the template prompt guiding the render
   * @returns {Promise<import('../services/newsletter-template.js').NewsletterArticle[] | null>} parsed articles, or null on failure
   */
  async function render(message, prompt) {
    const body = (message?.body ?? '').trim()
    if (!body) return []
    try {
      await loadEnv()
      const userContent = `${prompt}\n\n---\n${body}`
      const reply = await chat({ system: SYSTEM_PROMPT, user: userContent })
      const text = (reply?.content ?? '').trim()
      // Strip markdown code fences if the model wrapped the JSON
      const json = text.replace(CODE_FENCE_OPEN_RE, '').replace(CODE_FENCE_CLOSE_RE, '').trim()
      return JSON.parse(json)
    } catch {
      return null
    }
  }

  return { render }
}
