import { invoke } from '@tauri-apps/api/core'

// UI/in-app transport: route a tool call to its Tauri command. Input keys map
// 1:1 to the command's args (camelCase). Commands with no input get `{}`.

/**
 * @param {object} tool tool definition (uses `tool.tauri`)
 * @param {object} input tool input, forwarded as command args
 * @returns {Promise<unknown>} the command result
 */
export function tauriTransport(tool, input) {
  return Object.keys(input ?? {}).length ? invoke(tool.tauri, input) : invoke(tool.tauri)
}
