import { describe, it, expect, vi, beforeEach } from 'vitest'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { useAuthStore, _resetForTest } = await import('./auth-store.js')

/**
 *
 */
beforeEach(() => {
  invokeMock.mockReset()
  _resetForTest()
})

describe('useAuthStore initial state', () => {
  it('starts with the public loading and auth flags disabled', async () => {
    vi.resetModules()
    const { useAuthStore: useFreshAuthStore } = await import('./auth-store.js')
    const store = useFreshAuthStore()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.isLoading.value).toBe(false)
    expect(store.isMessageLoading.value).toBe(false)
    expect(store.isUnsubscribing.value).toBe(false)
    expect(store.onlyNewsletters.value).toBe(false)
  })
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
    expect(invokeMock).not.toHaveBeenCalledWith('auth_current_email')
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_inbox_count')
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_random_message')
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

  it('sets isLoading=true while login is waiting for the backend', async () => {
    const { promise: loginPromise, resolve: resolveLogin } = Promise.withResolvers()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_start_login') return loginPromise
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    const pendingLogin = store.login()
    expect(store.isLoading.value).toBe(true)
    resolveLogin({ email: 'a@b' })
    await pendingLogin
    expect(store.isLoading.value).toBe(false)
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

  it('clears transient message and newsletter flags', async () => {
    const newsletter = {
      id: 'm1',
      from: 'n@l',
      subject: 's',
      date: 'd',
      body: 'b',
      unsubscribe: { kind: 'OneClick', url: 'https://l.com/u/x' }
    }
    const { promise: unsubscribePromise, resolve: resolveUnsubscribe } = Promise.withResolvers()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe') return unsubscribePromise
      if (cmd === 'auth_logout') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    store.setOnlyNewsletters(true)
    const pendingUnsubscribe = store.unsubscribeFromCurrent()
    expect(store.isUnsubscribing.value).toBe(true)
    resolveUnsubscribe()
    await pendingUnsubscribe
    await store.logout()
    expect(store.isUnsubscribing.value).toBe(false)
    expect(store.onlyNewsletters.value).toBe(false)
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

describe('useAuthStore.unsubscribeFromCurrent', () => {
  const newsletter = {
    id: 'm1',
    from: 'n@l',
    subject: 's',
    date: 'd',
    body: 'b',
    unsubscribe: { kind: 'OneClick', url: 'https://l.com/u/x' }
  }

  it('invokes gmail_unsubscribe with action from currentMessage', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(invokeMock).toHaveBeenCalledWith('gmail_unsubscribe', { action: newsletter.unsubscribe })
  })

  it('loads next random message after successful unsubscribe', async () => {
    let randomCall = 0
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') {
        randomCall += 1
        return Promise.resolve({ ...newsletter, id: `m${randomCall}` })
      }
      if (cmd === 'gmail_unsubscribe') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value.id).toBe('m1')
    await store.unsubscribeFromCurrent()
    expect(store.currentMessage.value.id).toBe('m2')
  })

  it('is a no-op when currentMessage has no unsubscribe action', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve({ ...newsletter, unsubscribe: null })
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_unsubscribe', expect.anything())
  })

  it('captures error.kind when gmail_unsubscribe fails', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe') return Promise.reject(Object.assign(new Error('Http'), { kind: 'Http' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(store.unsubscribeErrorKind.value).toBe('Http')
    // currentMessage is NOT replaced on failure
    expect(store.currentMessage.value.id).toBe('m1')
  })

  it('resets isUnsubscribing to false after completion', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(store.isUnsubscribing.value).toBe(false)
  })
})

describe('useAuthStore.refreshInboxCount direct call', () => {
  it('is no-op when not authenticated', async () => {
    // _resetForTest sets _isAuthenticated = false
    const store = useAuthStore()
    await store.refreshInboxCount()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_inbox_count')
    expect(store.inboxCount.value).toBe(null)
  })

  it('sets inboxCount directly when called on authenticated store', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(77)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'gmail_inbox_count') return Promise.resolve(99)
      return Promise.resolve(null)
    })
    await store.refreshInboxCount()
    expect(store.inboxCount.value).toBe(99)
  })

  it('stores Unknown kind when error has no .kind', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'gmail_inbox_count') return Promise.reject(new Error('boom'))
      return Promise.resolve(null)
    })
    await store.refreshInboxCount()
    expect(store.inboxErrorKind.value).toBe('Unknown')
    expect(store.inboxCount.value).toBe(null)
  })

  it('stores error.kind when error is a structured object', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'gmail_inbox_count') return Promise.reject(Object.assign(new Error('Http'), { kind: 'Http' }))
      return Promise.resolve(null)
    })
    await store.refreshInboxCount()
    expect(store.inboxErrorKind.value).toBe('Http')
  })
})

describe('useAuthStore.loadRandomMessage direct call', () => {
  it('is no-op when not authenticated', async () => {
    const store = useAuthStore()
    await store.loadRandomMessage()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_random_message')
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_random_newsletter')
    expect(store.currentMessage.value).toBe(null)
  })

  it('sets isMessageLoading=true during the invoke', async () => {
    let loadingDuringCall = null
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'gmail_random_message') {
        loadingDuringCall = store.isMessageLoading.value
        return Promise.resolve({ id: 'x', from: 'a', subject: 's', date: 'd', body: 'b' })
      }
      return Promise.resolve(null)
    })
    await store.loadRandomMessage()
    expect(loadingDuringCall).toBe(true)
    expect(store.isMessageLoading.value).toBe(false)
  })

  it('stores Unknown kind when error has no .kind', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'gmail_random_message') return Promise.reject(new Error('oops'))
      return Promise.resolve(null)
    })
    await store.loadRandomMessage()
    expect(store.messageErrorKind.value).toBe('Unknown')
    expect(store.currentMessage.value).toBe(null)
  })

  it('stores error.kind when error is a structured object', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'gmail_random_message') return Promise.reject(Object.assign(new Error('Parse'), { kind: 'Parse' }))
      return Promise.resolve(null)
    })
    await store.loadRandomMessage()
    expect(store.messageErrorKind.value).toBe('Parse')
  })
})

describe('useAuthStore.unsubscribeFromCurrent additional', () => {
  it('is no-op when currentMessage is null', async () => {
    // _resetForTest sets currentMessage = null
    const store = useAuthStore()
    await store.unsubscribeFromCurrent()
    expect(invokeMock).not.toHaveBeenCalled()
    expect(store.isUnsubscribing.value).toBe(false)
  })

  it('sets isUnsubscribing=true during the invoke', async () => {
    let unsubscribingDuringCall = null
    const newsletter = {
      id: 'm1',
      from: 'n@l',
      subject: 's',
      date: 'd',
      body: 'b',
      unsubscribe: { kind: 'OneClick', url: 'https://l.com/u/x' }
    }
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe') {
        unsubscribingDuringCall = useAuthStore().isUnsubscribing.value
        return Promise.resolve()
      }
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(unsubscribingDuringCall).toBe(true)
    expect(store.isUnsubscribing.value).toBe(false)
  })

  it('stores Unknown kind when unsubscribe error has no .kind', async () => {
    const newsletter = {
      id: 'm1',
      from: 'n@l',
      subject: 's',
      date: 'd',
      body: 'b',
      unsubscribe: { kind: 'OneClick', url: 'https://l.com/u/x' }
    }
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe') return Promise.reject(new Error('boom'))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(store.unsubscribeErrorKind.value).toBe('Unknown')
  })

  it('forces logout state on ReauthRequired from unsubscribe', async () => {
    const newsletter = {
      id: 'm1',
      from: 'n@l',
      subject: 's',
      date: 'd',
      body: 'b',
      unsubscribe: { kind: 'OneClick', url: 'https://l.com/u/x' }
    }
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') return Promise.resolve(newsletter)
      if (cmd === 'gmail_unsubscribe')
        return Promise.reject(Object.assign(new Error('ReauthRequired'), { kind: 'ReauthRequired' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.unsubscribeFromCurrent()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
  })
})

describe('_resetForTest', () => {
  it('resets transient message loading flags to their defaults', async () => {
    const { promise: messagePromise, resolve: resolveMessage } = Promise.withResolvers()
    let messageCall = 0
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message') {
        messageCall += 1
        if (messageCall === 1) return Promise.resolve({ id: 'initial', from: 'a', subject: 's', date: 'd', body: 'b' })
        return messagePromise
      }
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    const pendingLoad = store.loadRandomMessage()
    expect(store.isMessageLoading.value).toBe(true)
    store.setOnlyNewsletters(true)
    _resetForTest()
    expect(store.isMessageLoading.value).toBe(false)
    expect(store.onlyNewsletters.value).toBe(false)
    resolveMessage({ id: 'm1', from: 'a', subject: 's', date: 'd', body: 'b' })
    await pendingLoad
  })
})

describe('useAuthStore.onlyNewsletters', () => {
  it('routes loadRandomMessage to gmail_random_newsletter when enabled', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_newsletter')
        return Promise.resolve({ id: 'n1', from: 'a', subject: 's', date: 'd', body: 'b', unsubscribe: null })
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    store.setOnlyNewsletters(true)
    await store.loadRandomMessage()
    expect(invokeMock).toHaveBeenCalledWith('gmail_random_newsletter', {})
    expect(store.currentMessage.value.id).toBe('n1')
  })

  it('uses gmail_random_message when disabled (default)', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(1)
      if (cmd === 'gmail_random_message')
        return Promise.resolve({ id: 'r1', from: 'a', subject: 's', date: 'd', body: 'b', unsubscribe: null })
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    await store.loadRandomMessage()
    expect(store.onlyNewsletters.value).toBe(false)
    expect(invokeMock).toHaveBeenCalledWith('gmail_random_message', {})
  })
})
