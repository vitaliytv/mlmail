import { invoke } from '@tauri-apps/api/core'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { createOpenAiChat } from '@7n/tauri-components'

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
 * The local omlx model (direct HTTP, reusing the agent's own connection),
 * as the other way to run the above analysis for a journal record.
 * @param {object} agent the in-app agent gateway (from useAgent())
 * @returns {{analyzeWithPi: (rec: object) => Promise<string>, analyzeWithOmlx: (rec: object) => Promise<string>}} analysis runners
 */
export function useCallAnalysis(agent) {
  /**
   * @param {object} rec journal record to analyze
   * @returns {Promise<string>} the local model's text answer
   */
  async function analyzeWithOmlx(rec) {
    const chat = createOpenAiChat({
      baseUrl: agent.baseUrl.value,
      model: agent.model.value,
      apiKey: agent.apiKey.value || undefined,
      fetchFn: tauriFetch
    })
    const reply = await chat({ messages: [{ role: 'user', content: buildAnalysisPrompt(rec) }], tools: [] })
    return reply.content ?? ''
  }

  return { analyzeWithPi, analyzeWithOmlx }
}
