import { useAgent as useAgentBase } from '@7n/tauri-components/vue'
import { TOOLS } from '../tool/catalog.js'
import { systemPrompt } from '../tool/prompt.js'

// In-app agent gateway: binds the shared @7n/tauri-components agent to mlmail's
// Gmail tool catalog + domain prompt. Tauri specifics (omlx over tauri-http,
// tools/journal via tauri-plugin-agent) are wired inside the base composable.

/**
 * @returns {object} the in-app agent gateway (baseUrl/model/apiKey refs, journal, request/respond/approve)
 */
export function useAgent() {
  return useAgentBase({
    catalog: TOOLS,
    systemPrompt,
    omlx: { storagePrefix: 'mlmail' }
  })
}
