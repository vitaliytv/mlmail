import { readFile, writeFile, rename, mkdir } from 'node:fs/promises'
import { join, dirname } from 'node:path'

const MANIFEST_REL_PATH = 'docs/ci4/manifest.json'

/**
 * Build an empty docs-regen manifest.
 * @returns {object} Fresh manifest with default fields.
 */
export function defaultManifest() {
  return {
    version: 1,
    generated_at: null,
    tool: { name: 'docs-regen', version: '0.1.0', model: null },
    rules: {},
    templates: {},
    adrs: {},
    projections: {}
  }
}

/**
 * Load the docs-regen manifest, returning a default one if absent.
 * @param {string} rootDir Repository root directory.
 * @returns {Promise<object>} Parsed manifest object.
 */
export async function loadManifest(rootDir) {
  const path = join(rootDir, MANIFEST_REL_PATH)
  let text
  try {
    text = await readFile(path, 'utf8')
  } catch (error) {
    if (error.code === 'ENOENT') return defaultManifest()
    throw error
  }
  return JSON.parse(text)
}

/**
 * Atomically write the docs-regen manifest with deterministically sorted keys.
 * @param {string} rootDir Repository root directory.
 * @param {object} manifest Manifest object to persist.
 * @returns {Promise<void>} Resolves once the manifest is written.
 */
export async function saveManifest(rootDir, manifest) {
  const path = join(rootDir, MANIFEST_REL_PATH)
  const tmpPath = path + '.tmp'
  await mkdir(dirname(path), { recursive: true })
  const sorted = sortKeysDeep(manifest)
  const json = JSON.stringify(sorted, null, 2) + '\n'
  await writeFile(tmpPath, json, 'utf8')
  await rename(tmpPath, path)
}

/**
 * Recursively sort object keys to produce deterministic JSON output.
 * @param {unknown} value Value to normalize.
 * @returns {unknown} Value with all nested object keys sorted alphabetically.
 */
function sortKeysDeep(value) {
  if (Array.isArray(value)) return value.map(item => sortKeysDeep(item))
  if (value !== null && typeof value === 'object') {
    const out = {}
    for (const key of Object.keys(value).toSorted()) {
      out[key] = sortKeysDeep(value[key])
    }
    return out
  }
  return value
}
