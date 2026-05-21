import { mkdir, readFile, writeFile, access } from 'node:fs/promises'
import { join } from 'node:path'
import { sha256 } from './hash.js'

const TEMPLATE_NAMES = [
  '_global.prompt.md',
  '01-context.prompt.md',
  '02-containers.prompt.md',
  '03-components.prompt.md',
  '04-code.prompt.md',
  'decisions.prompt.md'
]

const TEMPLATE_DIR_REL = 'docs/ci4/_templates'

/**
 * Copy any missing prompt templates from the defaults directory.
 * @param {string} rootDir Repository root directory.
 * @param {string} defaultDir Directory holding the bundled default templates.
 * @returns {Promise<{created: string[]}>} Names of templates that were created.
 */
export async function bootstrapTemplates(rootDir, defaultDir) {
  const targetDir = join(rootDir, TEMPLATE_DIR_REL)
  await mkdir(targetDir, { recursive: true })
  const created = []
  for (const name of TEMPLATE_NAMES) {
    const target = join(targetDir, name)
    try {
      await access(target)
    } catch (error) {
      if (error.code !== 'ENOENT') throw error
      const source = join(defaultDir, name)
      const text = await readFile(source, 'utf8')
      await writeFile(target, text, 'utf8')
      created.push(name)
    }
  }
  return { created }
}

/**
 * Load all prompt templates from the project's template directory.
 * @param {string} rootDir Repository root directory.
 * @returns {Promise<Record<string, string>>} Template content keyed by file name.
 */
export async function loadTemplates(rootDir) {
  const dir = join(rootDir, TEMPLATE_DIR_REL)
  const out = {}
  for (const name of TEMPLATE_NAMES) {
    out[name] = await readFile(join(dir, name), 'utf8')
  }
  return out
}

/**
 * Compute SHA-256 hashes for every prompt template.
 * @param {string} rootDir Repository root directory.
 * @returns {Promise<Record<string, string>>} Template hashes keyed by file name.
 */
export async function templateHashes(rootDir) {
  const all = await loadTemplates(rootDir)
  const out = {}
  for (const [name, content] of Object.entries(all)) {
    out[name] = sha256(content)
  }
  return out
}
