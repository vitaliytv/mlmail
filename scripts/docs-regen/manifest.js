import { readFile, writeFile, rename, mkdir } from 'node:fs/promises'
import { join, dirname } from 'node:path'

const MANIFEST_REL_PATH = 'docs/ci4/manifest.json'

export function defaultManifest() {
  return {
    version: 1,
    generated_at: null,
    tool: { name: 'docs-regen', version: '0.1.0', model: null },
    rules: {},
    templates: {},
    adrs: {},
    projections: {},
  }
}

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

export async function saveManifest(rootDir, manifest) {
  const path = join(rootDir, MANIFEST_REL_PATH)
  const tmpPath = path + '.tmp'
  await mkdir(dirname(path), { recursive: true })
  const sorted = sortKeysDeep(manifest)
  const json = JSON.stringify(sorted, null, 2) + '\n'
  await writeFile(tmpPath, json, 'utf8')
  await rename(tmpPath, path)
}

function sortKeysDeep(value) {
  if (Array.isArray(value)) return value.map(sortKeysDeep)
  if (value !== null && typeof value === 'object') {
    const out = {}
    for (const key of Object.keys(value).sort()) {
      out[key] = sortKeysDeep(value[key])
    }
    return out
  }
  return value
}
