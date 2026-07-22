import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

// Local re-implementation of the direct omlx (OpenAI-compatible local LLM)
// chat client that @7n/tauri-components dropped in 0.11.0 in favor of the
// ACP agent path (spawned codex/claude/cursor/pi CLIs — see use-agent.js).
// The summarize/ask/pattern/newsletter-render/call-analysis composables still
// need a plain chat-completion call against the local omlx server rather than
// spawning a CLI agent, so that client lives here now.

const DIRECT_OMLX_BASE_URL = 'http://127.0.0.1:8000/v1'
const PROXY_OMLX_BASE_URL = 'http://127.0.0.1:8088/v1'
const PROXY_PROBE_TIMEOUT_MS = 400
const PROXY_PROBE_TTL_MS = 12_000

/**
 * Build a `chat` function that calls an OpenAI-compatible endpoint (omlx).
 * @param {object} params config
 * @param {string} params.baseUrl base URL incl. /v1 (e.g. http://127.0.0.1:8000/v1)
 * @param {string} params.model served model id
 * @param {string} [params.apiKey] optional bearer token
 * @param {typeof fetch} [params.fetchFn] fetch implementation (injectable for tests / tauri-http)
 * @returns {(req: {messages: object[], tools: object[]}) => Promise<object>} chat function
 */
export function createOpenAiChat({ baseUrl, model, apiKey, fetchFn = fetch }) {
  return async function chat({ messages, tools }) {
    const response = await fetchFn(`${baseUrl}/chat/completions`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        ...(apiKey && { authorization: `Bearer ${apiKey}` })
      },
      body: JSON.stringify({ model, messages, tools, tool_choice: 'auto' })
    })
    if (!response.ok) {
      throw new Error(`omlx ${response.status}: ${await response.text()}`)
    }
    const data = await response.json()
    return data.choices[0].message
  }
}

/**
 * List the models a local omlx (OpenAI-compatible) server has loaded, via
 * GET {baseUrl}/models. Generic OpenAI shape: { data: [{ id }] }. Returns []
 * on any failure so callers degrade gracefully.
 * @param {{ baseUrl?: string, apiKey?: string, fetchFn?: typeof fetch }} [params] config
 * @returns {Promise<string[]>} loaded model ids (empty on error)
 */
async function listOmlxModels({ baseUrl, apiKey, fetchFn = fetch } = {}) {
  if (!baseUrl) return []
  try {
    const response = await fetchFn(`${baseUrl}/models`, {
      headers: apiKey ? { authorization: `Bearer ${apiKey}` } : {}
    })
    if (!response.ok) return []
    const data = await response.json()
    return Array.isArray(data?.data) ? data.data.map(m => m?.id).filter(Boolean) : []
  } catch {
    return []
  }
}

/**
 * Is this URL the default local omlx server — the only target eligible for
 * the myllm proxy override? A user who deliberately pointed the app at
 * another host/port must never be silently rerouted.
 * @param {string} url candidate base URL
 * @returns {boolean} true for 127.0.0.1:8000 / localhost:8000, false otherwise (incl. parse errors)
 */
function isDirectOmlxUrl(url) {
  try {
    const { host } = new URL(url)
    return host === '127.0.0.1:8000' || host === 'localhost:8000'
  } catch {
    return false
  }
}

// proxyUrl -> { promise, expiresAt }. Caching the promise (not the value)
// dedupes concurrent probes when several composables call loadEnv() at once.
const proxyProbeCache = new Map()

/**
 * Probe the myllm reverse proxy (logs every omlx request/response for its
 * history view) at {@link PROXY_OMLX_BASE_URL}/health; route through it while
 * it's alive, otherwise talk to omlx directly. Cached per TTL so callers may
 * resolve before every LLM call without paying probe latency each time.
 * @param {string} directUrl the base URL to fall back to
 * @returns {Promise<string>} the base URL to use
 */
function resolveOmlxBaseUrlCached(directUrl) {
  const cached = proxyProbeCache.get(directUrl)
  if (cached && Date.now() < cached.expiresAt) return cached.promise

  const promise = (async () => {
    try {
      const healthUrl = new URL('/health', PROXY_OMLX_BASE_URL).href
      const signal =
        typeof AbortSignal?.timeout === 'function' ? AbortSignal.timeout(PROXY_PROBE_TIMEOUT_MS) : undefined
      const response = await tauriFetch(healthUrl, { signal })
      return response.ok ? PROXY_OMLX_BASE_URL : directUrl
    } catch {
      return directUrl
    }
  })()
  proxyProbeCache.set(directUrl, { promise, expiresAt: Date.now() + PROXY_PROBE_TTL_MS })
  return promise
}

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
 * Persisted config for the local omlx server (OpenAI-compatible MLX) that
 * drives the direct-chat composables. baseUrl/model are edited by the user
 * and cached in localStorage; loadEnv() probes the myllm proxy and routes
 * through it while alive (runtime-only override, never persisted).
 * storagePrefix namespaces the localStorage keys per composable/feature.
 * @param {{ storagePrefix?: string, defaultBaseUrl?: string, defaultModel?: string }} [options] config
 * @returns {{ baseUrl: import('vue').Ref<string>, model: import('vue').Ref<string>, apiKey: import('vue').Ref<string>, save: () => void, loadEnv: () => Promise<void> }} persisted omlx config, an env loader and a saver
 */
export function useOmlx({ storagePrefix = 'agent', defaultBaseUrl = DIRECT_OMLX_BASE_URL, defaultModel = '' } = {}) {
  const baseUrlKey = `${storagePrefix}:omlxBaseUrl`
  const modelKey = `${storagePrefix}:omlxModel`

  const baseUrl = ref(readStored(baseUrlKey) || defaultBaseUrl)
  const model = ref(readStored(modelKey) || defaultModel)
  // Filled from the global env in loadEnv(); never persisted to localStorage.
  const apiKey = ref('')

  /**
   * Pull OMLX_* from the user's global env (via Rust) and apply them, then
   * probe the myllm proxy when the resolved base is the default local one.
   * No-op outside Tauri (tests / web) or when the Rust command is missing.
   * @returns {Promise<void>}
   */
  async function loadEnv() {
    try {
      const env = await invoke('omlx_config')
      if (env) {
        if (env.apiKey) apiKey.value = env.apiKey
        if (env.baseUrl && !readStored(baseUrlKey)) baseUrl.value = env.baseUrl
        if (env.model && !readStored(modelKey)) model.value = env.model
      }
    } catch {
      // not running under Tauri, or the omlx_config command isn't registered
      // — keep localStorage / defaults
    }
    if (isDirectOmlxUrl(baseUrl.value)) {
      baseUrl.value = await resolveOmlxBaseUrlCached(baseUrl.value)
    }
    // The server requires a non-empty model per request (no server-side
    // default); when the user hasn't picked one, auto-resolve the first
    // loaded model. Not persisted — re-resolved every call so a model
    // swapped in omlx is picked up without editing localStorage by hand.
    if (!model.value) {
      const models = await listOmlxModels({ baseUrl: baseUrl.value, apiKey: apiKey.value })
      if (models.length > 0) model.value = models[0]
    }
  }

  /**
   * Persist baseUrl/model to localStorage. The API key is intentionally NOT
   * persisted — it comes from the global OMLX_API_KEY env on each launch.
   */
  function save() {
    writeStored(baseUrlKey, baseUrl.value)
    writeStored(modelKey, model.value)
  }

  return { baseUrl, model, apiKey, save, loadEnv }
}
