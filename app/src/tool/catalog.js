// Single source of truth for the headless tool surface. Each tool is a named,
// schema-described callable reachable identically by the UI and the in-app LLM
// agent (@7n/tauri-components). Handlers delegate to a Tauri command (`tauri`).
//
// `tier` gates trust (read < write < destructive): an agent runs read/write
// directly; destructive calls pause for human approval. Lifecycle/secret
// commands (login, logout, access token) are intentionally NOT tools — they stay
// direct invokes in the auth store.

export const TOOLS = [
  {
    tier: 'read',
    name: 'is_authenticated',
    summary: 'Report whether a Gmail session is currently active.',
    input: {},
    tauri: 'auth_is_authenticated',
  },
  {
    tier: 'read',
    name: 'current_email',
    summary: 'Return the email address of the signed-in Gmail account.',
    input: {},
    tauri: 'auth_current_email',
  },
  {
    tier: 'read',
    name: 'inbox_count',
    summary: 'Return the total number of messages in the Gmail INBOX.',
    input: {},
    tauri: 'gmail_inbox_count',
  },
  {
    tier: 'read',
    name: 'search',
    summary: 'Search the inbox with a Gmail query (e.g. "from:bob unread", "subject:invoice", "newer_than:7d"). Returns up to 15 message summaries (id, from, subject, date).',
    input: {
      q: { type: 'string', required: true, description: 'Gmail search query. Empty string lists recent inbox messages.' },
    },
    tauri: 'gmail_search',
  },
  {
    tier: 'read',
    name: 'read',
    summary: 'Read one message in full (from, subject, date, body) by its id.',
    input: {
      id: { type: 'string', required: true, description: 'Gmail message id (from a search result).' },
    },
    tauri: 'gmail_read',
  },
  {
    tier: 'read',
    name: 'random_message',
    summary: 'Fetch one random message from the inbox (from, subject, date, body, unsubscribe).',
    input: {},
    tauri: 'gmail_random_message',
  },
  {
    tier: 'read',
    name: 'random_newsletter',
    summary: 'Fetch one random newsletter (message carrying a List-Unsubscribe header) from the inbox.',
    input: {},
    tauri: 'gmail_random_newsletter',
  },
  {
    tier: 'write',
    name: 'unsubscribe',
    summary: 'Unsubscribe from a newsletter via its List-Unsubscribe action (one-click POST, URL, or mailto).',
    input: {
      action: { type: 'object', required: true, description: 'The unsubscribe action object from a message (OneClick { url } | Url { url } | Mailto { to, subject? }).' },
    },
    tauri: 'gmail_unsubscribe',
  },
  {
    tier: 'write',
    name: 'save',
    summary: 'Apply the "Збережено" label to one message and archive it (removes from INBOX). The message stays permanently in Gmail under the Збережено label.',
    input: {
      id: { type: 'string', required: true, description: 'Gmail message id to save.' },
    },
    tauri: 'gmail_save',
  },
  {
    tier: 'destructive',
    name: 'trash',
    summary: 'Move one message to Trash by its id. Reversible (Gmail keeps Trash 30 days). Destructive — agents need human approval.',
    input: {
      id: { type: 'string', required: true, description: 'Gmail message id to trash (from a search result).' },
    },
    tauri: 'gmail_trash',
  },
  {
    tier: 'destructive',
    name: 'trash_query',
    summary: 'Move EVERY inbox message matching a Gmail query to Trash (paginated, batched). Reversible (Gmail keeps Trash 30 days). Destructive — agents need human approval.',
    input: {
      q: { type: 'string', required: true, description: 'Gmail search query selecting the messages to trash, e.g. `from:npm subject:"Successfully published"`. Must be non-empty (an empty query would match the whole inbox).' },
    },
    tauri: 'gmail_trash_query',
  },
  {
    tier: 'write',
    name: 'create_filter',
    summary: 'Create a Gmail filter that automatically moves future matching mail to Trash. Matches by sender and/or subject phrase.',
    input: {
      from: { type: 'string', required: false, description: 'Sender to match (email address or domain).' },
      subject: { type: 'string', required: false, description: 'Subject phrase to match.' },
    },
    validate: data => (!data.from?.trim() && !data.subject?.trim()) ? 'Provide at least one of from/subject' : null,
    tauri: 'gmail_create_filter',
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
