import { SAMPLE_SIDEBAR_STREAM, validateA2uiStream } from '@7n/tauri-components/a2ui'
import { describe, expect, it } from 'vitest'

describe('a2ui validate (from @7n/tauri-components)', () => {
  it('accepts sample sidebar stream', () => {
    const r = validateA2uiStream(SAMPLE_SIDEBAR_STREAM)
    expect(r.ok).toBe(true)
    expect(r.surfaces.has('sidebar.draft-helper')).toBe(true)
  })

  it('rejects unknown component so UI never renders it', () => {
    const r = validateA2uiStream([
      {
        version: 'v1.0',
        createSurface: {
          surfaceId: 's',
          catalogId: 'nitra.core',
          components: [{ id: 'root', component: 'WebView', url: 'https://x' }]
        }
      }
    ])
    expect(r.ok).toBe(false)
    expect(r.error).toMatch(/unknown component/)
  })
})
