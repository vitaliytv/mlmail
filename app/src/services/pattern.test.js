import { describe, expect, it } from 'vitest'
import { buildPatternQuery, parseFromEmail, sanitizeSubjectSuggestion } from './pattern.js'

describe('parseFromEmail', () => {
  it('extracts the address from a "Name <email>" header', () => {
    expect(parseFromEmail('npm <support@npmjs.com>')).toBe('support@npmjs.com')
  })

  it('returns a bare address unchanged', () => {
    expect(parseFromEmail('support@npmjs.com')).toBe('support@npmjs.com')
  })

  it('trims surrounding whitespace', () => {
    expect(parseFromEmail('  a@b.com  ')).toBe('a@b.com')
  })

  it('returns empty string for null/undefined/empty', () => {
    expect(parseFromEmail(null)).toBe('')
    expect(parseFromEmail()).toBe('')
    expect(parseFromEmail('')).toBe('')
  })
})

describe('buildPatternQuery', () => {
  it('combines from and subject into a Gmail query', () => {
    expect(buildPatternQuery({ from: 'support@npmjs.com', subject: 'Successfully published' }))
      .toBe('from:support@npmjs.com subject:"Successfully published"')
  })

  it('emits only from when subject is blank', () => {
    expect(buildPatternQuery({ from: 'a@b.com', subject: '  ' })).toBe('from:a@b.com')
  })

  it('emits only subject when from is blank', () => {
    expect(buildPatternQuery({ subject: 'Invoice' })).toBe('subject:"Invoice"')
  })

  it('strips embedded quotes from the subject phrase', () => {
    expect(buildPatternQuery({ subject: 'say "hi"' })).toBe('subject:"say hi"')
  })

  it('returns empty string when both parts are blank', () => {
    expect(buildPatternQuery({ from: '', subject: '' })).toBe('')
    expect(buildPatternQuery()).toBe('')
  })
})

describe('sanitizeSubjectSuggestion', () => {
  it('returns a clean single-line phrase', () => {
    expect(sanitizeSubjectSuggestion('Successfully published', 'fb')).toBe('Successfully published')
  })

  it('takes only the first line', () => {
    expect(sanitizeSubjectSuggestion('Successfully published\nblah blah', 'fb')).toBe('Successfully published')
  })

  it('strips wrapping quotes and backticks', () => {
    expect(sanitizeSubjectSuggestion('"Successfully published"', 'fb')).toBe('Successfully published')
    expect(sanitizeSubjectSuggestion('`done`', 'fb')).toBe('done')
  })

  it('falls back when the suggestion is empty or not a string', () => {
    expect(sanitizeSubjectSuggestion('', 'fallback')).toBe('fallback')
    expect(sanitizeSubjectSuggestion('   ', 'fallback')).toBe('fallback')
    expect(sanitizeSubjectSuggestion(null, 'fallback')).toBe('fallback')
    expect(sanitizeSubjectSuggestion(42, 'fallback')).toBe('fallback')
  })
})
