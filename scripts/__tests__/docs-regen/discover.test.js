import { describe, it, expect } from 'bun:test'
import { join } from 'node:path'
import { discoverCleanAdrs } from '../../docs-regen/discover.js'

const FIXTURE_ROOT = join(import.meta.dir, 'fixtures', 'discover')

describe('discoverCleanAdrs', () => {
  it('returns only clean ADRs, ignoring drafts and _inbox', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const slugs = adrs.map((a) => a.slug).sort()
    expect(slugs).toEqual(['clean', 'clean-status', 'with-mark'])
  })

  it('detects hasMark correctly per ADR', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const bySlug = Object.fromEntries(adrs.map((a) => [a.slug, a]))
    expect(bySlug['clean'].hasMark).toBe(false)
    expect(bySlug['clean-status'].hasMark).toBe(false)
    expect(bySlug['with-mark'].hasMark).toBe(true)
  })

  it('returns path relative to rootDir', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const paths = adrs.map((a) => a.path).sort()
    expect(paths).toEqual([
      'docs/adr/clean-status.md',
      'docs/adr/clean.md',
      'docs/adr/with-mark.md',
    ])
  })

  it('returns body and rawContent for each ADR', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const clean = adrs.find((a) => a.slug === 'clean')
    expect(clean.rawContent).toContain('Clean ADR без frontmatter')
    expect(clean.body).toContain('Clean ADR без frontmatter')
  })

  it('throws on slug collision', async () => {
    const collisionRoot = join(import.meta.dir, 'fixtures', 'collision')
    await expect(discoverCleanAdrs(collisionRoot)).rejects.toThrow(/slug collision/)
  })
})
