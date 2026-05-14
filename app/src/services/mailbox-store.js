import { invoke } from '@tauri-apps/api/core'
import { readonly, ref } from 'vue'

const _inboxCount = ref(null)
const _isLoading = ref(false)
const _errorKind = ref(null)

/**
 * @returns {object} mailbox store
 */
export function useMailboxStore() {
  /**
   *
   */
  async function fetchInboxCount() {
    _isLoading.value = true
    _errorKind.value = null
    try {
      _inboxCount.value = await invoke('get_inbox_count')
    } catch (error) {
      _errorKind.value = error && typeof error === 'object' && error.kind ? error.kind : 'Network'
    } finally {
      _isLoading.value = false
    }
  }

  return {
    inboxCount: readonly(_inboxCount),
    isLoading: readonly(_isLoading),
    errorKind: readonly(_errorKind),
    fetchInboxCount,
  }
}
