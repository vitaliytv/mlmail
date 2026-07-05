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
    .map(a => `- ${a.tool}(${JSON.stringify(a.input)}) -> ${a.envelope?.ok ? 'ok' : `error: ${a.envelope?.error?.message ?? a.envelope?.error ?? '?'}`}`)
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
    'Проаналізуй цей запит і відповідь та запропонуй, як адаптувати код проєкту, щоб такий запит виконувався ефективніше (менше зайвих дій/токенів/помилок). За можливості вказуй конкретні файли й зміни.',
  ].join('\n')
}

/**
 * Two ways to run the above analysis for a journal record: the local omlx
 * model (direct HTTP, reusing the agent's own connection) or the `pi` CLI
 * on a cloud model (spawned by the Rust backend, debug builds only).
 * @param {object} agent the in-app agent gateway (from useAgent())
 * @returns {{analyzeWithPi: (rec: object) => Promise<string>, analyzeWithOmlx: (rec: object) => Promise<string>}} analysis runners
 */
export function useCallAnalysis(agent) {
  /**
   * @param {object} rec journal record to analyze
   * @returns {Promise<string>} pi's text answer
   */
  async function analyzeWithPi(rec) {
    return invoke('analyze_call_with_pi', { prompt: buildAnalysisPrompt(rec) })
  }

  /**
   * @param {object} rec journal record to analyze
   * @returns {Promise<string>} the local model's text answer
   */
  async function analyzeWithOmlx(rec) {
    const chat = createOpenAiChat({
      baseUrl: agent.baseUrl.value,
      model: agent.model.value,
      apiKey: agent.apiKey.value || undefined,
      fetchFn: tauriFetch,
    })
    const reply = await chat({ messages: [{ role: 'user', content: buildAnalysisPrompt(rec) }], tools: [] })
    return reply.content ?? ''
  }

  return { analyzeWithPi, analyzeWithOmlx }
}
