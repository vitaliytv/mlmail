# Coverage Fix Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `## Recommendations` section to `COVERAGE.md` (with LLM-generated guidance per survived mutant) and a new `/n-coverage-fix` skill that iteratively writes tests until mutation score stops improving.

**Architecture:** The js-lint coverage provider (`rules/js-lint/coverage/coverage.mjs`) gains three new internal helpers — `parseSurvivedMutants`, `findExampleTest`, and an injectable LLM call in `lib/generate-recommendation.mjs`. The orchestrator's `renderMarkdown` gets a `recommendations` field in its row type. The new skill lives at `npm/skills/n-coverage-fix/SKILL.md`.

**Tech Stack:** Bun/Node.js, `@anthropic-ai/sdk@^0.98.0` (prompt caching with `ephemeral`), `bun:test`, existing `makeFixture` helper pattern.

---

## File Map

| File                                                                | Action | Responsibility                                                                             |
| ------------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------ |
| `npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs` | Modify | Add `incremental: true` + `incrementalFile`                                                |
| `npm/rules/js-lint/coverage/coverage.mjs`                           | Modify | Add `parseSurvivedMutants`, `findExampleTest`, `extractFirstTestBlock`; update `collect()` |
| `npm/rules/js-lint/coverage/lib/generate-recommendation.mjs`        | Create | LLM call via Anthropic SDK                                                                 |
| `npm/rules/js-lint/coverage/tests/coverage.test.mjs`                | Modify | Tests for new helpers and updated `collect()`                                              |
| `npm/rules/js-lint/coverage/tests/generate-recommendation.test.mjs` | Create | Tests for LLM function with mock client                                                    |
| `npm/rules/test/coverage/coverage.mjs`                              | Modify | Update `renderMarkdown` to render `## Recommendations`                                     |
| `npm/rules/test/coverage/tests/coverage.test.mjs`                   | Modify | Tests for Recommendations rendering                                                        |
| `npm/skills/n-coverage-fix/SKILL.md`                                | Create | New skill                                                                                  |
| `npm/skills/n-coverage-fix/auto.md`                                 | Create | Auto-activate condition                                                                    |
| `npm/package.json`                                                  | Modify | Add `@anthropic-ai/sdk`, bump version to `1.20.0`                                          |
| `npm/CHANGELOG.md`                                                  | Modify | Entry for `1.20.0`                                                                         |

The `mlmail/app/stryker.config.mjs` already has `incremental: true` — no change needed.

---

## Task 1: Stryker incremental mode in baseline

**Files:**

- Modify: `npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs`

This file is the canonical template copied to new projects via `n-cursor fix test`. Adding `incremental` here means all new JS projects get checkpoint-based Stryker runs. Note: existing projects won't auto-update (the copy only runs if the file doesn't exist yet).

- [ ] **Step 1: Add incremental to baseline**

Edit `npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs` — add two lines before `coverageAnalysis`:

```js
/** @type {import('@stryker-mutator/core').PartialStrykerOptions} */
export default {
  testRunner: 'command',
  commandRunner: { command: 'bun test' },
  // inPlace: уникає hoisted-node_modules issues у Bun monorepo (sandbox-копія втрачає resolution).
  // Також тести, що читають git/fs-state (integration checks), працюють тільки in-place.
  inPlace: true,
  tempDirName: 'reports/stryker/.tmp',
  reporters: ['json', 'clear-text'],
  jsonReporter: { fileName: 'reports/stryker/mutation.json' },
  // incremental: зберігає результати мутантів між запусками — після crash Stryker
  // відновлюється з місця зупинки замість повного рестарту.
  incremental: true,
  incrementalFile: 'reports/stryker/stryker-incremental.json',
  coverageAnalysis: 'off'
}
```

- [ ] **Step 2: Run stryker_config tests to verify**

```bash
bun test npm/rules/test/js/tests/stryker_config.test.mjs
```

Expected: all pass. If a snapshot test fails, update it.

---

## Task 2: `parseSurvivedMutants` function (TDD)

**Files:**

- Modify: `npm/rules/js-lint/coverage/coverage.mjs`
- Modify: `npm/rules/js-lint/coverage/tests/coverage.test.mjs`

This pure function extracts survived/no-coverage mutants from the already-parsed mutation.json object. It returns an array grouped by filename — no file I/O here (keeps it testable).

- [ ] **Step 1: Write failing test**

In `npm/rules/js-lint/coverage/tests/coverage.test.mjs`, add after the existing `describe('js-lint coverage collect()'...)`:

```js
describe('parseSurvivedMutants()', () => {
  test('returns empty array when all mutants killed', () => {
    const report = {
      files: {
        'src/a.js': {
          mutants: [
            {
              status: 'Killed',
              mutatorName: 'X',
              replacement: 'y',
              location: { start: { line: 1, column: 0 }, end: { line: 1, column: 5 } }
            }
          ]
        }
      }
    }
    expect(parseSurvivedMutants(report)).toEqual([])
  })

  test('groups survived mutants by filename with location info', () => {
    const report = {
      files: {
        'src/a.js': {
          mutants: [
            {
              status: 'Survived',
              mutatorName: 'BooleanLiteral',
              replacement: 'true',
              location: { start: { line: 4, column: 10 }, end: { line: 4, column: 15 } }
            },
            {
              status: 'Killed',
              mutatorName: 'X',
              replacement: 'y',
              location: { start: { line: 5, column: 0 }, end: { line: 5, column: 1 } }
            }
          ]
        },
        'src/b.js': {
          mutants: [
            {
              status: 'NoCoverage',
              mutatorName: 'ConditionalExpression',
              replacement: 'false',
              location: { start: { line: 10, column: 5 }, end: { line: 10, column: 20 } }
            }
          ]
        }
      }
    }
    expect(parseSurvivedMutants(report)).toEqual([
      {
        filename: 'src/a.js',
        mutants: [{ line: 4, type: 'BooleanLiteral', replacement: 'true' }]
      },
      {
        filename: 'src/b.js',
        mutants: [{ line: 10, type: 'ConditionalExpression', replacement: 'false' }]
      }
    ])
  })

  test('excludes CompileError and RuntimeError from survived list', () => {
    const report = {
      files: {
        'src/a.js': {
          mutants: [
            {
              status: 'CompileError',
              mutatorName: 'X',
              replacement: 'y',
              location: { start: { line: 1, column: 0 }, end: { line: 1, column: 1 } }
            },
            {
              status: 'RuntimeError',
              mutatorName: 'X',
              replacement: 'y',
              location: { start: { line: 2, column: 0 }, end: { line: 2, column: 1 } }
            }
          ]
        }
      }
    }
    expect(parseSurvivedMutants(report)).toEqual([])
  })
})
```

You'll need to import `parseSurvivedMutants` at the top of the test file — it's not exported yet, so this will fail at import time.

- [ ] **Step 2: Run to verify it fails**

```bash
bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | grep -E "fail|pass|error" | tail -5
```

Expected: import error or test failures.

- [ ] **Step 3: Implement `parseSurvivedMutants` in `coverage.mjs`**

Add this function after `parseStrykerReport` in `npm/rules/js-lint/coverage/coverage.mjs`:

```js
/**
 * Витягує мутантів що вижили зі Stryker mutation.json, групує по файлах.
 * @param {{files:Record<string,{mutants:Array<{status:string,mutatorName:string,replacement:string,location:{start:{line:number,column:number},end:{line:number,column:number}}}>}>}} report
 * @returns {Array<{filename:string, mutants:Array<{line:number,type:string,replacement:string}>}>}
 */
export function parseSurvivedMutants(report) {
  const result = []
  for (const [filename, data] of Object.entries(report.files)) {
    const survived = data.mutants.filter(m => m.status === 'Survived' || m.status === 'NoCoverage')
    if (survived.length === 0) continue
    result.push({
      filename,
      mutants: survived.map(m => ({
        line: m.location.start.line,
        type: m.mutatorName,
        replacement: m.replacement
      }))
    })
  }
  return result
}
```

Also add `parseSurvivedMutants` to the import in the test file.

- [ ] **Step 4: Run tests to verify pass**

```bash
bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | tail -5
```

Expected: all existing tests pass + 3 new `parseSurvivedMutants` tests pass.

---

## Task 3: `findExampleTest` + `extractFirstTestBlock` (TDD)

**Files:**

- Modify: `npm/rules/js-lint/coverage/coverage.mjs`
- Modify: `npm/rules/js-lint/coverage/tests/coverage.test.mjs`

Finds the test file for a given source file and extracts the first `it(` or `test(` block as a style example.

- [ ] **Step 1: Write failing tests**

Add to `npm/rules/js-lint/coverage/tests/coverage.test.mjs`:

```js
describe('extractFirstTestBlock()', () => {
  test('extracts first it() block', () => {
    const content = `import { test } from 'bun:test'

test('first', () => {
  expect(1).toBe(1)
})

test('second', () => {
  expect(2).toBe(2)
})`
    expect(extractFirstTestBlock(content)).toBe("test('first', () => {\n  expect(1).toBe(1)\n})")
  })

  test('extracts first it() block with nested braces', () => {
    const content = `it('works', async () => {
  const obj = { a: { b: 1 } }
  expect(obj.a.b).toBe(1)
})`
    expect(extractFirstTestBlock(content)).toBe(
      "it('works', async () => {\n  const obj = { a: { b: 1 } }\n  expect(obj.a.b).toBe(1)\n})"
    )
  })

  test('returns null when no test block found', () => {
    expect(extractFirstTestBlock('// just a comment\nconst x = 1')).toBeNull()
  })
})

describe('findExampleTest()', () => {
  test('finds .test.js file next to source and extracts first block', () => {
    const dir = mkdtempSync(join(tmpdir(), 'find-example-test-'))
    writeFileSync(join(dir, 'foo.js'), 'export const x = 1')
    writeFileSync(
      join(dir, 'foo.test.js'),
      `import { it, expect } from 'bun:test'
it('x is 1', () => {
  expect(1).toBe(1)
})`
    )
    const result = findExampleTest(dir, 'foo.js')
    expect(result).toEqual({
      testFile: 'foo.test.js',
      code: "it('x is 1', () => {\n  expect(1).toBe(1)\n})"
    })
    rmSync(dir, { recursive: true, force: true })
  })

  test('returns null when no test file found', () => {
    const dir = mkdtempSync(join(tmpdir(), 'find-example-test-empty-'))
    expect(findExampleTest(dir, 'src/foo.js')).toBeNull()
    rmSync(dir, { recursive: true, force: true })
  })
})
```

Add `extractFirstTestBlock, findExampleTest` to the import from `../coverage.mjs`.

- [ ] **Step 2: Run to verify fail**

```bash
bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | grep -E "fail|error" | tail -5
```

- [ ] **Step 3: Implement both functions in `coverage.mjs`**

Add after `parseSurvivedMutants` in `npm/rules/js-lint/coverage/coverage.mjs`:

```js
/**
 * Витягує перший `it(` або `test(` блок з вмісту тест-файлу.
 * Відстежує глибину дужок для коректного завершення.
 * @param {string} content вміст тест-файлу
 * @returns {string | null} перший тест-блок або null
 */
export function extractFirstTestBlock(content) {
  const lines = content.split('\n')
  let startLine = -1
  let depth = 0
  let inBlock = false
  const result = []
  for (let i = 0; i < lines.length; i++) {
    if (startLine === -1 && /^\s*(it|test)\(/.test(lines[i])) startLine = i
    if (startLine === -1) continue
    result.push(lines[i])
    for (const ch of lines[i]) {
      if (ch === '{') {
        depth++
        inBlock = true
      } else if (ch === '}') depth--
    }
    if (inBlock && depth === 0) break
  }
  return result.length > 0 ? result.join('\n') : null
}

/**
 * Шукає тест-файл для заданого source-файлу і повертає перший тест-блок як приклад стилю.
 * Варіанти: `<base>.test.js`, `<base>.test.mjs`, `<dir>/tests/<name>.test.js`.
 * @param {string} jsRoot абсолютний шлях до JS-кореня
 * @param {string} filename відносний шлях до source-файлу (від jsRoot)
 * @returns {{testFile:string, code:string|null} | null} null — якщо тест-файл не знайдено
 */
export function findExampleTest(jsRoot, filename) {
  const base = filename.replace(/\.[^.]+$/, '')
  const candidates = [`${base}.test.js`, `${base}.test.mjs`, `${base}.test.ts`]
  const lastSlash = base.lastIndexOf('/')
  if (lastSlash >= 0) {
    const dir = base.slice(0, lastSlash)
    const name = base.slice(lastSlash + 1)
    candidates.push(`${dir}/tests/${name}.test.js`, `${dir}/tests/${name}.test.mjs`)
  }
  for (const rel of candidates) {
    const full = join(jsRoot, rel)
    if (!existsSync(full)) continue
    const content = readFileSync(full, 'utf8')
    return { testFile: rel, code: extractFirstTestBlock(content) }
  }
  return null
}
```

Add `readFileSync` to the existing `import { existsSync } from 'node:fs'` import (change to `import { existsSync, readFileSync } from 'node:fs'`).

- [ ] **Step 4: Run tests to verify pass**

```bash
bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | tail -5
```

Expected: all tests pass including 5 new ones.

---

## Task 4: `generate-recommendation.mjs` LLM function (TDD)

**Files:**

- Create: `npm/rules/js-lint/coverage/lib/generate-recommendation.mjs`
- Create: `npm/rules/js-lint/coverage/tests/generate-recommendation.test.mjs`

This module wraps the Anthropic SDK call. It accepts an injectable `client` so tests don't need a real API key. The system prompt (which includes the source file content) is marked with `cache_control: ephemeral` so multiple mutants from the same file hit the cache.

First, add `@anthropic-ai/sdk` as a dependency:

- [ ] **Step 1: Add SDK dependency**

```bash
cd npm && bun add @anthropic-ai/sdk
```

Verify `npm/package.json` now lists `"@anthropic-ai/sdk": "^0.98.0"` under `dependencies`.

- [ ] **Step 2: Write failing test**

Create `npm/rules/js-lint/coverage/tests/generate-recommendation.test.mjs`:

```js
import { describe, expect, test } from 'bun:test'
import { createAnthropicClient, generateMutantRecommendation } from '../lib/generate-recommendation.mjs'

const mockClient = {
  messages: {
    create: async () => ({ content: [{ type: 'text', text: 'Тест повинен передати null замість рядка.' }] })
  }
}

describe('generateMutantRecommendation()', () => {
  test('returns LLM text for a mutant', async () => {
    const result = await generateMutantRecommendation(
      mockClient,
      'export function check(kind) {\n  if (kind !== null) return kind\n  return "default"\n}',
      { type: 'ConditionalExpression', replacement: 'false', line: 2 }
    )
    expect(typeof result).toBe('string')
    expect(result.length).toBeGreaterThan(5)
  })

  test('uses the source context and mutant type in the call', async () => {
    const calls = []
    const spyClient = {
      messages: {
        create: async params => {
          calls.push(params)
          return { content: [{ type: 'text', text: 'ok' }] }
        }
      }
    }
    await generateMutantRecommendation(spyClient, 'const x = 1', { type: 'StringLiteral', replacement: '""', line: 1 })
    expect(calls).toHaveLength(1)
    expect(calls[0].model).toContain('claude')
    const systemText = calls[0].system[0].text
    expect(systemText).toContain('const x = 1')
    expect(calls[0].messages[0].content).toContain('StringLiteral')
  })
})

describe('createAnthropicClient()', () => {
  test('returns null when ANTHROPIC_API_KEY is not set', () => {
    const saved = process.env.ANTHROPIC_API_KEY
    delete process.env.ANTHROPIC_API_KEY
    expect(createAnthropicClient()).toBeNull()
    if (saved !== undefined) process.env.ANTHROPIC_API_KEY = saved
  })
})
```

- [ ] **Step 3: Run to verify fail**

```bash
bun test npm/rules/js-lint/coverage/tests/generate-recommendation.test.mjs 2>&1 | tail -5
```

Expected: module not found error.

- [ ] **Step 4: Create `lib/generate-recommendation.mjs`**

Create directory and file:

```bash
mkdir -p npm/rules/js-lint/coverage/lib
```

Create `npm/rules/js-lint/coverage/lib/generate-recommendation.mjs`:

```js
import Anthropic from '@anthropic-ai/sdk'

/**
 * Генерує 2-3 речення рекомендації для вижилого мутанту через LLM.
 * System prompt кешується (ephemeral) — всі мутанти одного файлу
 * використовують один і той самий кешований контекст.
 * @param {Anthropic} client ін'єктований Anthropic client (для тестів — мок)
 * @param {string} sourceContent повний вміст source-файлу
 * @param {{type:string, replacement:string, line:number}} mutant деталі вижилого мутанту
 * @returns {Promise<string>} 2-3 речення українською
 */
export async function generateMutantRecommendation(client, sourceContent, mutant) {
  const response = await client.messages.create({
    model: 'claude-haiku-4-5-20251001',
    max_tokens: 200,
    system: [
      {
        type: 'text',
        text: `Ти — експерт з тестування. Проаналізуй вижилий мутант і дай 2-3 речення українською: що саме не перевіряють наявні тести і яке нове значення/стан треба передати у тест щоб виявити цю зміну.

Вміст файлу:
\`\`\`js
${sourceContent}
\`\`\``,
        cache_control: { type: 'ephemeral' }
      }
    ],
    messages: [
      {
        role: 'user',
        content: `Мутант типу "${mutant.type}" на рядку ${mutant.line} вижив. Оригінальний код замінено на: \`${mutant.replacement}\`. Тести не впіймали цю зміну.`
      }
    ]
  })
  return response.content[0].text.trim()
}

/**
 * Створює Anthropic client з env, або повертає null якщо ANTHROPIC_API_KEY не встановлено.
 * @returns {Anthropic | null}
 */
export function createAnthropicClient() {
  if (!process.env.ANTHROPIC_API_KEY) return null
  return new Anthropic()
}
```

- [ ] **Step 5: Run tests to verify pass**

```bash
bun test npm/rules/js-lint/coverage/tests/generate-recommendation.test.mjs 2>&1 | tail -5
```

Expected: 3 tests pass.

---

## Task 5: Wire recommendations into `collect()` (TDD)

**Files:**

- Modify: `npm/rules/js-lint/coverage/coverage.mjs`
- Modify: `npm/rules/js-lint/coverage/tests/coverage.test.mjs`

The `collect()` function gains an optional `opts.anthropicClient`. After Stryker runs and `parseSurvivedMutants` extracts survivors, for each file we: read source, find example test, optionally call LLM. Returns `recommendations` alongside existing metrics.

- [ ] **Step 1: Write failing test for collect() with recommendations**

Add to `describe('js-lint coverage collect()')` in the test file:

```js
test('collect() returns recommendations for survived mutants', async () => {
  const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })
  const reportDir = join(dir, 'reports', 'stryker')
  mkdirSync(reportDir, { recursive: true })

  // Source file with a survived mutant
  const srcDir = join(dir, 'src')
  mkdirSync(srcDir, { recursive: true })
  writeFileSync(
    join(srcDir, 'utils.js'),
    'export function check(x) {\n  if (x !== null) return x\n  return "default"\n}'
  )
  writeFileSync(
    join(srcDir, 'utils.test.js'),
    "import { it, expect } from 'bun:test'\nit('returns value', () => {\n  expect(check('a')).toBe('a')\n})"
  )

  writeFileSync(
    join(reportDir, 'mutation.json'),
    JSON.stringify({
      files: {
        'src/utils.js': {
          mutants: [
            {
              status: 'Survived',
              mutatorName: 'ConditionalExpression',
              replacement: 'false',
              location: { start: { line: 2, column: 2 }, end: { line: 2, column: 15 } }
            }
          ]
        }
      }
    })
  )

  const mockLcov = 'TN:\nSF:src/utils.js\nLF:3\nLH:2\nFNF:1\nFNH:1\nend_of_record\n'
  const mockRunner = {
    runJsCoverage: async ({ lcovDir }) => {
      writeFileSync(join(lcovDir, 'lcov.info'), mockLcov)
      return 0
    },
    runStryker: async () => 0
  }

  const mockLlmClient = {
    messages: { create: async () => ({ content: [{ type: 'text', text: 'Тест для null' }] }) }
  }

  const rows = await collect(dir, { runner: mockRunner, anthropicClient: mockLlmClient })

  expect(rows).toHaveLength(1)
  expect(rows[0].recommendations).toHaveLength(1)
  expect(rows[0].recommendations[0].filename).toBe('src/utils.js')
  expect(rows[0].recommendations[0].mutants).toHaveLength(1)
  expect(rows[0].recommendations[0].mutants[0].type).toBe('ConditionalExpression')
  expect(rows[0].recommendations[0].exampleTest).not.toBeNull()
  expect(rows[0].recommendations[0].exampleTest.testFile).toBe('src/utils.test.js')
  expect(rows[0].recommendations[0].recommendationText).toBe('Тест для null')

  rmSync(dir, { recursive: true, force: true })
})

test('collect() returns empty recommendations when all mutants killed', async () => {
  const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })
  const reportDir = join(dir, 'reports', 'stryker')
  mkdirSync(reportDir, { recursive: true })
  writeFileSync(
    join(reportDir, 'mutation.json'),
    JSON.stringify({
      files: {
        'src/a.js': {
          mutants: [
            {
              status: 'Killed',
              mutatorName: 'X',
              replacement: 'y',
              location: { start: { line: 1, column: 0 }, end: { line: 1, column: 1 } }
            }
          ]
        }
      }
    })
  )
  const mockLcov = 'TN:\nSF:src/a.js\nLF:1\nLH:1\nFNF:1\nFNH:1\nend_of_record\n'
  const mockRunner = {
    runJsCoverage: async ({ lcovDir }) => {
      writeFileSync(join(lcovDir, 'lcov.info'), mockLcov)
      return 0
    },
    runStryker: async () => 0
  }
  const rows = await collect(dir, { runner: mockRunner })
  expect(rows[0].recommendations).toEqual([])
})
```

Note: `mockRunner` functions here are `async` — update the runner signature in the implementation to support both sync and async runners.

- [ ] **Step 2: Run to verify fail**

```bash
bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | grep -E "fail|error" | tail -5
```

- [ ] **Step 3: Update `collect()` in `coverage.mjs`**

Add import at top of file:

```js
import { createAnthropicClient, generateMutantRecommendation } from './lib/generate-recommendation.mjs'
```

Update `defaultRunner` to have async-compatible interface (add `async` keyword to both methods):

```js
const defaultRunner = {
  async runJsCoverage({ cwd, lcovDir }) {
    const r = spawnSync('bun', ['run', 'test:coverage', '--coverage-reporter=lcov', `--coverage-dir=${lcovDir}`], {
      cwd,
      stdio: 'inherit',
      env: process.env
    })
    return r.status ?? 1
  },
  async runStryker({ cwd }) {
    const r = spawnSync('bunx', ['@stryker-mutator/core', 'run'], { cwd, stdio: 'inherit', env: process.env })
    return r.status ?? 1
  }
}
```

Replace the `collect()` function body — after `const mutation = parseStrykerReport(mutationReport)` and before `return`:

```js
// 3. Recommendations для вижилих мутантів
const survivedGroups = parseSurvivedMutants(mutationReport)
const anthropicClient = opts.anthropicClient ?? createAnthropicClient()
const recommendations = []
for (const group of survivedGroups) {
  const sourceFile = join(jsRoot, group.filename)
  const sourceContent = existsSync(sourceFile) ? await readFile(sourceFile, 'utf8') : ''
  const exampleTest = findExampleTest(jsRoot, group.filename)
  let recommendationText = null
  if (anthropicClient && sourceContent) {
    // Один LLM-call на файл (всі мутанти одного файлу описуються разом)
    const mutantSummary = group.mutants
      .map(m => `рядок ${m.line}: тип "${m.type}", замінено на \`${m.replacement}\``)
      .join('; ')
    recommendationText = await generateMutantRecommendation(anthropicClient, sourceContent, {
      type: group.mutants.map(m => m.type).join(', '),
      replacement: mutantSummary,
      line: group.mutants[0].line
    })
  }
  recommendations.push({ filename: group.filename, mutants: group.mutants, exampleTest, recommendationText })
}

return [{ area: 'JS', coverage, mutation, recommendations }]
```

Also update the function JSDoc to document the new `opts.anthropicClient` parameter and the `recommendations` field in the return type.

- [ ] **Step 4: Update existing collect() tests**

The existing test for `collect()` that checks `{ area: 'JS', coverage, mutation }` needs to be updated to also check `recommendations`:

Find the test `'парсить lcov + stryker mutation.json і повертає один CoverageRow'` and update the assertion:

```js
expect(rows[0]).toMatchObject({
  area: 'JS',
  coverage: { lines: { covered: 2, total: 3 }, functions: { covered: 1, total: 1 } },
  mutation: { caught: 2, total: 3 }
})
expect(Array.isArray(rows[0].recommendations)).toBe(true)
```

- [ ] **Step 5: Run all coverage tests**

```bash
bun test npm/rules/js-lint/coverage/tests/ 2>&1 | tail -8
```

Expected: all tests pass.

---

## Task 6: Update orchestrator `renderMarkdown` (TDD)

**Files:**

- Modify: `npm/rules/test/coverage/coverage.mjs`
- Modify: `npm/rules/test/coverage/tests/coverage.test.mjs`

`renderMarkdown(rows)` currently returns just the summary table. It now appends `## Recommendations` if any row has non-empty `recommendations`.

- [ ] **Step 1: Write failing test**

In `npm/rules/test/coverage/tests/coverage.test.mjs`, add to `describe('renderMarkdown')`:

```js
test('appends ## Recommendations section when recommendations present', () => {
  const rows = [
    {
      area: 'JS',
      coverage: { lines: { covered: 10, total: 20 }, functions: { covered: 3, total: 5 } },
      mutation: { caught: 7, total: 10 },
      recommendations: [
        {
          filename: 'src/auth.js',
          mutants: [{ line: 19, type: 'ConditionalExpression', replacement: 'false' }],
          exampleTest: { testFile: 'src/auth.test.js', code: "it('works', () => {\n  expect(1).toBe(1)\n})" },
          recommendationText: 'Передай null як аргумент і перевір результат.'
        }
      ]
    }
  ]
  const md = renderMarkdown(rows)
  expect(md).toContain('## Recommendations')
  expect(md).toContain('### src/auth.js')
  expect(md).toContain('**Вижило мутантів: 1**')
  expect(md).toContain('| 19 | ConditionalExpression | `false` |')
  expect(md).toContain('**Приклад наявного тесту**')
  expect(md).toContain("it('works'")
  expect(md).toContain('**Що треба протестувати:** Передай null як аргумент')
})

test('no ## Recommendations when all recommendations empty', () => {
  const rows = [
    {
      area: 'JS',
      coverage: { lines: { covered: 10, total: 10 }, functions: { covered: 3, total: 3 } },
      mutation: { caught: 10, total: 10 },
      recommendations: []
    }
  ]
  expect(renderMarkdown(rows)).not.toContain('## Recommendations')
})

test('renderMarkdown works with rows without recommendations field (backward compat)', () => {
  const rows = [
    {
      area: 'Rust',
      coverage: { lines: { covered: 100, total: 120 }, functions: { covered: 20, total: 25 } },
      mutation: { caught: 15, total: 15 }
    }
  ]
  expect(renderMarkdown(rows)).not.toContain('## Recommendations')
})
```

- [ ] **Step 2: Run to verify fail**

```bash
bun test npm/rules/test/coverage/tests/coverage.test.mjs 2>&1 | grep -E "fail|error" | tail -5
```

- [ ] **Step 3: Update `renderMarkdown` in orchestrator**

In `npm/rules/test/coverage/coverage.mjs`, replace `renderMarkdown`:

````js
export function renderMarkdown(rows) {
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

  const allRecs = rows.flatMap(r => r.recommendations ?? [])
  if (allRecs.length > 0) {
    lines.push(
      '',
      '## Recommendations',
      '',
      '> Автоматично генерується після кожного `bun coverage`. Використовується `/n-coverage-fix` для дописування тестів.',
      ''
    )
    for (const rec of allRecs) {
      lines.push(`### ${rec.filename}`, '')
      lines.push(`**Вижило мутантів: ${rec.mutants.length}**`, '')
      lines.push('| Рядок | Тип | Вижив |')
      lines.push('|---|---|---|')
      for (const m of rec.mutants) {
        lines.push(`| ${m.line} | ${m.type} | \`${m.replacement}\` |`)
      }
      lines.push('')
      if (rec.exampleTest?.code) {
        lines.push(`**Приклад наявного тесту** (з \`${rec.exampleTest.testFile}\`):`)
        lines.push('```js')
        lines.push(rec.exampleTest.code)
        lines.push('```', '')
      }
      if (rec.recommendationText) {
        lines.push(`**Що треба протестувати:** ${rec.recommendationText}`, '')
      }
    }
  }

  return `${lines.join('\n')}\n`
}
````

- [ ] **Step 4: Run all orchestrator tests**

```bash
bun test npm/rules/test/coverage/tests/coverage.test.mjs 2>&1 | tail -8
```

Expected: all pass.

- [ ] **Step 5: Run full test suite to verify no regressions**

```bash
bun test --parallel npm/rules/test/ npm/rules/js-lint/ npm/scripts/tests/ 2>&1 | tail -5
```

Expected: 0 fail.

---

## Task 7: New `/n-coverage-fix` skill

**Files:**

- Create: `npm/skills/n-coverage-fix/SKILL.md`
- Create: `npm/skills/n-coverage-fix/auto.md`

The skill file instructs Claude Code on how to execute the iterative coverage-fix loop. It references the `## Recommendations` section in `COVERAGE.md` as its input.

- [ ] **Step 1: Create auto.md**

Create `npm/skills/n-coverage-fix/auto.md`:

```
[js-lint, test]
```

This auto-activates the skill when both `js-lint` and `test` rules are present in `.n-cursor.json`.

- [ ] **Step 2: Create SKILL.md**

Create `npm/skills/n-coverage-fix/SKILL.md`:

````markdown
# n-coverage-fix

Iteratively improves mutation test coverage until score stops improving.

## When to use

When you want to improve the mutation score shown in `COVERAGE.md`, specifically targeting the **survived mutants** listed in `## Recommendations`.

## Algorithm

Follow these steps exactly in order:

### Step 1 — Run coverage

```bash
bun coverage
```

Wait for completion. This generates `COVERAGE.md` with a `## Recommendations` section.

### Step 2 — Check for recommendations

Read `COVERAGE.md`. If there is no `## Recommendations` section, or it has no `###` subsections → **DONE** (no survived mutants, score is already maximized).

Record the current mutation score from the summary table as `baseline_score`.

### Step 3 — Write tests for each file in Recommendations

For each `### <filename>` section in Recommendations:

1. Read the source file at `<filename>` (absolute path relative to your working directory)
2. Read the corresponding test file shown in "Приклад наявного тесту"
3. Study the example test to understand test style (imports, describe/it structure, assertion style)
4. For each row in the mutant table (Рядок / Тип / Вижив):
   - Look at the source at that line
   - Understand: the mutation replaced the original code with `Вижив` value — tests passed anyway, meaning no test exercises this path
   - Write a new `it()` or `test()` block in the same test file that would FAIL if `Вижив` were the real code
5. Add the new tests to the existing test file. Do NOT create a new test file.

Follow the "Що треба протестувати" guidance if present.

### Step 4 — Verify tests pass

```bash
bun test <test-file-path>
```

**If tests FAIL:**

- Do NOT revert your changes
- Report: which test failed, what the error is, which files were modified
- Show the current state of the test file
- Wait for user decision: fix manually → tell me to continue, or skip this file, or stop

**If tests PASS:** continue.

### Step 5 — Run coverage again

```bash
bun coverage
```

Wait for completion.

### Step 6 — Check for convergence

Read the new mutation score from `COVERAGE.md`.

- If new score > `baseline_score`: update `baseline_score = new score`, go to **Step 3** (next iteration)
- If new score ≤ `baseline_score`: **DONE** (convergence — cannot improve further)

**If `bun coverage` crashes:**
Note that `stryker.config.mjs` has `incremental: true` — re-running `bun coverage` will resume from the checkpoint. Tell the user and ask them to re-run or confirm continuation.

## Notes

- Work on one file at a time (all mutants in a file together)
- Never delete existing tests
- Never change source files — only test files
- If a new test requires a mock or fixture that doesn't exist, create it in the same test file or a `test/fixtures/` directory next to the test file
````

- [ ] **Step 3: Verify skill is discoverable**

```bash
cd npm && node bin/n-cursor.js skills list 2>&1 | grep coverage-fix || echo "skill not in list yet"
```

(If the command doesn't exist yet, skip — the skill file exists and will be picked up on next `n-cursor fix`.)

---

## Task 8: Version bump, CHANGELOG, and final check

**Files:**

- Modify: `npm/package.json`
- Modify: `npm/CHANGELOG.md`

- [ ] **Step 1: Bump version to 1.20.0**

In `npm/package.json`, change `"version": "1.19.2"` → `"version": "1.20.0"`.

- [ ] **Step 2: Add CHANGELOG entry**

In `npm/CHANGELOG.md`, add before the existing `## [1.19.2]` entry:

```markdown
## [1.20.0] - 2026-05-25

### Added

- **`COVERAGE.md` — секція `## Recommendations`**: після кожного `bun coverage` js-lint провайдер
  витягує вижилих мутантів зі Stryker `mutation.json`, знаходить приклад тесту з відповідного
  тест-файлу і (якщо встановлено `ANTHROPIC_API_KEY`) генерує рекомендації через
  `claude-haiku-4-5-20251001` з prompt caching. Секція служить вхідним промптом для
  нового скіла `/n-coverage-fix`.
- **Скіл `/n-coverage-fix`**: ітеративно дописує тести для вижилих мутантів до конвергенції
  (score не зростає). Checkpoint-based: помилка = пауза з показом стану, без відкату.
  Активується автоматично коли присутні правила `js-lint` + `test`.
- **`stryker.config.baseline.mjs`**: додано `incremental: true` + `incrementalFile` —
  Stryker зберігає прогрес між запусками, відновлюється після краш.

### Dependencies

- `@anthropic-ai/sdk@^0.98.0` — для LLM-call в recommendations generator.
```

- [ ] **Step 3: Run full test suite**

```bash
cd npm && bun test --parallel --timeout 15000 scripts/tests/ rules/ 2>&1 | tail -8
```

Expected: 0 fail.

- [ ] **Step 4: Update mlmail to 1.20.0**

In `mlmail/package.json`, update `"@nitra/cursor": "^1.19.2"` → `"@nitra/cursor": "^1.20.0"` and run:

```bash
bun install
```

(This step happens after `@nitra/cursor@1.20.0` is published.)
