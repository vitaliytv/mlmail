// Single source of truth for the headless tool surface (n-tool-surface).
// Each tool is a named, schema-described callable reachable identically by the
// UI and (next) the on-device LLM agent — omlx on desktop, LiteRT-LM on
// Android. Handlers delegate to a Tauri command (`tauri`); the catalog stays the
// only place a tool is declared, and manifests/clients derive from it.
//
// `scope` gates intent: "safe" tools are read-only and freely callable by the
// LLM; "mutate" tools have side effects and require a confirm gate before an
// agent runs them. Lifecycle/secret commands (login, logout, access token) are
// intentionally NOT tools — they stay direct invokes in the auth store.

export const TOOLS = [
  {
    scope: 'safe',
    name: 'is_authenticated',
    summary: 'Report whether a Gmail session is currently active.',
    input: {},
    tauri: 'auth_is_authenticated',
  },
  {
    scope: 'safe',
    name: 'current_email',
    summary: 'Return the email address of the signed-in Gmail account.',
    input: {},
    tauri: 'auth_current_email',
  },
  {
    scope: 'safe',
    name: 'inbox_count',
    summary: 'Return the total number of messages in the Gmail INBOX.',
    input: {},
    tauri: 'gmail_inbox_count',
  },
  {
    scope: 'safe',
    name: 'random_message',
    summary: 'Fetch one random message from the inbox (from, subject, date, body, unsubscribe).',
    input: {},
    tauri: 'gmail_random_message',
  },
  {
    scope: 'safe',
    name: 'random_newsletter',
    summary: 'Fetch one random newsletter (message carrying a List-Unsubscribe header) from the inbox.',
    input: {},
    tauri: 'gmail_random_newsletter',
  },
  {
    scope: 'mutate',
    name: 'unsubscribe',
    summary: 'Unsubscribe from a newsletter via its List-Unsubscribe action (one-click POST, URL, or mailto).',
    input: {
      action: { type: 'object', required: true, description: 'The unsubscribe action object from a message (OneClick { url } | Url { url } | Mailto { to, subject? }).' },
    },
    tauri: 'gmail_unsubscribe',
  },
]

/**
 * Look up a tool by name.
 * @param {string} name tool name
 * @returns {object|null} the tool definition, or null if unknown
 */
export function getTool(name) {
  return TOOLS.find(tool => tool.name === name) ?? null
}