/** Керує явним OpenAI-compatible endpoint, моделлю й тимчасовим ключем для локальних LLM-функцій. */
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const LOCAL_OPENAI_PROVIDER = 'local-openai'
const LEGACY_PROVIDERS = new Set(['omlx', 'turbofieldfare', 'litellm'])
const transientApiKeys = new Map()

/**
 * Read a localStorage value, or null when localStorage is unavailable
 * (component tests without a DOM store, SSR).
 * @param {string} key storage key
 * @returns {string|null} stored value
 */
function readStored(key) {
  try {
    return globalThis.localStorage?.getItem(key) ?? null
  } catch {
    return null
  }
}

/**
 * Write a localStorage value, or no-op when storage is unavailable.
 * @param {string} key storage key
 * @param {string} value stored value
 */
function writeStored(key, value) {
  try {
    globalThis.localStorage?.setItem(key, value)
  } catch {
    // Tests and SSR can lack localStorage; the reactive state still works.
  }
}

/**
 * Convert an OpenAI-compatible API root into its canonical `/v1/` form.
 * @param {string} raw user-provided base URL
 * @returns {string} canonical URL
 */
export function normalizeBaseUrl(raw) {
  let url
  try {
    url = new URL(raw.trim())
  } catch {
    throw new Error('Вкажіть повну адресу LLM, наприклад http://127.0.0.1:8080/v1/.')
  }
  if (!['http:', 'https:'].includes(url.protocol) || !url.hostname) {
    throw new Error('Адреса LLM має використовувати http:// або https://.')
  }
  let path = url.pathname
  while (path.endsWith('/')) path = path.slice(0, -1)
  if (url.search || url.hash || path !== '/v1') {
    throw new Error('Адреса LLM має завершуватися на /v1/.')
  }
  url.pathname = '/v1/'
  return url.href
}

/**
 * Race a chat call against a UI timeout without cancelling its native request.
 * @param {Promise<T>} promise pending operation
 * @param {number} timeoutMs maximum UI wait
 * @returns {Promise<T>} operation result or timeout
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
 * Migrate prior provider and URL keys into the single local OpenAI profile.
 * @param {string} storagePrefix settings namespace
 * @returns {{baseUrl: string, model: string}} persisted non-secret values
 */
function loadPersistedConfig(storagePrefix) {
  const providerKey = `${storagePrefix}:llmProvider`
  const baseUrlKey = `${storagePrefix}:llmBaseUrl`
  const modelKey = `${storagePrefix}:llmModel`
  const savedProvider = readStored(providerKey)
  if (LEGACY_PROVIDERS.has(savedProvider)) writeStored(providerKey, LOCAL_OPENAI_PROVIDER)
  const savedBaseUrl = readStored(baseUrlKey) || readStored(`${storagePrefix}:omlxBaseUrl`) || ''
  if (savedBaseUrl && !readStored(baseUrlKey)) writeStored(baseUrlKey, savedBaseUrl)
  const savedModel = readStored(modelKey) || readStored(`${storagePrefix}:omlxModel`) || ''
  if (savedModel && !readStored(modelKey)) writeStored(modelKey, savedModel)
  return { baseUrl: savedBaseUrl, model: savedModel }
}

/**
 * Persisted endpoint/model selection and memory-only API-key state for direct
 * local-LLM features. Every request carries the explicit endpoint to Rust;
 * neither `.omlx` files nor a hard-coded port participate in resolution.
 * @param {{ storagePrefix?: string, defaultModel?: string }} [options] config
 * @returns {{ provider: import('vue').Ref<string>, baseUrl: import('vue').Ref<string>, apiKey: import('vue').Ref<string>, model: import('vue').Ref<string>, save: () => void, loadEnv: () => Promise<void>, refreshModels: () => Promise<string[]>, chat: (params: { system?: string, user: string, timeoutMs?: number }) => Promise<{content: string}> }} local LLM configuration and chat surface
 */
export function useLlm({ storagePrefix = 'agent', defaultModel = '' } = {}) {
  const { baseUrl: savedBaseUrl, model: savedModel } = loadPersistedConfig(storagePrefix)
  const provider = ref(LOCAL_OPENAI_PROVIDER)
  const baseUrl = ref(savedBaseUrl)
  const model = ref(savedModel || defaultModel)
  const apiKey = transientApiKeys.get(storagePrefix) || ref('')
  transientApiKeys.set(storagePrefix, apiKey)

  /**
   * Pull a launch-time endpoint from Rust only when the user has none saved.
   * @returns {Promise<void>} resolves after endpoint/model preparation
   */
  async function loadEnv() {
    if (!baseUrl.value) {
      const config = await invoke('llm_default_config')
      if (config?.baseUrl) baseUrl.value = config.baseUrl
    }
    if (!baseUrl.value) {
      throw new Error('LLM не налаштовано. Відкрийте «Налаштування LLM» і вкажіть адресу сервера.')
    }
    if (!model.value) await refreshModels()
  }

  /**
   * Validate the endpoint through Rust and use its first model if none is selected.
   * @returns {Promise<string[]>} models reported by the endpoint
   */
  async function refreshModels() {
    const models = await invoke('llm_list_models', {
      baseUrl: normalizeBaseUrl(baseUrl.value),
      apiKey: apiKey.value || null
    })
    if (!model.value && models.length > 0) model.value = models[0]
    return models
  }

  /**
   * Persist only endpoint and model. API keys intentionally remain in memory.
   * @returns {void} no return value
   */
  function save() {
    const canonicalUrl = normalizeBaseUrl(baseUrl.value)
    baseUrl.value = canonicalUrl
    writeStored(`${storagePrefix}:llmProvider`, LOCAL_OPENAI_PROVIDER)
    writeStored(`${storagePrefix}:llmBaseUrl`, canonicalUrl)
    writeStored(`${storagePrefix}:llmModel`, model.value)
  }

  /**
   * Send a one-shot request using the currently selected endpoint and model.
   * @param {{ system?: string, user: string, timeoutMs?: number }} params request data
   * @returns {Promise<{content: string}>} assistant response
   */
  async function chat({ system, user, timeoutMs }) {
    await loadEnv()
    if (!model.value) throw new Error('LLM не повернула жодної моделі. Перевірте налаштування сервера.')
    const call = invoke('llm_chat', {
      modelSpec: `${LOCAL_OPENAI_PROVIDER}/${model.value}`,
      system,
      user,
      baseUrl: normalizeBaseUrl(baseUrl.value),
      apiKey: apiKey.value || null
    })
    const content = await (timeoutMs ? withTimeout(call, timeoutMs) : call)
    return { content }
  }

  return { provider, baseUrl, apiKey, model, save, loadEnv, refreshModels, chat }
}
