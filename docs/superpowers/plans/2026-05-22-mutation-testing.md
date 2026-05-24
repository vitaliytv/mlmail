# Mutation Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand `bun run coverage` so it runs coverage AND mutation testing sequentially and writes both results to `COVERAGE.md` in two sections.

**Architecture:** `scripts/coverage.js` gains four new async functions (`collectJsMutation`, `collectRustMutation`) plus an updated `renderMarkdown()` and `main()`. Single command (`bun run coverage`), single output file. COVERAGE.md grows a `## Мутаційне тестування` section replaced on each run — no timestamp, git-diff-friendly.

**Tech Stack:** `cargo-mutants` (global cargo tool), `@stryker-mutator/core ^8` (devDep), existing `scripts/coverage.js` Bun-spawn pattern, new `app/stryker.config.json`.

---

## File Map

| File                      | Action      | Responsibility                                                                                             |
| ------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------- |
| `scripts/coverage.js`     | Modify      | Add JS/Rust mutation collectors + update `renderMarkdown` + update `main()`                                |
| `app/package.json`        | Modify      | Add `@stryker-mutator/core` devDep; add `test:mutation` + `test:rust:mutation` scripts; bump `0.1.2→0.1.3` |
| `app/stryker.config.json` | Create      | Stryker: command-runner on `bun test`, JSON reporter, `inPlace: true`, mutate `src/**/*.{js,vue}`          |
| `app/CHANGELOG.md`        | Modify      | Add `[0.1.3]` entry                                                                                        |
| `scripts/package.json`    | Modify      | Bump `0.1.1→0.1.2`                                                                                         |
| `scripts/CHANGELOG.md`    | Modify      | Add `[0.1.2]` entry                                                                                        |
| `.gitignore`              | Modify      | Ignore `app/reports/` and `app/src-tauri/mutants.out/`                                                     |
| `COVERAGE.md`             | Regenerated | Gets `## Мутаційне тестування` section on first `bun run coverage`                                         |

---

## Task 1: Install cargo-mutants (global)

**Files:** n/a (system-wide)

- [ ] **Step 1: Check if already installed**

```bash
cargo mutants --version 2>/dev/null && echo "already installed" || echo "needs install"
```

- [ ] **Step 2: Install if missing**

```bash
cargo install cargo-mutants
```

Expected last line: `Installed package 'cargo-mutants v...'`

- [ ] **Step 3: Verify**

```bash
cargo mutants --version
```

Expected: `cargo-mutants X.Y.Z`

---

## Task 2: Add Stryker and config to app

**Files:**

- Modify: `app/package.json`
- Create: `app/stryker.config.json`

- [ ] **Step 1: Add devDep and scripts to app/package.json**

In `app/package.json` `"devDependencies"`, add:

```json
"@stryker-mutator/core": "^8.0.0",
```

In `app/package.json` `"scripts"`, after `test:rust:coverage`:

```json
"test:mutation": "bunx stryker run",
"test:rust:mutation": "cargo mutants --manifest-path src-tauri/Cargo.toml",
```

- [ ] **Step 2: Create app/stryker.config.json**

```json
{
  "testRunner": "command",
  "commandRunner": {
    "command": "bun test --preload ./test/happy-dom.preload.js src"
  },
  "inPlace": true,
  "coverageAnalysis": "off",
  "mutate": ["src/**/*.{js,vue}", "!src/**/*.test.js", "!src/test-utils/**", "!src/main.js"],
  "reporters": ["progress", "json"],
  "jsonReporter": {
    "fileName": "reports/mutation/mutation.json"
  },
  "allowConsoleColors": true,
  "timeoutMS": 15000
}
```

Notes:

- `inPlace: true` — required because hoisted `node_modules` (monorepo root) is unreachable in Stryker's default sandbox copy
- `coverageAnalysis: "off"` — `bun test` emits no Istanbul instrumentation; without this Stryker warns and falls back anyway
- `reporters: ["progress", "json"]` — `json` writes `app/reports/mutation/mutation.json`; `progress` shows terminal progress
- Stryker v8 mutates `<script>` blocks inside `.vue` SFCs natively; no extra plugin needed

- [ ] **Step 3: Install dependency**

```bash
bun i
```

Expected: lock file updated with `@stryker-mutator/core`.

- [ ] **Step 4: Smoke test Stryker finds mutants**

```bash
bun --cwd=app run test:mutation -- --dryRunTimeoutMs=10000 2>&1 | grep -i "mutant"
```

Expected: line mentioning mutants found, e.g. `Initial test run succeeded, N mutant(s) found`. No actual mutations run.

---

## Task 3: Update .gitignore

**Files:**

- Modify: `.gitignore` (root)

- [ ] **Step 1: Append mutation artifact entries**

Add at the end of `.gitignore`:

```
# Mutation testing artifacts (COVERAGE.md holds the summary)
app/reports/
app/src-tauri/mutants.out/
```

`app/reports/mutation/mutation.json` is a per-run temp artifact, NOT committed. `COVERAGE.md` at repo root is the committed summary.

---

## Task 4: Expand scripts/coverage.js

**Files:**

- Modify: `scripts/coverage.js`

Read the current file. Then apply these changes.

- [ ] **Step 1: Add collectJsMutation() after collectRustCoverage()**

```js
/**
 * Run StrykerJS and return killed/total mutant counts.
 * Reads app/reports/mutation/mutation.json produced by the JSON reporter.
 * @returns {Promise<{killed: number, total: number}>}
 */
async function collectJsMutation() {
  const proc = Bun.spawn(['bun', 'run', 'test:mutation'], {
    cwd: APP_DIR,
    stdout: 'inherit',
    stderr: 'inherit'
  })
  const code = await proc.exited
  if (code !== 0) throw new Error(`Stryker run failed (exit ${code}) — ensure @stryker-mutator/core is installed`)

  const reportPath = join(APP_DIR, 'reports', 'mutation', 'mutation.json')
  const report = JSON.parse(await readFile(reportPath, 'utf8'))
  const mutants = Object.values(report.files).flatMap(f => f.mutants)
  const killed = mutants.filter(m => m.status === 'Killed').length + mutants.filter(m => m.status === 'Timeout').length
  const noCoverage = mutants.filter(m => m.status === 'NoCoverage').length
  return { killed, total: mutants.length - noCoverage }
}
```

- [ ] **Step 2: Add collectRustMutation() after collectJsMutation()**

```js
/**
 * Run cargo-mutants and return caught/total counts parsed from stdout summary.
 * Exit 0 = all caught; exit 3 = some missed — both are valid outcomes.
 * @returns {Promise<{killed: number, total: number}>}
 */
async function collectRustMutation() {
  const proc = Bun.spawn(['cargo', 'mutants', '--manifest-path', 'src-tauri/Cargo.toml'], {
    cwd: APP_DIR,
    stdout: 'pipe',
    stderr: 'inherit'
  })
  const stdout = await new Response(proc.stdout).text()
  const code = await proc.exited
  if (code !== 0 && code !== 3)
    throw new Error(`cargo-mutants failed (exit ${code}) — install: cargo install cargo-mutants`)

  // Summary line: "49 mutants tested: 43 caught, 0 missed, 6 unviable"
  const caught = Number(stdout.match(/(\d+) caught/)?.[1] ?? 0)
  const missed = Number(stdout.match(/(\d+) missed/)?.[1] ?? 0)
  return { killed: caught, total: caught + missed }
}
```

- [ ] **Step 3: Replace renderMarkdown() with a version that includes both sections**

Replace the existing `renderMarkdown` function entirely:

```js
/**
 * Render coverage + mutation report as Markdown (no timestamp).
 * @param {object} js JS coverage totals.
 * @param {object} rust Rust coverage totals.
 * @param {object} total Combined coverage totals.
 * @param {{killed: number, total: number}} jsMut JS mutation counts.
 * @param {{killed: number, total: number}} rustMut Rust mutation counts.
 * @returns {string} Markdown document.
 */
function renderMarkdown(js, rust, total, jsMut, rustMut) {
  const totalMut = { killed: jsMut.killed + rustMut.killed, total: jsMut.total + rustMut.total }
  const scoreCell = ({ killed, total: t }) => (t === 0 ? '—' : `${((killed / t) * 100).toFixed(1)}% (${killed}/${t})`)
  return (
    [
      '# Coverage',
      '',
      '## Покриття коду',
      '',
      '| Область | Рядки | Функції |',
      '| --- | --- | --- |',
      `| JS (app) | ${formatMetric(js.lines)} | ${formatMetric(js.functions)} |`,
      `| Rust (src-tauri) | ${formatMetric(rust.lines)} | ${formatMetric(rust.functions)} |`,
      `| **Разом** | ${formatMetric(total.lines)} | ${formatMetric(total.functions)} |`,
      '',
      '## Мутаційне тестування',
      '',
      '| Область | Score |',
      '| --- | --- |',
      `| JS (app) | ${scoreCell(jsMut)} |`,
      `| Rust (src-tauri) | ${scoreCell(rustMut)} |`,
      `| **Разом** | ${scoreCell(totalMut)} |`
    ].join('\n') + '\n'
  )
}
```

- [ ] **Step 4: Replace main() to call mutation collectors**

Replace the existing `main()` function:

```js
/**
 * Collect JS and Rust coverage + mutation scores, write COVERAGE.md, print report.
 * @returns {Promise<number>} Process exit code.
 */
async function main() {
  console.log('→ JS coverage (bun test)…')
  const js = await collectJsCoverage()
  console.log('→ Rust coverage (cargo llvm-cov)…')
  const rust = await collectRustCoverage()
  console.log('→ JS mutation (stryker)…')
  const jsMut = await collectJsMutation()
  console.log('→ Rust mutation (cargo mutants)…')
  const rustMut = await collectRustMutation()

  const markdown = renderMarkdown(js, rust, add(js, rust), jsMut, rustMut)
  await writeFile(OUTPUT_PATH, markdown, 'utf8')

  console.log(`\n${markdown}✓ COVERAGE.md`)
  return 0
}
```

- [ ] **Step 5: Verify syntax is valid**

```bash
node --input-type=module < scripts/coverage.js 2>&1 | grep -i "syntaxerror" || echo "syntax OK"
```

Expected: `syntax OK`

---

## Task 5: Version bumps and changelogs

**Files:**

- `app/package.json` — bump `0.1.2 → 0.1.3`
- `app/CHANGELOG.md` — add `[0.1.3]` section
- `scripts/package.json` — bump `0.1.1 → 0.1.2`
- `scripts/CHANGELOG.md` — add `[0.1.2]` section

- [ ] **Step 1: Bump app/package.json version**

Change `"version": "0.1.2"` to `"version": "0.1.3"`.

- [ ] **Step 2: Prepend to app/CHANGELOG.md (before existing `## [0.1.2]`)**

```markdown
## [0.1.3] - 2026-05-22

### Added

- Скрипти `test:mutation` (StrykerJS) та `test:rust:mutation` (`cargo mutants`) для мутаційного тестування.
- `app/stryker.config.json` — конфіг StrykerJS: command-runner на `bun test`, `inPlace: true`, мутує `src/**/*.{js,vue}`.
```

- [ ] **Step 3: Bump scripts/package.json version**

Change `"version": "0.1.1"` to `"version": "0.1.2"`.

- [ ] **Step 4: Prepend to scripts/CHANGELOG.md (before existing `## [0.1.1]`)**

```markdown
## [0.1.2] - 2026-05-22

### Changed

- `coverage.js` розширено: після покриття запускає мутаційне тестування (StrykerJS + cargo-mutants) і записує `## Мутаційне тестування` до `COVERAGE.md`.
```

---

## Task 6: Format + rules check

- [ ] **Step 1: Format all files**

```bash
oxfmt .
```

Expected: `Finished in Xs on N files`

- [ ] **Step 2: Rules check**

```bash
npx @nitra/cursor check
```

Expected: `✨ Результат: 12/12 правил без зауважень`

If `❌ app: version не підвищено` fires against a higher base (parallel commits bumped main), re-bump `app/package.json` to the next patch and add a new CHANGELOG section.

---

## Task 7: End-to-end run (slow — 5–30 min)

**Timing estimate:** cargo-mutants tests ~70 Rust mutants × build+test time. StrykerJS tests ~10 JS mutants × `bun test` time (~1–2 s each). Total: typically 10–30 min on first run.

- [ ] **Step 1: Run full suite**

```bash
bun run coverage 2>&1 | tee /tmp/coverage-run.log | tail -25
```

Expected final section:

```
## Мутаційне тестування

| Область | Score |
| --- | --- |
| JS (app) | XX.X% (N/M) |
| Rust (src-tauri) | XX.X% (N/M) |
| **Разом** | XX.X% (N/M) |
✓ COVERAGE.md
```

If `collectRustMutation` throws a parsing error (stdout format differs from expected regex), inspect `/tmp/coverage-run.log` for the actual cargo-mutants summary line and adjust the regexes in `collectRustMutation()` to match.

- [ ] **Step 2: Verify COVERAGE.md structure**

```bash
grep "^## " COVERAGE.md
```

Expected:

```
## Покриття коду
## Мутаційне тестування
```

- [ ] **Step 3: Verify determinism**

```bash
bun run coverage >/dev/null 2>&1 && git diff COVERAGE.md
```

Expected: empty output — no diff on identical test results.

---

## Self-Review

- [x] Single command: `bun run coverage` — Task 4 wires mutation into `main()`
- [x] No timestamp in output — `renderMarkdown` has no date/time
- [x] Two COVERAGE.md sections: `## Покриття коду` + `## Мутаційне тестування`
- [x] `.vue` mutation: `mutate` glob includes `**/*.vue`; Stryker v8 mutates `<script>` blocks natively
- [x] `inPlace: true`: required for hoisted monorepo `node_modules`
- [x] `app/` and `scripts/` version bumps with CHANGELOG entries (changelog rule)
- [x] `.gitignore` covers `app/reports/` and `app/src-tauri/mutants.out/`
- [x] cargo-mutants exit code 3 (mutants survived) treated as valid run, not error
- [x] No unit tests for coverage.js per user decision ("прибрати з обсягу")
