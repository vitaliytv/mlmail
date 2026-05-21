import { describe, it, expect, beforeEach, afterEach } from 'bun:test'
import { randomUUID } from 'node:crypto'
import { join } from 'node:path'
import { mkdir, rm, writeFile, readFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { defaultManifest, loadManifest, saveManifest } from '../../docs-regen/manifest.js'

let TMP

beforeEach(async () => {
  TMP = join(tmpdir(), `manifest-test-${Date.now()}-${randomUUID()}`)
  await mkdir(join(TMP, 'docs', 'ci4'), { recursive: true })
})

afterEach(async () => {
  await rm(TMP, { recursive: true, force: true })
})

describe('defaultManifest', () => {
  it('has version 1 and empty adrs/projections/templates/rules', () => {
    const m = defaultManifest()
    expect(m.version).toBe(1)
    expect(m.adrs).toEqual({})
    expect(m.projections).toEqual({})
    expect(m.templates).toEqual({})
    expect(m.rules).toEqual({})
  })
})

describe('loadManifest', () => {
  it('returns defaultManifest when file is missing', async () => {
    const m = await loadManifest(TMP)
    expect(m.version).toBe(1)
    expect(m.adrs).toEqual({})
  })

  it('parses existing manifest', async () => {
    const stored = {
      version: 1,
      generated_at: '2026-05-18T16:00:00Z',
      adrs: { 'foo-adr': { path: 'docs/adr/foo-adr.md', processed_at: '2026-05-18', projections: ['01-context'] } },
      projections: {},
      templates: {},
      rules: {}
    }
    await writeFile(join(TMP, 'docs', 'ci4', 'manifest.json'), JSON.stringify(stored), 'utf8')
    const m = await loadManifest(TMP)
    expect(m.adrs['foo-adr'].processed_at).toBe('2026-05-18')
  })

  it('throws on invalid JSON', async () => {
    await writeFile(join(TMP, 'docs', 'ci4', 'manifest.json'), '{ not json', 'utf8')
    await expect(loadManifest(TMP)).rejects.toThrow()
  })
})

describe('saveManifest', () => {
  it('writes manifest with sorted keys and trailing newline', async () => {
    const m = {
      version: 1,
      tool: { name: 'docs-regen', version: '0.1.0' },
      adrs: {
        'z-adr': { path: 'docs/adr/z-adr.md', projections: [] },
        'a-adr': { path: 'docs/adr/a-adr.md', projections: ['01-context'] }
      },
      projections: {},
      templates: {},
      rules: {}
    }
    await saveManifest(TMP, m)
    const text = await readFile(join(TMP, 'docs', 'ci4', 'manifest.json'), 'utf8')
    expect(text.endsWith('\n')).toBe(true)
    // Both keys present, "a-adr" appears before "z-adr" in lexicographic order
    expect(text).toContain('"a-adr"')
    expect(text).toContain('"z-adr"')
    expect(text.indexOf('"a-adr"')).toBeLessThan(text.indexOf('"z-adr"'))
    // Top-level keys sorted: adrs before projections
    expect(text.indexOf('"adrs"')).toBeLessThan(text.indexOf('"projections"'))
  })

  it('round-trip: save then load gives equal structure', async () => {
    const m = {
      version: 1,
      generated_at: '2026-05-18T16:00:00Z',
      tool: { name: 'docs-regen', version: '0.1.0' },
      adrs: { foo: { path: 'docs/adr/foo.md', processed_at: '2026-05-18', projections: ['01-context'] } },
      projections: {
        '01-context': { path: 'docs/ci4/01-context.md', output_hash: 'sha256:abc', used_adrs: ['foo'] }
      },
      templates: { '_global.prompt.md': { hash: 'sha256:def' } },
      rules: { '.cursor/rules/n-ci4.mdc': { hash: 'sha256:ghi' } }
    }
    await saveManifest(TMP, m)
    const loaded = await loadManifest(TMP)
    expect(loaded).toEqual(m)
  })
})
