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
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
  })

  it('hydrates email when backend reports authenticated', async () => {
    invokeMock.mockImplementation(cmd => {
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
    invokeMock.mockResolvedValueOnce()
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

describe('useAuthStore inbox count', () => {
  it('refreshInboxCount sets inboxCount on success', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(348)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxCount.value).toBe(348)
    expect(store.inboxErrorKind.value).toBe(null)
  })

  it('login also refreshes inbox count', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_start_login') return Promise.resolve({ email: 'u@e' })
      if (cmd === 'gmail_inbox_count') return Promise.resolve(12)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.login()
    expect(store.inboxCount.value).toBe(12)
  })

  it('captures error.kind on Gmail failure (Http)', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.reject(Object.assign(new Error('Http'), { kind: 'Http' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxCount.value).toBe(null)
    expect(store.inboxErrorKind.value).toBe('Http')
  })

  it('falls back to Unknown when Gmail error has no kind', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.reject(new Error('boom'))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxErrorKind.value).toBe('Unknown')
  })

  it('ReauthRequired from Gmail forces logout state', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count')
        return Promise.reject(Object.assign(new Error('ReauthRequired'), { kind: 'ReauthRequired' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
    expect(store.inboxCount.value).toBe(null)
  })

  it('logout clears inboxCount and inboxErrorKind', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(9)
      if (cmd === 'auth_logout') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxCount.value).toBe(9)
    await store.logout()
    expect(store.inboxCount.value).toBe(null)
    expect(store.inboxErrorKind.value).toBe(null)
  })

  it('does not call gmail_inbox_count when not authenticated', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_inbox_count')
    expect(store.inboxCount.value).toBe(null)
  })
})

describe('useAuthStore random message', () => {
  const sampleMessage = { id: 'm1', from: 'a@e', subject: 's', date: 'd', body: 'b' }

  it('initialize loads currentMessage after successful auth', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value).toEqual(sampleMessage)
    expect(store.messageErrorKind.value).toBe(null)
    expect(store.isMessageLoading.value).toBe(false)
  })

  it('login also loads currentMessage', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_start_login') return Promise.resolve({ email: 'u@e' })
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.login()
    expect(store.currentMessage.value).toEqual(sampleMessage)
  })

  it('loadRandomMessage replaces currentMessage', async () => {
    let call = 0
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') {
        call += 1
        return Promise.resolve({ ...sampleMessage, id: `m${call}` })
      }
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value.id).toBe('m1')
    await store.loadRandomMessage()
    expect(store.currentMessage.value.id).toBe('m2')
  })

  it('captures Empty kind from Gmail', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(0)
      if (cmd === 'gmail_random_message') return Promise.reject(Object.assign(new Error('Empty'), { kind: 'Empty' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value).toBe(null)
    expect(store.messageErrorKind.value).toBe('Empty')
  })

  it('ReauthRequired from random message forces logout state', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message')
        return Promise.reject(Object.assign(new Error('ReauthRequired'), { kind: 'ReauthRequired' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
    expect(store.currentMessage.value).toBe(null)
  })

  it('logout clears currentMessage and messageErrorKind', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      if (cmd === 'auth_logout') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value).toEqual(sampleMessage)
    await store.logout()
    expect(store.currentMessage.value).toBe(null)
    expect(store.messageErrorKind.value).toBe(null)
    expect(store.isMessageLoading.value).toBe(false)
  })

  it('does not call gmail_random_message when not authenticated', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_random_message')
    expect(store.currentMessage.value).toBe(null)
  })
})
