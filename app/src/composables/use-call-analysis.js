import { invoke } from '@tauri-apps/api/core'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { createOpenAiChat, useOmlx } from '../omlx.js'

/**
 * Render a journaled agent call (request + actions + response) as a
 * self-contained prompt asking the model to suggest project-code changes
 * that would make this exact request run more efficiently.
 * @param {object} rec journal record ({ intent, actions?, summary?, question?, error? })
 * @returns {string} prompt text
 */
export function buildAnalysisPrompt(rec) {
  const actions = (rec.actions ?? [])
    .map(a => {
      const status = a.envelope?.ok ? 'ok' : `error: ${a.envelope?.error?.message ?? a.envelope?.error ?? '?'}`
      return `- ${a.tool}(${JSON.stringify(a.input)}) -> ${status}`
    })
    .join('\n')
  const responseText = rec.summary ?? rec.question ?? rec.error ?? '(немає тексту відповіді)'

  return [
    'Ось лог одного виклику LLM-агента застосунку mlmail (поточна робоча директорія — корінь проєкту).',
    '',
    `Запит: ${rec.intent}`,
    '',
    'Дії агента:',
    actions || '(немає дій)',
    '',
    `Відповідь: ${responseText}`,
    '',
    'Проаналізуй цей запит і відповідь та запропонуй, як адаптувати код проєкту, щоб такий запит виконувався ефективніше (менше зайвих дій/токенів/помилок). За можливості вказуй конкретні файли й зміни.'
  ].join('\n')
}

/**
 * Run the analysis via the `pi` CLI on a cloud model (spawned by the Rust
 * backend, debug builds only). Doesn't depend on the in-app agent gateway,
 * so it lives at module scope rather than inside `useCallAnalysis`.
 * @param {object} rec journal record to analyze
 * @returns {Promise<string>} pi's text answer
 */
export async function analyzeWithPi(rec) {
  return await invoke('analyze_call_with_pi', { prompt: buildAnalysisPrompt(rec) })
}

/**
 * Local omlx + pi runners for journal-call analysis.
 * omlx no longer rides on the agent gateway (useAcpAgent has no baseUrl/model —
 * see CHANGELOG @7n/tauri-components@0.11.0); it uses the app-local useOmlx.
 * @returns {{analyzeWithPi: (rec: object) => Promise<string>, analyzeWithOmlx: (rec: object) => Promise<string>}} analysis runners
 */
export function useCallAnalysis() {
  const { baseUrl, model, apiKey, loadEnv } = useOmlx({ storagePrefix: 'mlmail' })

  /**
   * @param {object} rec journal record to analyze
   * @returns {Promise<string>} the local model's text answer
   */
  async function analyzeWithOmlx(rec) {
    await loadEnv()
    const chat = createOpenAiChat({
      baseUrl: baseUrl.value,
      model: model.value,
      apiKey: apiKey.value || undefined,
      fetchFn: tauriFetch
    })
    const reply = await chat({ messages: [{ role: 'user', content: buildAnalysisPrompt(rec) }], tools: [] })
    return reply.content ?? ''
  }

  return { analyzeWithPi, analyzeWithOmlx }
}
