import { invoke } from '@tauri-apps/api/core'
import { readonly, ref } from 'vue'

const _email = ref(null)
const _isAuthenticated = ref(false)
const _isLoading = ref(false)
const _errorKind = ref(null)

export function useAuthStore() {
  async function initialize() {
    const ok = await invoke('auth_is_authenticated')
    _isAuthenticated.value = ok
    if (ok) {
      _email.value = await invoke('auth_current_email')
    }
  }

  async function login() {
    _isLoading.value = true
    _errorKind.value = null
    try {
      const session = await invoke('auth_start_login')
      _email.value = session.email
      _isAuthenticated.value = true
    } catch (err) {
      _errorKind.value = err && typeof err === 'object' && err.kind ? err.kind : 'Unknown'
    } finally {
      _isLoading.value = false
    }
  }

  async function getAccessToken() {
    return invoke('auth_get_access_token')
  }

  async function logout() {
    await invoke('auth_logout')
    _email.value = null
    _isAuthenticated.value = false
    _errorKind.value = null
  }

  return {
    email: readonly(_email),
    isAuthenticated: readonly(_isAuthenticated),
    isLoading: readonly(_isLoading),
    errorKind: readonly(_errorKind),
    initialize,
    login,
    getAccessToken,
    logout
  }
}

export function _resetForTest() {
  _email.value = null
  _isAuthenticated.value = false
  _isLoading.value = false
  _errorKind.value = null
}
