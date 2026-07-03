import { describe, expect, it } from 'vitest'
import { buildSummaryPrompt } from './summary.js'

describe('buildSummaryPrompt', () => {
  it('puts sender and subject above the body', () => {
    expect(buildSummaryPrompt({ from: 'npm <s@npm.com>', subject: 'Published', body: 'A new version…' }))
      .toBe('Від: npm <s@npm.com>\nТема: Published\n\nA new version…')
  })

  it('omits blank header lines', () => {
    expect(buildSummaryPrompt({ from: '', subject: 'Hi', body: 'text' })).toBe('Тема: Hi\n\ntext')
  })

  it('returns just the body when there are no headers', () => {
    expect(buildSummaryPrompt({ body: 'only body' })).toBe('only body')
  })

  it('handles a missing/empty message gracefully', () => {
    expect(buildSummaryPrompt()).toBe('')
    expect(buildSummaryPrompt({})).toBe('')
  })
})
