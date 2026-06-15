import { describe, it, expect, vi } from 'vitest'

const { TOOLS, getTool } = await import('./catalog.js')
const { createDispatch, validateInput } = await import('./dispatch.js')
const { toolManifest, listTools } = await import('./manifest.js')

describe('catalog', () => {
  it('every tool has scope, name, summary, input and a tauri command', () => {
    for (const tool of TOOLS) {
      expect(tool.scope).toBeTruthy()
      expect(tool.name).toBeTruthy()
      expect(tool.summary).toBeTruthy()
      expect(tool.input).toBeTypeOf('object')
      expect(tool.tauri).toBeTruthy()
    }
  })

  it('getTool resolves known and unknown names', () => {
    expect(getTool('inbox_count')?.tauri).toBe('gmail_inbox_count')
    expect(getTool('nope')).toBeNull()
  })

  it('marks read tools safe and unsubscribe mutate', () => {
    expect(getTool('random_message').scope).toBe('safe')
    expect(getTool('unsubscribe').scope).toBe('mutate')
  })
})

describe('validateInput', () => {
  it('passes when a no-input tool gets nothing', () => {
    expect(validateInput(getTool('inbox_count'))).toBeNull()
  })

  it('flags a missing required field', () => {
    expect(validateInput(getTool('unsubscribe'), {})).toBe('Missing required field: action')
  })

  it('flags a wrong type', () => {
    expect(validateInput(getTool('unsubscribe'), { action: 'x' })).toBe('Field "action" must be an object')
    expect(validateInput(getTool('unsubscribe'), { action: [] })).toBe('Field "action" must be an object')
  })

  it('passes a valid action object', () => {
    expect(validateInput(getTool('unsubscribe'), { action: { OneClick: { url: 'https://x' } } })).toBeNull()
  })
})

describe('dispatch', () => {
  it('returns an ok envelope from the transport output', async () => {
    const transport = vi.fn(() => 42)
    const result = await createDispatch(transport)('inbox_count', {})
    expect(result).toEqual({ ok: true, output: 42 })
    expect(transport).toHaveBeenCalledWith(getTool('inbox_count'), {})
  })

  it('rejects an unknown tool without calling the transport', async () => {
    const transport = vi.fn(() => null)
    const result = await createDispatch(transport)('nope', {})
    expect(result).toEqual({ ok: false, error: { code: 'not_found', message: 'Unknown tool: nope' } })
    expect(transport).not.toHaveBeenCalled()
  })

  it('rejects invalid input before the transport', async () => {
    const transport = vi.fn(() => null)
    const result = await createDispatch(transport)('unsubscribe', {})
    expect(result.ok).toBe(false)
    expect(result.error.code).toBe('validation')
    expect(transport).not.toHaveBeenCalled()
  })

  it('wraps a transport failure as an io error and preserves the backend kind', async () => {
    const transport = vi.fn(() => {
      const error = new Error('re-auth')
      error.kind = 'ReauthRequired'
      throw error
    })
    const result = await createDispatch(transport)('inbox_count', {})
    expect(result.ok).toBe(false)
    expect(result.error.code).toBe('io')
    expect(result.error.kind).toBe('ReauthRequired')
  })

  it('defaults kind to Unknown for plain errors', async () => {
    const transport = vi.fn(() => {
      throw new Error('boom')
    })
    const result = await createDispatch(transport)('inbox_count', {})
    expect(result.error.kind).toBe('Unknown')
    expect(result.error.message).toBe('boom')
  })
})

describe('manifest', () => {
  it('emits OpenAI function-calling tools from the catalog', () => {
    const manifest = toolManifest()
    expect(manifest).toHaveLength(TOOLS.length)
    const unsub = manifest.find(entry => entry.function.name === 'unsubscribe')
    expect(unsub).toMatchObject({
      type: 'function',
      function: { name: 'unsubscribe', parameters: { type: 'object', required: ['action'] } },
    })
    const count = manifest.find(entry => entry.function.name === 'inbox_count')
    expect(count.function.parameters).toEqual({ type: 'object', properties: {} })
  })

  it('lists tools as name + summary + scope', () => {
    const list = listTools()
    expect(list).toHaveLength(TOOLS.length)
    expect(list.every(item => item.name && item.summary && item.scope)).toBe(true)
  })
})
