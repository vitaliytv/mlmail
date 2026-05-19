import { describe, it, expect } from 'bun:test'
import { detectTriggers } from '../../docs-regen/triggers.js'

const baseManifest = () => ({
  version: 1,
  adrs: {
    foo: { path: 'docs/adr/foo.md', processed_at: '2026-05-18', projections: [] },
    bar: { path: 'docs/adr/bar.md', processed_at: '2026-05-18', projections: ['01-context'] },
  },
  templates: { '_global.prompt.md': { hash: 'sha256:t1' } },
  rules: { '.cursor/rules/n-ci4.mdc': { hash: 'sha256:r1' } },
  projections: {},
})

describe('detectTriggers', () => {

  it('returns all empty when nothing changed', () => {
    const adrs = [
      { slug: 'foo', hasMark: true },
      { slug: 'bar', hasMark: true },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result).toEqual({
      unmarked: [],
      removed: [],
      rulesChanged: false,
      templatesChanged: false,
    })
  })

  it('detects unmarked ADRs', () => {
    const adrs = [
      { slug: 'foo', hasMark: false },
      { slug: 'bar', hasMark: true },
      { slug: 'new-adr', hasMark: false },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result.unmarked.sort()).toEqual(['foo', 'new-adr'])
    expect(result.removed).toEqual([])
  })

  it('detects removed ADRs', () => {
    const adrs = [{ slug: 'foo', hasMark: true }]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result.removed).toEqual(['bar'])
  })

  it('detects rule hash drift', () => {
    const adrs = [
      { slug: 'foo', hasMark: true },
      { slug: 'bar', hasMark: true },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r2-NEW',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result.rulesChanged).toBe(true)
  })

  it('detects template hash drift', () => {
    const adrs = [
      { slug: 'foo', hasMark: true },
      { slug: 'bar', hasMark: true },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t2-NEW' },
    })
    expect(result.templatesChanged).toBe(true)
  })

  it('detects added template (not in manifest)', () => {
    const adrs = []
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: {
        '_global.prompt.md': 'sha256:t1',
        '01-context.prompt.md': 'sha256:newone',
      },
    })
    expect(result.templatesChanged).toBe(true)
  })
})
