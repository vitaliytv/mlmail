import { describe, expect, it } from 'vitest'
import { buildAnalysisPrompt } from './use-call-analysis.js'

describe('buildAnalysisPrompt', () => {
  it('includes the intent, actions and response text', () => {
    const rec = {
      intent: 'Знайти листи від npm',
      actions: [{ tool: 'gmail_search', input: { q: 'from:npm' }, envelope: { ok: true } }],
      summary: 'Знайдено 3 листи.'
    }
    const prompt = buildAnalysisPrompt(rec)
    expect(prompt).toContain('Знайти листи від npm')
    expect(prompt).toContain('gmail_search({"q":"from:npm"}) -> ok')
    expect(prompt).toContain('Знайдено 3 листи.')
  })

  it('reports failed actions with their error message', () => {
    const rec = {
      intent: 'Видалити лист',
      actions: [{ tool: 'gmail_trash', input: {}, envelope: { ok: false, error: { message: 'not found' } } }]
    }
    expect(buildAnalysisPrompt(rec)).toContain('gmail_trash({}) -> error: not found')
  })

  it('falls back to question/error/placeholder when there is no summary', () => {
    expect(buildAnalysisPrompt({ intent: 'x', question: 'Уточніть будь ласка?' })).toContain('Уточніть будь ласка?')
    expect(buildAnalysisPrompt({ intent: 'x', error: 'boom' })).toContain('boom')
    expect(buildAnalysisPrompt({ intent: 'x' })).toContain('немає тексту відповіді')
  })

  it('notes when there are no actions', () => {
    expect(buildAnalysisPrompt({ intent: 'x' })).toContain('немає дій')
  })
})
