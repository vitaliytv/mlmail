import { useAcpAgent } from '@7n/tauri-components/vue'
import { homeDir } from '@tauri-apps/api/path'
import { TOOLS } from '../tool/catalog.js'

// In-app ACP agent gateway for mlmail. Domain catalog stays here; agent kinds
// and model tiers come from the backend (`acp_list_tiers` via useAcpAgent) —
// no frontend spawn presets (removed in @7n/tauri-components@0.15+).
//
// cwd is homeDir() as a sane default for spawned CLIs (packaged app has no
// checkout of this repo). Falls back to "." outside Tauri so a missing home
// dir can't crash the module graph via an unhandled top-level await rejection.
let cwd
try {
  cwd = await homeDir()
} catch {
  cwd = '.'
}

/**
 * @returns {object} the in-app ACP agent gateway (agentKind/modelTier refs, journal, loadEnv/request/respond/approve)
 */
export function useAgent() {
  return useAcpAgent({
    catalog: TOOLS,
    cwd
  })
}
