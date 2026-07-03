// Pure helpers for the "rule from this email" feature: derive a sender/subject
// pattern from a viewed message and turn it into a Gmail search query. Kept
// side-effect free so they're trivially unit-testable; the LLM subject
// suggestion (I/O) lives in composables/use-pattern.js.

/**
 * Extract the bare email address from a raw `From` header.
 * `npm <support@npmjs.com>` → `support@npmjs.com`; a bare address is returned
 * as-is.
 * @param {string|null|undefined} from raw From header value
 * @returns {string} the email address (or trimmed input when no angle brackets)
 */
export function parseFromEmail(from) {
  if (!from) return ''
  const lt = from.indexOf('<')
  const gt = lt === -1 ? -1 : from.indexOf('>', lt + 1)
  const raw = lt !== -1 && gt !== -1 ? from.slice(lt + 1, gt) : from
  return raw.trim()
}

/**
 * Build a Gmail search query from a sender and/or subject phrase. Embedded
 * quotes are stripped so the `subject:"…"` phrase stays well-formed.
 * @param {{ from?: string, subject?: string }} pattern sender/subject pattern
 * @returns {string} a Gmail query (empty when both parts are blank)
 */
export function buildPatternQuery({ from, subject } = {}) {
  const parts = []
  const f = (from ?? '').trim()
  const s = (subject ?? '').trim()
  if (f) parts.push(`from:${f}`)
  if (s) parts.push(`subject:"${s.replaceAll('"', '')}"`)
  return parts.join(' ')
}

const QUOTE_CHARS = new Set(['"', "'", '`'])

/**
 * Clean a raw LLM subject-pattern suggestion: take its first line, strip
 * wrapping quotes/backticks, and fall back to `fallback` when the result is
 * empty or not a string.
 * @param {unknown} raw model output
 * @param {string} fallback value to use when the suggestion is unusable
 * @returns {string} a usable subject phrase
 */
export function sanitizeSubjectSuggestion(raw, fallback) {
  if (typeof raw !== 'string') return fallback
  const firstLine = raw.split('\n')[0].trim()
  let start = 0
  let end = firstLine.length
  while (start < end && QUOTE_CHARS.has(firstLine[start])) start++
  while (end > start && QUOTE_CHARS.has(firstLine[end - 1])) end--
  const unquoted = firstLine.slice(start, end).trim()
  return unquoted || fallback
}
