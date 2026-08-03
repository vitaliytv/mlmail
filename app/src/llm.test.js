import { describe, it, expect, vi, beforeEach } from 'vitest'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { useLlm } = await import('./llm.js')

beforeEach(() => {
  invokeMock.mockReset()
  localStorage.clear()
})

describe('useLlm', () => {
  it('defaults provider/model to the given options when nothing is stored', () => {
    const { provider, model } = useLlm({ storagePrefix: 'test', defaultProvider: 'omlx', defaultModel: 'gemma' })
    expect(provider.value).toBe('omlx')
    expect(model.value).toBe('gemma')
  })

  it('save() persists provider/model under the storagePrefix, reloaded by a later useLlm()', () => {
    const first = useLlm({ storagePrefix: 'test' })
    first.provider.value = 'turbofieldfare'
    first.model.value = 'gemma-4-26b-a4b-it'
    first.save()

    const second = useLlm({ storagePrefix: 'test' })
    expect(second.provider.value).toBe('turbofieldfare')
    expect(second.model.value).toBe('gemma-4-26b-a4b-it')
  })

  it('loadEnv() auto-resolves an empty model via llm_list_models', async () => {
    invokeMock.mockResolvedValue(['gemma-4-e4b', 'gemma-4-26b'])
    const { model, loadEnv } = useLlm({ storagePrefix: 'test' })
    await loadEnv()
    expect(invokeMock).toHaveBeenCalledWith('llm_list_models', { provider: 'omlx' })
    expect(model.value).toBe('gemma-4-e4b')
  })

  it('loadEnv() is a no-op once a model is already set', async () => {
    const { loadEnv } = useLlm({ storagePrefix: 'test', defaultModel: 'gemma' })
    await loadEnv()
    expect(invokeMock).not.toHaveBeenCalled()
  })

  it('loadEnv() leaves model empty when llm_list_models fails or is unavailable', async () => {
    invokeMock.mockRejectedValue(new Error('not running under Tauri'))
    const { model, loadEnv } = useLlm({ storagePrefix: 'test' })
    await loadEnv()
    expect(model.value).toBe('')
  })

  it('chat() invokes llm_chat with "provider/model" and returns { content }', async () => {
    invokeMock.mockResolvedValue('Привіт!')
    const { chat } = useLlm({ storagePrefix: 'test', defaultProvider: 'turbofieldfare', defaultModel: 'gemma-4-26b-a4b-it' })
    const reply = await chat({ system: 'sys', user: 'hi' })
    expect(invokeMock).toHaveBeenCalledWith('llm_chat', {
      modelSpec: 'turbofieldfare/gemma-4-26b-a4b-it',
      system: 'sys',
      user: 'hi'
    })
    expect(reply).toEqual({ content: 'Привіт!' })
  })

  it('chat() rejects with a timeout error when timeoutMs elapses before llm_chat resolves', async () => {
    invokeMock.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve('late'), 50)))
    const { chat } = useLlm({ storagePrefix: 'test', defaultModel: 'gemma' })
    await expect(chat({ user: 'hi', timeoutMs: 5 })).rejects.toThrow(/no response within/)
  })
})
