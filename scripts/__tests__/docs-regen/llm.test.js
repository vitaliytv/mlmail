import { describe, it, expect } from 'bun:test'
import { parseLlmResponse } from '../../docs-regen/llm.js'

const PARSE_ERROR_RE = /parse/i
const CONTENT_ERROR_RE = /content/
const USED_ADRS_ERROR_RE = /used_adrs/

describe('parseLlmResponse', () => {
  it('parses raw JSON', () => {
    const raw = '{"content":"# Hello","used_adrs":["foo","bar"]}'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('# Hello')
    expect(result.used_adrs).toEqual(['foo', 'bar'])
  })

  it('parses JSON wrapped in markdown fence ```json', () => {
    const raw = '```json\n{"content":"# Hi","used_adrs":[]}\n```'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('# Hi')
    expect(result.used_adrs).toEqual([])
  })

  it('parses JSON wrapped in bare ``` fence', () => {
    const raw = '```\n{"content":"x","used_adrs":["a"]}\n```'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('x')
  })

  it('strips surrounding prose and finds JSON object', () => {
    const raw = 'Here is the JSON:\n\n{"content":"x","used_adrs":[]}\n\nThanks.'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('x')
  })

  it('throws on broken JSON', () => {
    expect(() => parseLlmResponse('{ not json')).toThrow(PARSE_ERROR_RE)
  })

  it('throws when content field missing', () => {
    expect(() => parseLlmResponse('{"used_adrs":[]}')).toThrow(CONTENT_ERROR_RE)
  })

  it('throws when used_adrs missing', () => {
    expect(() => parseLlmResponse('{"content":"x"}')).toThrow(USED_ADRS_ERROR_RE)
  })

  it('throws when used_adrs is not an array', () => {
    expect(() => parseLlmResponse('{"content":"x","used_adrs":"foo"}')).toThrow(USED_ADRS_ERROR_RE)
  })
})
