import { createDispatch } from './dispatch.js'
import { tauriTransport } from './transports.js'

// In-app entry to the tool surface. The UI and the (next) in-app LLM runner
// share this single dispatch instance (Tauri transport). dispatch(name, input)
// returns the uniform { ok, output } / { ok:false, error } envelope.

export const dispatch = createDispatch(tauriTransport)
export { listTools, toolManifest } from './manifest.js'
