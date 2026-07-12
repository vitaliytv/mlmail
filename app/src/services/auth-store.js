import { invoke } from '@tauri-apps/api/core'
import { Notify } from 'quasar'
import { dispatch } from '../tool/index.js'
import { useTaskScan } from '../composables/use-task-scan.js'

const NETWORK_RETRY_DELAY_MS = 15000

/**
 * Like dispatch(), but on a Network-kind failure it notifies the user and
 * keeps retrying every 15s instead of surfacing the error — Gmail calls are
 * background/UI-triggered, not one-shot user actions, so a flaky connection
 * shouldn't need a manual retry.
 * @param {string} name tool name to dispatch
 * @param {object} [payload] tool payload
 * @returns {Promise<{ok: boolean, output?: unknown, error?: {kind?: string}}>} dispatch result
 */
async function dispatchWithRetry(name, payload) {
  for (;;) {
    const result = await dispatch(name, payload)
    if (result.ok || result.error?.kind !== 'Network') return result
    // Notify is only registered once Quasar is mounted (main.js); guard so this
    // composable stays importable in plain unit tests that skip that mount.
    if (typeof Notify.create === 'function') {
      Notify.create({
        type: 'negative',
        message: "Не вдалося з'єднатися з Google. Перевірте мережу.",
        caption: 'Повторна спроба через 15 с…',
        position: 'bottom',
        timeout: NETWORK_RETRY_DELAY_MS
      })
    }
    // oxlint-disable-next-line promise/avoid-new
    await new Promise(resolve => setTimeout(resolve, NETWORK_RETRY_DELAY_MS))
  }
}

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
const _isLoadingFilters = ref(false)
const _filtersErrorKind = ref(null)
const _filters = ref(
  /** @type {{ id: string, criteria: { from?: string, to?: string, subject?: string, query?: string, negatedQuery?: string, hasAttachment?: boolean, excludeChats?: boolean, size?: number, sizeComparison?: string }, action: { addLabelIds?: string[], removeLabelIds?: string[], forward?: string } }[]} */ ([])
)
const _isDeletingFilterId = ref(null)
const _deleteFilterErrorKind = ref(null)
const _openingAttachmentId = ref(null)
const _openAttachmentErrorKind = ref(null)
const _labels = ref(/** @type {{ id: string, name: string }[]} */ ([]))
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
    const result = await dispatchWithRetry('inbox_count')
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
    const result = await dispatchWithRetry('random_message')
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
   * Load one specific message by id (e.g. from a task-panel entry) and make
   * it the current message.
   * @param {string} id Gmail message id
   */
  async function loadMessageById(id) {
    if (!_isAuthenticated.value || !id) return
    _isMessageLoading.value = true
    _messageErrorKind.value = null
    const result = await dispatchWithRetry('read', { id })
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
    const result = await dispatchWithRetry('unsubscribe', { action })
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
    const result = await dispatchWithRetry('trash', { id })
    if (result.ok) {
      await Promise.all([loadRandomMessage(), refreshInboxCount(), useTaskScan().refresh()])
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

  /**
   *
   */
  async function saveCurrent() {
    const id = _currentMessage.value?.id
    if (!id) return
    _isSaving.value = true
    _saveErrorKind.value = null
    const result = await dispatchWithRetry('save', { id })
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
   * Download one attachment of the current message and open it with the OS
   * default app.
   * @param {{ attachment_id: string, filename: string }} attachment attachment to open
   */
  async function openAttachment({ attachment_id: attachmentId, filename } = {}) {
    const messageId = _currentMessage.value?.id
    if (!messageId || !attachmentId) return
    _openingAttachmentId.value = attachmentId
    _openAttachmentErrorKind.value = null
    const result = await dispatchWithRetry('open_attachment', { messageId, attachmentId, filename })
    if (!result.ok) {
      const kind = result.error.kind ?? 'Unknown'
      _openAttachmentErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
    _openingAttachmentId.value = null
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
    const result = await dispatchWithRetry('trash_query', { q })
    if (result.ok) {
      const trashed = result.output?.trashed ?? 0
      _lastTrashedCount.value = trashed
      _actionLog.value.unshift({ ts: Date.now(), text: `Видалено ${trashed} лист(ів) за запитом: ${q}` })
      await Promise.all([loadRandomMessage(), refreshInboxCount(), useTaskScan().refresh()])
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
    const result = await dispatchWithRetry('create_filter', { from, subject })
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
   * Fetch all configured Gmail filters.
   */
  async function listFilters() {
    _isLoadingFilters.value = true
    _filtersErrorKind.value = null
    try {
      _filters.value = await invoke('gmail_list_filters')
    } catch (error) {
      _filters.value = []
      _filtersErrorKind.value = getErrorKind(error)
      if (_filtersErrorKind.value === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    } finally {
      _isLoadingFilters.value = false
    }
  }

  /**
   * Delete a Gmail filter by id, then refresh the list.
   * @param {string} id filter id to delete
   */
  async function deleteFilter(id) {
    _isDeletingFilterId.value = id
    _deleteFilterErrorKind.value = null
    try {
      await invoke('gmail_delete_filter', { id })
      await listFilters()
    } catch (error) {
      _deleteFilterErrorKind.value = getErrorKind(error)
      if (_deleteFilterErrorKind.value === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    } finally {
      _isDeletingFilterId.value = null
    }
  }

  /**
   * Fetch all Gmail labels, used to resolve filter action label ids to names.
   * Best-effort: leaves the previous list in place on failure.
   */
  async function listLabels() {
    try {
      _labels.value = await invoke('gmail_list_labels')
    } catch {
      // Supplementary data for filter action tooltips; ignore failures.
    }
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
    _isLoadingFilters.value = false
    _filtersErrorKind.value = null
    _filters.value = []
    _isDeletingFilterId.value = null
    _deleteFilterErrorKind.value = null
    _labels.value = []
    _openingAttachmentId.value = null
    _openAttachmentErrorKind.value = null
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
    isLoadingFilters: readonly(_isLoadingFilters),
    filtersErrorKind: readonly(_filtersErrorKind),
    filters: readonly(_filters),
    isDeletingFilterId: readonly(_isDeletingFilterId),
    deleteFilterErrorKind: readonly(_deleteFilterErrorKind),
    labels: readonly(_labels),
    actionLog: readonly(_actionLog),
    openingAttachmentId: readonly(_openingAttachmentId),
    openAttachmentErrorKind: readonly(_openAttachmentErrorKind),
    initialize,
    login,
    getAccessToken,
    logout,
    refreshInboxCount,
    loadRandomMessage,
    loadMessageById,
    unsubscribeFromCurrent,
    saveCurrent,
    trashCurrent,
    trashByQuery,
    createFilter,
    listFilters,
    deleteFilter,
    listLabels,
    openAttachment,
    clearPatternFeedback
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
  _isLoadingFilters.value = false
  _filtersErrorKind.value = null
  _filters.value = []
  _isDeletingFilterId.value = null
  _deleteFilterErrorKind.value = null
  _openingAttachmentId.value = null
  _openAttachmentErrorKind.value = null
}
