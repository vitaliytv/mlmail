/** Підбирає стабільний шаблон теми листа через налаштований локальний LLM із fail-safe fallback на оригінал. */
import { useLlm } from '../llm.js'
import { sanitizeSubjectSuggestion } from '../services/pattern.js'

const SYSTEM = [
  'You extract a reusable matching pattern from an automated email subject line.',
  'Return ONLY the stable, sender-generated leading phrase that repeats across',
  'similar notifications — drop the variable tail (version numbers, package or',
  'user names, ids, dates, amounts).',
  'Example: "Successfully published @nitra/cursor@12.13.0" → "Successfully published".',
  'Reply with just that phrase: no quotes, no labels, no explanation.'
].join(' ')

/**
 * @returns {{ suggestSubjectPattern: (subject: string) => Promise<string> }} pattern helpers
 */
export function usePattern() {
  const { loadEnv, chat } = useLlm({ storagePrefix: 'mlmail' })

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
      const reply = await chat({ system: SYSTEM, user: fallback })
      return sanitizeSubjectSuggestion(reply?.content, fallback)
    } catch {
      return fallback
    }
  }

  return { suggestSubjectPattern }
}
