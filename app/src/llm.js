import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Platform-agnostic local-LLM composable for the direct-chat composables
// (summarize/ask/pattern/newsletter-render/call-analysis). Replaces the old
// omlx.js, which fetched an OpenAI-compatible endpoint straight from the
// webview — a desktop-only assumption. The actual HTTP call (and the
// desktop/Android split — omlx/litellm/turbofieldfare via Rust's llm-lib vs.
// LiteRT-LM) now lives behind the Rust `llm_chat`/`llm_list_models`/
// `llm_providers` commands (app/src-tauri/src/llm.rs), so Vue only tracks
// *which* provider/model it wants, never a baseUrl.

/**
 * Read a localStorage value, or null when localStorage is unavailable
 * (component tests without a DOM store, SSR).
 * @param {string} key storage key
 * @returns {string|null} stored value, or null
 */
function readStored(key) {
  try {
    return globalThis.localStorage?.getItem(key) ?? null
  } catch {
    return null
  }
}

/**
 * Write a localStorage value; no-op when localStorage is unavailable.
 * @param {string} key storage key
 * @param {string} value value to store
 */
function writeStored(key, value) {
  try {
    globalThis.localStorage?.setItem(key, value)
  } catch {
    // no localStorage (tests / SSR) — in-memory ref state is still updated
  }
}

/**
 * Race `promise` against a rejection after `timeoutMs` — mirrors the old
 * `AbortSignal.timeout`-wrapped fetch's fail-fast UX (some local servers
 * stall indefinitely on a large batch instead of erroring). Doesn't cancel
 * the underlying Rust-side call, only stops the UI from waiting on it.
 * @param {Promise<T>} promise the pending call
 * @param {number} timeoutMs abort the wait after this many milliseconds
 * @returns {Promise<T>} `promise`'s value, or a timeout rejection
 * @template T
 */
function withTimeout(promise, timeoutMs) {
  return Promise.race([
    promise,
    new Promise((_resolve, reject) => {
      setTimeout(() => reject(new Error(`llm chat: no response within ${timeoutMs}ms`)), timeoutMs)
    })
  ])
}

/**
 * Persisted provider/model selection for the local-LLM direct-chat
 * composables. `storagePrefix` namespaces the localStorage keys per
 * composable/feature.
 * @param {{ storagePrefix?: string, defaultProvider?: string, defaultModel?: string }} [options] config
 * @returns {{
 *   provider: import('vue').Ref<string>,
 *   model: import('vue').Ref<string>,
 *   save: () => void,
 *   loadEnv: () => Promise<void>,
 *   chat: (params: { system?: string, user: string, timeoutMs?: number }) => Promise<{content: string}>
 * }} persisted local-LLM config, an env loader, a saver and the chat call
 */
export function useLlm({ storagePrefix = 'agent', defaultProvider = 'omlx', defaultModel = '' } = {}) {
  const providerKey = `${storagePrefix}:llmProvider`
  const modelKey = `${storagePrefix}:llmModel`

  const provider = ref(readStored(providerKey) || defaultProvider)
  const model = ref(readStored(modelKey) || defaultModel)

  /**
   * Auto-resolve `model` via `llm_list_models` when the user hasn't picked
   * one yet — the server requires a non-empty model per request (no
   * server-side default). Not persisted — re-resolved every call so a model
   * swapped on the server is picked up without editing localStorage by hand.
   * No-op outside Tauri (tests / web) or when the command isn't registered.
   * @returns {Promise<void>}
   */
  async function loadEnv() {
    if (model.value) return
    try {
      const models = await invoke('llm_list_models', { provider: provider.value })
      if (models.length > 0) model.value = models[0]
    } catch {
      // not running under Tauri, or no models loaded — keep empty; chat()
      // below will surface whatever error the backend gives for an empty model.
    }
  }

  /** Persist provider/model to localStorage. */
  function save() {
    writeStored(providerKey, provider.value)
    writeStored(modelKey, model.value)
  }

  /**
   * One-shot chat via the Rust `llm_chat` command — always system+single-user,
   * never tools (every composable using this is one-shot).
   * @param {{ system?: string, user: string, timeoutMs?: number }} params chat input
   * @returns {Promise<{content: string}>} the assistant's reply
   */
  async function chat({ system, user, timeoutMs }) {
    const call = invoke('llm_chat', { modelSpec: `${provider.value}/${model.value}`, system, user })
    const content = await (timeoutMs ? withTimeout(call, timeoutMs) : call)
    return { content }
  }

  return { provider, model, save, loadEnv, chat }
}
