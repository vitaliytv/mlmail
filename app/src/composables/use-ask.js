import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { createOpenAiChat } from '@7n/tauri-components'
import { useOmlx } from '@7n/tauri-components/vue'
import { buildSummaryPrompt } from '../services/summary.js'

const ASK_SYSTEM = [
  'Ти помічник, що відповідає на запитання щодо конкретного email.',
  'Тобі буде надано вміст листа, а потім запитання від користувача.',
  'Відповідай стисло, по суті, українською мовою.'
].join(' ')

/**
 * @returns {{ ask: (message: object, question: string) => Promise<string|null>, isAsking: import('vue').Ref<boolean> }}
 */
export function useAsk() {
  const { baseUrl, model, apiKey, loadEnv } = useOmlx({ storagePrefix: 'mlmail' })
  const isAsking = ref(false)

  /**
   *
   * @param message
   * @param question
   */
  async function ask(message, question) {
    if (!question.trim()) return null
    isAsking.value = true
    try {
      await loadEnv()
      const chat = createOpenAiChat({
        baseUrl: baseUrl.value,
        model: model.value,
        apiKey: apiKey.value || undefined,
        fetchFn: tauriFetch
      })
      const emailContext = buildSummaryPrompt(message)
      const reply = await chat({
        messages: [
          { role: 'system', content: ASK_SYSTEM },
          { role: 'user', content: `Лист:\n${emailContext}\n\nЗапитання: ${question}` }
        ],
        tools: []
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
