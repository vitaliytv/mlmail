import { describe, it, expect, vi, beforeEach } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { mountQuasar } from './test-utils/quasar.js'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { _resetForTest } = await import('./services/auth-store.js')
const AppModule = await import('./App.vue')
const App = AppModule.default

beforeEach(() => {
  invokeMock.mockReset()
  _resetForTest()
})

describe('App.vue', () => {
  it('renders the Login view inside the Quasar layout', async () => {
    invokeMock.mockResolvedValue(false)
    const w = mountQuasar(App)
    await flushPromises()
    expect(w.text()).toContain('MLMaiL')
    expect(w.text()).toContain('Увійти через Google')
  })

  it('reflects the authenticated state from the auth store', async () => {
    invokeMock.mockImplementation(cmd => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('me@example.com')
      return Promise.resolve(null)
    })
    const w = mountQuasar(App)
    await flushPromises()
    expect(w.text()).toContain('Ви увійшли як me@example.com')
  })
})
