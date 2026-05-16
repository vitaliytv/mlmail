import { invoke } from '@tauri-apps/api/core'

const _email = ref(null)
const _isAuthenticated = ref(false)
const _isLoading = ref(false)
const _errorKind = ref(null)
const _inboxCount = ref(null)
const _inboxErrorKind = ref(null)
const _currentMessage = ref(null)
const _messageErrorKind = ref(null)
const _isMessageLoading = ref(false)

/**
 * @returns {Promise<string>} access token
 */
function getAccessToken() {
  return invoke('auth_get_access_token')
}

/**
 * @returns {object} auth store
 */
export function useAuthStore() {
  /**
   *
   */
  async function refreshInboxCount() {
    if (!_isAuthenticated.value) return
    try {
      _inboxCount.value = await invoke('gmail_inbox_count')
      _inboxErrorKind.value = null
    } catch (error) {
      const kind = error && typeof error === 'object' && error.kind ? error.kind : 'Unknown'
      _inboxCount.value = null
      _inboxErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
  }

  /**
   *
   */
  async function loadRandomMessage() {
    if (!_isAuthenticated.value) return
    _isMessageLoading.value = true
    _messageErrorKind.value = null
    try {
      _currentMessage.value = await invoke('gmail_random_message')
    } catch (error) {
      const kind = error && typeof error === 'object' && error.kind ? error.kind : 'Unknown'
      _currentMessage.value = null
      _messageErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    } finally {
      _isMessageLoading.value = false
    }
  }

  /**
   *
   */
  async function initialize() {
    const ok = await invoke('auth_is_authenticated')
    _isAuthenticated.value = ok
    if (ok) {
      _email.value = await invoke('auth_current_email')
      await refreshInboxCount()
      await loadRandomMessage()
    }
  }

  /**
   *
   */
  async function login() {
    _isLoading.value = true
    _errorKind.value = null
    try {
      const session = await invoke('auth_start_login')
      _email.value = session.email
      _isAuthenticated.value = true
      await refreshInboxCount()
      await loadRandomMessage()
    } catch (error) {
      _errorKind.value = error && typeof error === 'object' && error.kind ? error.kind : 'Unknown'
    } finally {
      _isLoading.value = false
    }
  }

  /**
   *
   */
  async function logout() {
    await invoke('auth_logout')
    _email.value = null
    _isAuthenticated.value = false
    _errorKind.value = null
    _inboxCount.value = null
    _inboxErrorKind.value = null
    _currentMessage.value = null
    _messageErrorKind.value = null
    _isMessageLoading.value = false
  }

  return {
    email: readonly(_email),
    isAuthenticated: readonly(_isAuthenticated),
    isLoading: readonly(_isLoading),
    errorKind: readonly(_errorKind),
    inboxCount: readonly(_inboxCount),
    inboxErrorKind: readonly(_inboxErrorKind),
    currentMessage: readonly(_currentMessage),
    messageErrorKind: readonly(_messageErrorKind),
    isMessageLoading: readonly(_isMessageLoading),
    initialize,
    login,
    getAccessToken,
    logout,
    refreshInboxCount,
    loadRandomMessage
  }
}

/**
 *
 */
export function _resetForTest() {
  _email.value = null
  _isAuthenticated.value = false
  _isLoading.value = false
  _errorKind.value = null
  _inboxCount.value = null
  _inboxErrorKind.value = null
  _currentMessage.value = null
  _messageErrorKind.value = null
  _isMessageLoading.value = false
}
