import { invoke } from '@tauri-apps/api/core'
import { dispatch } from '../tool/index.js'

const _email = ref(null)
const _isAuthenticated = ref(false)
const _isLoading = ref(false)
const _errorKind = ref(null)
const _inboxCount = ref(null)
const _inboxErrorKind = ref(null)
const _currentMessage = ref(null)
const _messageErrorKind = ref(null)
const _isMessageLoading = ref(false)
const _isUnsubscribing = ref(false)
const _unsubscribeErrorKind = ref(null)
const _isSaving = ref(false)
const _saveErrorKind = ref(null)
const _isTrashing = ref(false)
const _trashErrorKind = ref(null)
const _isTrashingQuery = ref(false)
const _trashQueryErrorKind = ref(null)
const _lastTrashedCount = ref(null)
const _isCreatingFilter = ref(false)
const _filterErrorKind = ref(null)
const _filterCreated = ref(false)
const _actionLog = ref(/** @type {{ ts: number, text: string }[]} */ ([]))

/**
 * @returns {Promise<string>} access token
 */
function getAccessToken() {
  return invoke('auth_get_access_token')
}

/**
 * @param {unknown} error caught backend error
 * @returns {string} normalized error kind
 */
function getErrorKind(error) {
  return error?.kind || 'Unknown'
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
    const result = await dispatch('inbox_count')
    if (result.ok) {
      _inboxCount.value = result.output
      _inboxErrorKind.value = null
      return
    }
    const kind = result.error.kind ?? 'Unknown'
    _inboxCount.value = null
    _inboxErrorKind.value = kind
    if (kind === 'ReauthRequired') {
      _email.value = null
      _isAuthenticated.value = false
    }
  }

  /**
   *
   */
  async function loadRandomMessage() {
    if (!_isAuthenticated.value) return
    _isMessageLoading.value = true
    _messageErrorKind.value = null
    const result = await dispatch('random_message')
    if (result.ok) {
      _currentMessage.value = result.output
    } else {
      const kind = result.error.kind ?? 'Unknown'
      _currentMessage.value = null
      _messageErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _isMessageLoading.value = false
  }

  /**
   *
   */
  async function unsubscribeFromCurrent() {
    const action = _currentMessage.value?.unsubscribe
    if (!action) return
    _isUnsubscribing.value = true
    _unsubscribeErrorKind.value = null
    const result = await dispatch('unsubscribe', { action })
    if (result.ok) {
      await loadRandomMessage()
    } else {
      const kind = result.error.kind ?? 'Unknown'
      _unsubscribeErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _isUnsubscribing.value = false
  }

  /**
   *
   */
  async function trashCurrent() {
    const id = _currentMessage.value?.id
    if (!id) return
    _isTrashing.value = true
    _trashErrorKind.value = null
    const result = await dispatch('trash', { id })
    if (result.ok) {
      await Promise.all([loadRandomMessage(), refreshInboxCount()])
    } else {
      const kind = result.error.kind ?? 'Unknown'
      _trashErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _isTrashing.value = false
  }

  async function saveCurrent() {
    const id = _currentMessage.value?.id
    if (!id) return
    _isSaving.value = true
    _saveErrorKind.value = null
    const result = await dispatch('save', { id })
    if (result.ok) {
      await Promise.all([loadRandomMessage(), refreshInboxCount()])
    } else {
      const kind = result.error.kind ?? 'Unknown'
      _saveErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _isSaving.value = false
  }

  /**
   * Move every inbox message matching a Gmail query to Trash, then refresh.
   * @param {string} q non-empty Gmail search query
   */
  async function trashByQuery(q) {
    if (!q || !q.trim()) return
    _isTrashingQuery.value = true
    _trashQueryErrorKind.value = null
    _lastTrashedCount.value = null
    const result = await dispatch('trash_query', { q })
    if (result.ok) {
      const trashed = result.output?.trashed ?? 0
      _lastTrashedCount.value = trashed
      _actionLog.value.unshift({ ts: Date.now(), text: `Видалено ${trashed} лист(ів) за запитом: ${q}` })
      await Promise.all([loadRandomMessage(), refreshInboxCount()])
    } else {
      const kind = result.error.kind ?? 'Unknown'
      _trashQueryErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _isTrashingQuery.value = false
  }

  /**
   * Create a Gmail filter that auto-trashes future mail matching the pattern.
   * @param {{ from?: string, subject?: string }} pattern sender/subject criteria
   */
  async function createFilter({ from, subject } = {}) {
    _isCreatingFilter.value = true
    _filterErrorKind.value = null
    _filterCreated.value = false
    const result = await dispatch('create_filter', { from, subject })
    if (result.ok) {
      _filterCreated.value = true
    } else {
      const kind = result.error.kind ?? 'Unknown'
      _filterErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _isCreatingFilter.value = false
  }

  /**
   * Clear transient feedback from the pattern panel (errors, counts, success).
   */
  function clearPatternFeedback() {
    _trashQueryErrorKind.value = null
    _lastTrashedCount.value = null
    _filterErrorKind.value = null
    _filterCreated.value = false
  }

  /**
   *
   */
  async function initialize() {
    const authed = await dispatch('is_authenticated')
    const ok = authed.ok ? authed.output : false
    _isAuthenticated.value = ok
    if (ok) {
      const email = await dispatch('current_email')
      _email.value = email.ok ? email.output : null
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
      _errorKind.value = getErrorKind(error)
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
    _isUnsubscribing.value = false
    _unsubscribeErrorKind.value = null
    _isSaving.value = false
    _saveErrorKind.value = null
    _isTrashing.value = false
    _trashErrorKind.value = null
    _isTrashingQuery.value = false
    _trashQueryErrorKind.value = null
    _lastTrashedCount.value = null
    _isCreatingFilter.value = false
    _filterErrorKind.value = null
    _filterCreated.value = false
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
    isUnsubscribing: readonly(_isUnsubscribing),
    unsubscribeErrorKind: readonly(_unsubscribeErrorKind),
    isSaving: readonly(_isSaving),
    saveErrorKind: readonly(_saveErrorKind),
    isTrashing: readonly(_isTrashing),
    trashErrorKind: readonly(_trashErrorKind),
    isTrashingQuery: readonly(_isTrashingQuery),
    trashQueryErrorKind: readonly(_trashQueryErrorKind),
    lastTrashedCount: readonly(_lastTrashedCount),
    isCreatingFilter: readonly(_isCreatingFilter),
    filterErrorKind: readonly(_filterErrorKind),
    filterCreated: readonly(_filterCreated),
    actionLog: readonly(_actionLog),
    initialize,
    login,
    getAccessToken,
    logout,
    refreshInboxCount,
    loadRandomMessage,
    unsubscribeFromCurrent,
    saveCurrent,
    trashCurrent,
    trashByQuery,
    createFilter,
    clearPatternFeedback,
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
  _isUnsubscribing.value = false
  _unsubscribeErrorKind.value = null
  _isTrashing.value = false
  _trashErrorKind.value = null
  _isTrashingQuery.value = false
  _trashQueryErrorKind.value = null
  _lastTrashedCount.value = null
  _isCreatingFilter.value = false
  _filterErrorKind.value = null
  _filterCreated.value = false
}
