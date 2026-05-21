import { describe, it, expect } from 'bun:test'
import { hasMark, stripMark, formatMark, applyMark } from '../../docs-regen/marks.js'

const BODY_PLAIN = `# Foo

Some content.

## Rationale

More content.
`

const BODY_WITH_RULE_IN_BODY = `# Foo

Some content.

---

More content after a horizontal rule that is NOT a mark.
`

const BODY_WITH_MARK = `# Foo

Some content.

---

**Опрацьовано** 2026-05-17. Проекції:
- [01-context](../ci4/01-context.md)
- [03-components](../ci4/03-components.md)
`

const BODY_WITH_EMPTY_MARK = `# Foo

Some content.

---

**Опрацьовано** 2026-05-17. Проекції: жодної.
`

describe('hasMark', () => {
  it('returns false for body without any sentinel block', () => {
    expect(hasMark(BODY_PLAIN)).toBe(false)
  })

  it('returns false for body with horizontal rule but no Опрацьовано-paragraph after', () => {
    expect(hasMark(BODY_WITH_RULE_IN_BODY)).toBe(false)
  })

  it('returns true for body with mark of projections', () => {
    expect(hasMark(BODY_WITH_MARK)).toBe(true)
  })

  it('returns true for body with "жодної" mark', () => {
    expect(hasMark(BODY_WITH_EMPTY_MARK)).toBe(true)
  })
})

describe('stripMark', () => {
  it('returns body unchanged when no mark present', () => {
    expect(stripMark(BODY_PLAIN)).toBe(BODY_PLAIN)
  })

  it('preserves horizontal rule inside body, removes only trailing mark', () => {
    const input = BODY_WITH_RULE_IN_BODY + '\n---\n\n**Опрацьовано** 2026-05-17. Проекції: жодної.\n'
    const result = stripMark(input)
    expect(result).toBe(BODY_WITH_RULE_IN_BODY)
  })

  it('strips multi-line mark', () => {
    expect(stripMark(BODY_WITH_MARK)).toBe(`# Foo

Some content.
`)
  })
})

describe('formatMark', () => {
  it('formats empty projections as "жодної"', () => {
    expect(formatMark('2026-05-18', [])).toBe('**Опрацьовано** 2026-05-18. Проекції: жодної.')
  })

  it('formats projections as bullet list with relative links', () => {
    const result = formatMark('2026-05-18', ['01-context', '03-components'])
    expect(result).toBe(
      '**Опрацьовано** 2026-05-18. Проекції:\n' +
        '- [01-context](../ci4/01-context.md)\n' +
        '- [03-components](../ci4/03-components.md)'
    )
  })
})

describe('applyMark', () => {
  it('appends mark when none present', () => {
    const result = applyMark(BODY_PLAIN, '2026-05-18', ['01-context'])
    expect(
      result.endsWith('\n---\n\n**Опрацьовано** 2026-05-18. Проекції:\n- [01-context](../ci4/01-context.md)\n')
    ).toBe(true)
    expect(result.startsWith(BODY_PLAIN.trimEnd())).toBe(true)
  })

  it('replaces existing mark', () => {
    const result = applyMark(BODY_WITH_MARK, '2026-05-18', [])
    expect(result.endsWith('Проекції: жодної.\n')).toBe(true)
    expect(result).not.toContain('2026-05-17')
    expect(result).not.toContain('03-components')
  })

  it('preserves horizontal rule inside body when adding mark', () => {
    const result = applyMark(BODY_WITH_RULE_IN_BODY, '2026-05-18', ['04-code'])
    expect(result).toContain('More content after a horizontal rule that is NOT a mark.')
    expect(result.endsWith('- [04-code](../ci4/04-code.md)\n')).toBe(true)
  })
})
