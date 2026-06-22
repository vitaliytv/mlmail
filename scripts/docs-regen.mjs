import { readFile, writeFile, stat } from 'node:fs/promises'
import { join, dirname } from 'node:path'
import { env } from 'node:process'
import { setTimeout as sleep } from 'node:timers/promises'
import { fileURLToPath } from 'node:url'

import { parseCliArgs } from './docs-regen/cli.js'
import { discoverCleanAdrs } from './docs-regen/discover.js'
import { loadManifest, saveManifest } from './docs-regen/manifest.js'
import { detectTriggers } from './docs-regen/triggers.js'
import { applyMark } from './docs-regen/marks.js'
import { acquireLock } from './docs-regen/lock.js'
import { Logger } from './docs-regen/log.js'
import { regenerateProjection } from './docs-regen/projection.js'
import { sha256 } from './docs-regen/hash.js'
import { bootstrapTemplates, loadTemplates, templateHashes } from './docs-regen/templates.js'

const ROOT_DIR = process.cwd()
const RULE_FILE = '.cursor/rules/n-ci4.mdc'
const PROJECTIONS = ['01-context', '02-containers', '03-components', '04-code', 'decisions']
const LOCK_PATH = '.claude/hooks/.docs-regen.lock'
const TOOL_VERSION = '0.1.0'

/**
 * Entry point: acquire the docs-regen lock and run the regeneration pipeline.
 * @returns {Promise<number>} Process exit code.
 */
async function main() {
  const args = parseCliArgs(process.argv.slice(2))
  const logger = new Logger(ROOT_DIR)
  const lock = await acquireLock(join(ROOT_DIR, LOCK_PATH))
  if (!lock.acquired) {
    logger.warn('Another docs:regen is running, exiting')
    await logger.flush()
    return 0
  }
  try {
    return await run(args, logger)
  } finally {
    await lock.release()
    await logger.flush()
  }
}

/**
 * Run the docs-regen pipeline: detect drift, regenerate projections, update marks and manifest.
 * @param {{projection?: string, all: boolean, dry: boolean, noMark: boolean, check: boolean}} args Parsed CLI options.
 * @param {Logger} logger Logger instance for progress output.
 * @returns {Promise<number>} Process exit code.
 */
async function run(args, logger) {
  if (await isInMergeOrRebase()) {
    logger.warn('Repository is in merge/rebase state, aborting')
    return 0
  }

  const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url))
  const DEFAULT_TEMPLATE_DIR = join(SCRIPT_DIR, 'docs-regen', 'default-templates')

  const { created } = await bootstrapTemplates(ROOT_DIR, DEFAULT_TEMPLATE_DIR)
  if (created.length > 0) {
    logger.info(`Templates bootstrapped: ${created.join(', ')}`)
  }

  const adrs = await discoverCleanAdrs(ROOT_DIR)
  logger.info(`Clean ADRs found: ${adrs.length}`)

  const manifest = await loadManifest(ROOT_DIR)

  const tplHashes = await templateHashes(ROOT_DIR)
  const ruleHash = await fileHash(ROOT_DIR, RULE_FILE)

  const triggers = detectTriggers({
    adrs,
    manifest,
    ruleHash,
    templateHashes: tplHashes
  })
  logger.info(
    `Triggers: ${triggers.unmarked.length} unmarked, ${triggers.removed.length} removed, ` +
      `rules changed: ${triggers.rulesChanged ? 'yes' : 'no'}, templates changed: ${triggers.templatesChanged ? 'yes' : 'no'}`
  )

  const needRegen =
    args.all ||
    triggers.unmarked.length > 0 ||
    triggers.removed.length > 0 ||
    triggers.rulesChanged ||
    triggers.templatesChanged

  if (args.check) {
    if (!needRegen) {
      logger.info('docs:regen --check: in sync')
      return 0
    }
    logger.error('docs:regen --check: drift detected, run `bun run docs:regen`')
    return 1
  }

  if (!needRegen) {
    logger.info('OK, nothing to regenerate')
    return 0
  }

  let projectionsToRun
  if (args.projection) {
    if (!PROJECTIONS.includes(args.projection)) {
      logger.error(`Unknown projection: ${args.projection}`)
      return 2
    }
    projectionsToRun = [args.projection]
  } else {
    projectionsToRun = PROJECTIONS
  }

  logger.info(`Will regenerate ${projectionsToRun.length} projection(s): ${projectionsToRun.join(', ')}`)
  if (args.dry) {
    logger.info('--dry: stopping before LLM calls')
    return 0
  }

  await sleep(3000)

  const templates = await loadTemplates(ROOT_DIR)

  const projectionResults = {}
  for (const name of projectionsToRun) {
    const currentPath = join(ROOT_DIR, 'docs/ci4', `${name}.md`)
    const currentContent = await readFile(currentPath, 'utf8').catch(() => '')
    logger.info(`Generating ${name}.md ...`)
    const result = await regenerateProjection({
      name,
      adrs,
      currentContent,
      templates,
      model: env.DOCS_REGEN_MODEL,
      rootDir: ROOT_DIR
    })
    await writeFile(currentPath, result.content, 'utf8')
    projectionResults[name] = {
      path: `docs/ci4/${name}.md`,
      output_hash: sha256(result.content),
      generated_at: new Date().toISOString(),
      used_adrs: result.used_adrs,
      prompt_length: result.prompt_length,
      output_length: result.output_length
    }
    logger.info(`  → wrote, ${result.used_adrs.length} ADRs used`)
  }

  const adrToProjections = new Map()
  for (const [name, r] of Object.entries(projectionResults)) {
    for (const slug of r.used_adrs) {
      if (!adrToProjections.has(slug)) adrToProjections.set(slug, new Set())
      adrToProjections.get(slug).add(name)
    }
  }

  if (args.noMark) {
    logger.info('Marks skipped (--no-mark)')
  } else {
    const todayMark = isoDate()
    for (const adr of adrs) {
      const projectionsUsed = (adrToProjections.get(adr.slug) ?? []).toSorted()
      const updated = applyMark(adr.rawContent, todayMark, projectionsUsed)
      if (updated !== adr.rawContent) {
        await writeFile(join(ROOT_DIR, adr.path), updated, 'utf8')
      }
    }
    logger.info(`Marks updated: ${adrs.length} ADRs`)
  }

  const today = isoDate()
  const newManifest = {
    version: 1,
    generated_at: new Date().toISOString(),
    tool: {
      name: 'docs-regen',
      version: TOOL_VERSION,
      model: env.DOCS_REGEN_MODEL || 'sonnet'
    },
    rules: { [RULE_FILE]: { hash: ruleHash } },
    templates: Object.fromEntries(Object.entries(tplHashes).map(([n, h]) => [n, { hash: h }])),
    adrs: Object.fromEntries(
      adrs.map(a => [
        a.slug,
        {
          path: a.path,
          processed_at: today,
          projections: (adrToProjections.get(a.slug) ?? []).toSorted()
        }
      ])
    ),
    projections: {}
  }

  for (const name of PROJECTIONS) {
    if (projectionResults[name]) {
      newManifest.projections[name] = projectionResults[name]
    } else if (manifest.projections?.[name]) {
      newManifest.projections[name] = manifest.projections[name]
    }
  }

  await saveManifest(ROOT_DIR, newManifest)
  logger.info('Manifest updated')

  return 0
}

/**
 * Check whether the repository is mid-merge or mid-rebase.
 * @returns {Promise<boolean>} True if a merge or rebase is in progress.
 */
async function isInMergeOrRebase() {
  // Resolve real git-dir (handles worktrees where `.git` is a file pointer).
  let gitDir
  try {
    const proc = Bun.spawn(['git', 'rev-parse', '--git-dir'], {
      cwd: ROOT_DIR,
      stdout: 'pipe',
      stderr: 'pipe'
    })
    const stdout = await new Response(proc.stdout).text()
    const exit = await proc.exited
    if (exit !== 0) return false
    gitDir = stdout.trim()
    if (!gitDir.startsWith('/')) gitDir = join(ROOT_DIR, gitDir)
  } catch {
    return false
  }
  for (const file of ['MERGE_HEAD', 'rebase-merge', 'rebase-apply']) {
    try {
      await stat(join(gitDir, file))
      return true
    } catch (error) {
      if (error.code !== 'ENOENT' && error.code !== 'ENOTDIR') throw error
    }
  }
  return false
}

/**
 * Compute the SHA-256 hash of a file's content (empty string if missing).
 * @param {string} rootDir Repository root directory.
 * @param {string} relPath File path relative to `rootDir`.
 * @returns {Promise<string>} Hash of the file content.
 */
async function fileHash(rootDir, relPath) {
  const content = await readFile(join(rootDir, relPath), 'utf8').catch(() => '')
  return sha256(content)
}

/**
 * Return today's date in ISO `YYYY-MM-DD` format.
 * @returns {string} Current date string.
 */
function isoDate() {
  return new Date().toISOString().slice(0, 10)
}

// Set the exit code and let the runtime exit naturally once the event loop drains.
process.exitCode = await main()
