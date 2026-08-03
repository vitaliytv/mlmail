import { useLlm } from '../llm.js'
import { buildSummaryPrompt } from '../services/summary.js'

const ASK_SYSTEM = [
  'Ти помічник, що відповідає на запитання щодо конкретного email.',
  'Тобі буде надано вміст листа, а потім запитання від користувача.',
  'Відповідай стисло, по суті, українською мовою.'
].join(' ')

/**
 * @returns {{ ask: (message: object, question: string) => Promise<string|null>, isAsking: import('vue').Ref<boolean> }} the ask composable
 */
export function useAsk() {
  const { loadEnv, chat } = useLlm({ storagePrefix: 'mlmail' })
  const isAsking = ref(false)

  /**
   * @param {object} message the email being asked about
   * @param {string} question the user's question about the email
   * @returns {Promise<string|null>} the model's answer, or null on failure/empty question
   */
  async function ask(message, question) {
    if (!question.trim()) return null
    isAsking.value = true
    try {
      await loadEnv()
      const emailContext = buildSummaryPrompt(message)
      const reply = await chat({
        system: ASK_SYSTEM,
        user: `Лист:\n${emailContext}\n\nЗапитання: ${question}`
      })
      return (reply?.content ?? '').trim() || null
    } catch {
      return null
    } finally {
      isAsking.value = false
    }
  }

  return { ask, isAsking }
}
