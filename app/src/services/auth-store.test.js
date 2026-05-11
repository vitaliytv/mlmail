import { describe, it, expect, vi, beforeEach } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { useAuthStore, _resetForTest } = await import('./auth-store.js')

beforeEach(() => {
  invokeMock.mockReset()
  _resetForTest()
})

describe('useAuthStore.initialize', () => {
  it('sets isAuthenticated=false when backend has no session', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
  })

  it('hydrates email when backend reports authenticated', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('user@example.com')
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(true)
    expect(store.email.value).toBe('user@example.com')
  })
})

describe('useAuthStore.login', () => {
  it('sets email and isAuthenticated on successful login', async () => {
    invokeMock.mockResolvedValue({ email: 'new@example.com' })
    const store = useAuthStore()
    await store.login()
    expect(store.email.value).toBe('new@example.com')
    expect(store.isAuthenticated.value).toBe(true)
    expect(store.errorKind.value).toBe(null)
  })

  it('captures error.kind when login throws structured error', async () => {
    invokeMock.mockRejectedValue({ kind: 'Network', message: 'timeout' })
    const store = useAuthStore()
    await store.login()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.errorKind.value).toBe('Network')
  })

  it('falls back to Unknown when error has no kind', async () => {
    invokeMock.mockRejectedValue('plain string')
    const store = useAuthStore()
    await store.login()
    expect(store.errorKind.value).toBe('Unknown')
  })

  it('resets isLoading to false after login completes', async () => {
    invokeMock.mockResolvedValue({ email: 'a@b' })
    const store = useAuthStore()
    await store.login()
    expect(store.isLoading.value).toBe(false)
  })
})

describe('useAuthStore.logout', () => {
  it('clears email and isAuthenticated', async () => {
    invokeMock.mockResolvedValueOnce({ email: 'a@b' })
    const store = useAuthStore()
    await store.login()
    invokeMock.mockResolvedValueOnce(undefined)
    await store.logout()
    expect(store.email.value).toBe(null)
    expect(store.isAuthenticated.value).toBe(false)
  })
})

describe('useAuthStore.getAccessToken', () => {
  it('invokes auth_get_access_token and returns its result', async () => {
    invokeMock.mockResolvedValue('AT-123')
    const store = useAuthStore()
    const tok = await store.getAccessToken()
    expect(tok).toBe('AT-123')
    expect(invokeMock).toHaveBeenCalledWith('auth_get_access_token')
  })
})

describe('useAuthStore singleton', () => {
  it('returns shared state across calls', async () => {
    invokeMock.mockResolvedValue({ email: 'shared@example.com' })
    const a = useAuthStore()
    await a.login()
    const b = useAuthStore()
    expect(b.email.value).toBe('shared@example.com')
  })
})
