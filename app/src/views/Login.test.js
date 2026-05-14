import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invokeMock(...args) }))

const { _resetForTest } = await import('../services/auth-store.js')
const loginModule = await import('./Login.vue')
const Login = loginModule.default

beforeEach(() => {
  invokeMock.mockReset()
  _resetForTest()
})

describe('Login.vue', () => {
  it('renders "Увійти через Google" when not authenticated', async () => {
    invokeMock.mockResolvedValue(false)
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('Увійти через Google')
  })

  it('renders "Ви увійшли як ..." and "Вийти" when authenticated', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('me@example.com')
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('Ви увійшли як me@example.com')
    expect(w.text()).toContain('Вийти')
  })

  it('shows Ukrainian error after failed login', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      if (cmd === 'auth_start_login') return Promise.reject(Object.assign(new Error('Network'), { kind: 'Network' }))
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    await w.find('button').trigger('click')
    await flushPromises()
    expect(w.text()).toContain("Не вдалося з'єднатися з Google. Перевірте мережу.")
  })

  it('calls auth_start_login when sign-in button clicked', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      if (cmd === 'auth_start_login') return Promise.resolve({ email: 'x@y' })
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    await w.find('button').trigger('click')
    await flushPromises()
    expect(invokeMock).toHaveBeenCalledWith('auth_start_login')
  })

  it('calls auth_logout when sign-out button clicked', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('m@e')
      if (cmd === 'auth_logout') return Promise.resolve()
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    const buttons = w.findAll('button')
    const logoutBtn = buttons.find((b) => b.text() === 'Вийти')
    await logoutBtn.trigger('click')
    await flushPromises()
    expect(invokeMock).toHaveBeenCalledWith('auth_logout')
    expect(w.text()).toContain('Увійти через Google')
  })
})
