import matter from 'gray-matter'
import { readFile } from 'node:fs/promises'
import { join, basename } from 'node:path'
import { glob } from 'tinyglobby'
import { hasMark } from './marks.js'

/**
 * Discover all clean (non-session) ADR files under `docs/adr`.
 * @param {string} rootDir Repository root directory.
 * @returns {Promise<Array<{slug: string, path: string, body: string, rawContent: string, hasMark: boolean}>>} Discovered ADRs sorted by path.
 */
export async function discoverCleanAdrs(rootDir) {
  const paths = await glob(['docs/adr/**/*.md'], {
    cwd: rootDir,
    ignore: ['docs/adr/_inbox/**']
  })

  const adrs = []
  const slugs = new Map()

  for (const relPath of paths.toSorted()) {
    const absPath = join(rootDir, relPath)
    const rawContent = await readFile(absPath, 'utf8')
    const parsed = matter(rawContent)
    if (parsed.data && parsed.data.session) continue

    const slug = basename(relPath, '.md')
    if (slugs.has(slug)) {
      throw new Error(`ADR slug collision: ${slug} (paths: ${slugs.get(slug)}, ${relPath})`)
    }
    slugs.set(slug, relPath)

    adrs.push({
      slug,
      path: relPath,
      body: parsed.content,
      rawContent,
      hasMark: hasMark(rawContent)
    })
  }
  return adrs
}
