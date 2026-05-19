import { globby } from 'globby'
import matter from 'gray-matter'
import { readFile } from 'node:fs/promises'
import { join, basename } from 'node:path'
import { hasMark } from './marks.js'

export async function discoverCleanAdrs(rootDir) {
  const paths = await globby(['docs/adr/**/*.md'], {
    cwd: rootDir,
    ignore: ['docs/adr/_inbox/**'],
  })

  const adrs = []
  const slugs = new Map()

  for (const relPath of paths.sort()) {
    const absPath = join(rootDir, relPath)
    const rawContent = await readFile(absPath, 'utf8')
    const parsed = matter(rawContent)
    if (parsed.data && parsed.data.session) continue

    const slug = basename(relPath, '.md')
    if (slugs.has(slug)) {
      throw new Error(
        `ADR slug collision: ${slug} (paths: ${slugs.get(slug)}, ${relPath})`,
      )
    }
    slugs.set(slug, relPath)

    adrs.push({
      slug,
      path: relPath,
      body: parsed.content,
      rawContent,
      hasMark: hasMark(rawContent),
    })
  }
  return adrs
}
