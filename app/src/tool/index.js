import { createDispatch } from '@7n/tauri-components'
import { tauriTransport } from '@7n/tauri-components/vue'
import { TOOLS } from './catalog.js'

// In-app entry to the tool surface for direct (non-agent) UI calls. Binds the
// shared dispatcher to this app's catalog and the Tauri transport; the in-app
// agent (useAgent) builds its own dispatch inside the kit. dispatch(name, input)
// returns the uniform { ok, output } / { ok:false, error } envelope.

export const dispatch = createDispatch(TOOLS, tauriTransport)
