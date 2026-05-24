import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT_DIR = dirname(dirname(fileURLToPath(import.meta.url)))
const APP_DIR = join(ROOT_DIR, 'app')
const OUTPUT_PATH = join(ROOT_DIR, 'COVERAGE.md')

// --- Coverage ---------------------------------------------------------------

/**
 * Parse an lcov report and aggregate line/function counts across all records.
 * @param {string} text Raw lcov.info content.
 * @returns {{lines: {covered: number, total: number}, functions: {covered: number, total: number}}} Aggregated counts.
 */
function parseLcov(text) {
  const acc = { lines: { covered: 0, total: 0 }, functions: { covered: 0, total: 0 } }
  for (const line of text.split('\n')) {
    if (line.startsWith('LF:')) acc.lines.total += Number(line.slice(3))
    else if (line.startsWith('LH:')) acc.lines.covered += Number(line.slice(3))
    else if (line.startsWith('FNF:')) acc.functions.total += Number(line.slice(4))
    else if (line.startsWith('FNH:')) acc.functions.covered += Number(line.slice(4))
  }
  return acc
}

/**
 * Run the app's JS test suite with lcov coverage and return aggregated totals.
 * @returns {Promise<{lines: {covered: number, total: number}, functions: {covered: number, total: number}}>} JS coverage totals.
 */
async function collectJsCoverage() {
  const dir = await mkdtemp(join(tmpdir(), 'mlmail-cov-'))
  try {
    const proc = Bun.spawn(['bun', 'run', 'test:coverage', '--coverage-reporter=lcov', `--coverage-dir=${dir}`], {
      cwd: APP_DIR,
      stdout: 'inherit',
      stderr: 'inherit'
    })
    const code = await proc.exited
    if (code !== 0) throw new Error(`JS coverage run failed (exit ${code})`)
    return parseLcov(await readFile(join(dir, 'lcov.info'), 'utf8'))
  } finally {
    await rm(dir, { recursive: true, force: true })
  }
}

/**
 * Run cargo-llvm-cov for the Tauri crate and return aggregated totals.
 * @returns {Promise<{lines: {covered: number, total: number}, functions: {covered: number, total: number}}>} Rust coverage totals.
 */
async function collectRustCoverage() {
  const proc = Bun.spawn(['cargo', 'llvm-cov', '--manifest-path', 'src-tauri/Cargo.toml', '--json', '--summary-only'], {
    cwd: APP_DIR,
    stdout: 'pipe',
    stderr: 'inherit'
  })
  const stdout = await new Response(proc.stdout).text()
  const code = await proc.exited
  if (code !== 0) {
    throw new Error('Rust coverage run failed — install cargo-llvm-cov: `cargo install cargo-llvm-cov`')
  }
  const totals = JSON.parse(stdout).data[0].totals
  return {
    lines: { covered: totals.lines.covered, total: totals.lines.count },
    functions: { covered: totals.functions.covered, total: totals.functions.count }
  }
}

// --- Mutation ---------------------------------------------------------------

/**
 * Run StrykerJS for the app and return killed/total mutant counts.
 * Killed + Timeout count as caught; compile/runtime errors are excluded.
 * @returns {Promise<{caught: number, total: number}>} JS mutation counts.
 */
async function collectJsMutation() {
  const proc = Bun.spawn(['bun', 'run', 'test:mutation'], { cwd: APP_DIR, stdout: 'inherit', stderr: 'inherit' })
  const code = await proc.exited

  let report
  try {
    report = JSON.parse(await readFile(join(APP_DIR, 'reports', 'stryker', 'mutation.json'), 'utf8'))
  } catch {
    throw new Error(`Stryker produced no mutation.json (exit ${code}) — check app/stryker.config.mjs`)
  }

  let caught = 0
  let total = 0
  for (const file of Object.values(report.files)) {
    for (const mutant of file.mutants) {
      if (mutant.status === 'Killed' || mutant.status === 'Timeout') {
        caught += 1
        total += 1
      } else if (mutant.status === 'Survived' || mutant.status === 'NoCoverage') {
        total += 1
      }
    }
  }
  return { caught, total }
}

/**
 * Run cargo-mutants (via the test:rust:mutation script) and return killed/total counts.
 * Caught + timeout count as caught; unviable (non-compiling) mutants are excluded.
 * @returns {Promise<{caught: number, total: number}>} Rust mutation counts.
 */
async function collectRustMutation() {
  // cargo-mutants exits non-zero when mutants are missed — that is the signal
  // we measure, not a failure; a missing outcomes.json means it actually crashed.
  const proc = Bun.spawn(['bun', 'run', 'test:rust:mutation'], {
    cwd: APP_DIR,
    stdout: 'inherit',
    stderr: 'inherit'
  })
  await proc.exited

  let outcomes
  try {
    outcomes = JSON.parse(await readFile('/tmp/cargo-mutants-out/mutants.out/outcomes.json', 'utf8'))
  } catch {
    throw new Error('cargo mutants produced no outcomes.json — run failed (install: `cargo install cargo-mutants`)')
  }

  const caught = outcomes.caught + outcomes.timeout
  return { caught, total: caught + outcomes.missed }
}

// --- Report -----------------------------------------------------------------

/**
 * Sum two coverage totals.
 * @param {object} a First totals.
 * @param {object} b Second totals.
 * @returns {{lines: {covered: number, total: number}, functions: {covered: number, total: number}}} Combined totals.
 */
function addCoverage(a, b) {
  return {
    lines: { covered: a.lines.covered + b.lines.covered, total: a.lines.total + b.lines.total },
    functions: { covered: a.functions.covered + b.functions.covered, total: a.functions.total + b.functions.total }
  }
}

/**
 * Sum two mutation counts.
 * @param {{caught: number, total: number}} a First counts.
 * @param {{caught: number, total: number}} b Second counts.
 * @returns {{caught: number, total: number}} Combined counts.
 */
function addMutation(a, b) {
  return { caught: a.caught + b.caught, total: a.total + b.total }
}

/**
 * Format a covered/total pair as `XX.XX% (covered/total)`.
 * @param {{covered: number, total: number}} metric Covered and total counts.
 * @returns {string} Coverage cell.
 */
function formatCoverage({ covered, total }) {
  const percent = total === 0 ? '—' : `${((covered / total) * 100).toFixed(2)}%`
  return `${percent} (${covered}/${total})`
}

/**
 * Format a mutation score as `XX.XX%`.
 * @param {{caught: number, total: number}} metric Caught and total mutant counts.
 * @returns {string} Score cell.
 */
function formatScore({ caught, total }) {
  return total === 0 ? '—' : `${((caught / total) * 100).toFixed(2)}%`
}

/**
 * Render the combined coverage + mutation report as one Markdown table.
 * No timestamp, so `git diff` moves only when the metrics actually change.
 * @param {Array<{area: string, coverage: object, mutation: {caught: number, total: number}}>} rows Table rows.
 * @returns {string} Markdown document.
 */
function renderMarkdown(rows) {
  const lines = [
    '# Coverage',
    '',
    '| Область | Рядки | Функції | Вбито мутацій | Score |',
    '| --- | --- | --- | --- | --- |'
  ]
  for (const row of rows) {
    lines.push(
      `| ${row.area} | ${formatCoverage(row.coverage.lines)} | ${formatCoverage(row.coverage.functions)} | ` +
        `${row.mutation.caught}/${row.mutation.total} | ${formatScore(row.mutation)} |`
    )
  }
  return `${lines.join('\n')}\n`
}

/**
 * Collect coverage and mutation metrics for JS and Rust, then write COVERAGE.md.
 * Parallel-run protection is provided by scripts/with-lock.js (see package.json).
 * @returns {Promise<number>} Process exit code.
 */
async function main() {
  // Clear stale Stryker temp left by any interrupted previous run.
  await rm(join(APP_DIR, 'reports', 'stryker', '.tmp'), { recursive: true, force: true })

  console.log('→ JS coverage (bun test)…')
  const jsCoverage = await collectJsCoverage()
  console.log('→ Rust coverage (cargo llvm-cov)…')
  const rustCoverage = await collectRustCoverage()
  console.log('→ JS mutation (stryker)…')
  const jsMutation = await collectJsMutation()

  // Remove stale cargo-mutants output just before running to avoid lock conflicts.
  await rm('/tmp/cargo-mutants-out', { recursive: true, force: true })
  console.log('→ Rust mutation (cargo mutants)…')
  const rustMutation = await collectRustMutation()

  const markdown = renderMarkdown([
    { area: 'JS (app)', coverage: jsCoverage, mutation: jsMutation },
    { area: 'Rust (src-tauri)', coverage: rustCoverage, mutation: rustMutation },
    {
      area: '**Разом**',
      coverage: addCoverage(jsCoverage, rustCoverage),
      mutation: addMutation(jsMutation, rustMutation)
    }
  ])
  await writeFile(OUTPUT_PATH, markdown, 'utf8')
  console.log(`\n${markdown}✓ COVERAGE.md`)
  return 0
}

try {
  process.exitCode = await main()
} catch (error) {
  console.error(`✗ ${error.message}`)
  process.exitCode = 1
}
