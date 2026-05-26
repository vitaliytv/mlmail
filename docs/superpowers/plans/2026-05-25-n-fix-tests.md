# n-fix-tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stabilize Stryker coverage runs (incremental mode), add a survived-mutant recommendations section to COVERAGE.md, and create a `/n-fix-tests` skill that iteratively writes tests until mutation score converges.

**Architecture:** Three independent components: (A) mlmail gets `incremental: true` in `stryker.config.mjs` so killed processes resume without losing progress; (B) `@nitra/cursor` js-lint provider extracts survived mutants and the orchestrator appends a structured `## Вижилі мутанти` section to COVERAGE.md; (C) a Claude skill `.cursor/skills/n-fix-tests/SKILL.md` reads `mutation.json`, writes targeted tests, and loops `bun test → bun run coverage` until the score converges or reaches a target.

**Tech Stack:** Bun 1.3, @stryker-mutator/core, Node ESM (`node:fs/promises`), Bun test (`bun:test`), Claude skill SKILL.md

---

## Part A — mlmail: Stryker incremental mode

**Files:**
- Modify: `mlmail/app/stryker.config.mjs`
- Modify: `mlmail/.gitignore`

---

### Task A1: Add `incremental: true` to stryker config

All steps run from `mlmail/` root.

- [ ] **Step 1: Write failing check (verify incremental is missing)**

```bash
grep "incremental" app/stryker.config.mjs && echo "FOUND" || echo "NOT FOUND"
```
Expected output: `NOT FOUND`

- [ ] **Step 2: Add incremental config to stryker.config.mjs**

Replace `app/stryker.config.mjs` — add two lines after the `coverageAnalysis` line:

```js
/** @type {import('@stryker-mutator/core').PartialStrykerOptions} */
// @ts-nocheck

export default {
  testRunner: 'command',
  commandRunner: {
    command: 'bun test --timeout 15000 --preload ./test/happy-dom.preload.js src'
  },
  // inPlace avoids hoisted-node_modules issues in a Bun monorepo sandbox
  inPlace: true,
  mutate: [
    'src/services/auth-store.js',
    'src/i18n/auth-errors.js',
    'src/views/*.vue',
    'src/App.vue',
    '!src/**/*.test.js'
  ],
  // concurrency:1 prevents parallel workers from competing over inPlace source files
  concurrency: 1,
  // 60 s per mutant — bun test + happy-dom SFC compilation needs headroom
  timeoutMS: 60000,
  reporters: ['json', 'clear-text'],
  jsonReporter: { fileName: 'reports/stryker/mutation.json' },
  tempDirName: 'reports/stryker/.tmp',
  coverageAnalysis: 'off',
  // incremental: saves mutant status between runs; if process is killed, next run
  // re-tests only untested/survived mutants instead of starting from scratch
  incremental: true,
  incrementalFile: 'reports/stryker/incremental.json'
}
```

- [ ] **Step 3: Add incremental.json to .gitignore**

Check `mlmail/.gitignore` for existing stryker entries:

```bash
grep -n "stryker\|reports" .gitignore
```

Add after the existing stryker entries (already present: `**/reports/stryker/`):

```
# Stryker incremental cache (regenerated on each coverage run)
reports/stryker/incremental.json
```

Note: If `.gitignore` already has `**/reports/stryker/` (which it does), the incremental.json is already covered. Skip this step in that case.

- [ ] **Step 4: Verify config is valid**

```bash
cd app && node -e "import('./stryker.config.mjs').then(m => console.log('incremental:', m.default.incremental))"
```

Expected output: `incremental: true`

---

## Part B — @nitra/cursor: survived mutants in collect() and COVERAGE.md

Both tasks B1 and B2 are in the `cursor/npm/` workspace.

**Files:**
- Modify: `cursor/npm/rules/js-lint/coverage/coverage.mjs`
- Modify: `cursor/npm/rules/js-lint/coverage/tests/coverage.test.mjs`
- Modify: `cursor/npm/rules/test/coverage/coverage.mjs`
- Modify: `cursor/npm/rules/test/coverage/tests/coverage.test.mjs`

---

### Task B1: parseStrykerReport extracts survived mutants

Working directory: `cursor/npm/`

- [ ] **Step 1: Write the failing test**

In `rules/js-lint/coverage/tests/coverage.test.mjs`, replace the existing `collect()` test with a new one that includes survived mutants. **Replace the entire `describe('js-lint coverage collect()', ...)` block** with:

```js
describe('js-lint coverage collect()', () => {
  test('парсить lcov + stryker mutation.json і повертає один CoverageRow з survived', async () => {
    const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })

    const reportDir = join(dir, 'reports', 'stryker')
    mkdirSync(reportDir, { recursive: true })
    writeFileSync(
      join(reportDir, 'mutation.json'),
      JSON.stringify({
        files: {
          'src/a.js': {
            mutants: [
              { id: '0', status: 'Killed', mutatorName: 'StringLiteral', replacement: '""', location: { start: { line: 1, column: 0 }, end: { line: 1, column: 5 } } },
              { id: '1', status: 'Killed', mutatorName: 'BooleanLiteral', replacement: 'true', location: { start: { line: 2, column: 0 }, end: { line: 2, column: 5 } } },
              { id: '2', status: 'Survived', mutatorName: 'ConditionalExpression', replacement: 'false', location: { start: { line: 5, column: 3 }, end: { line: 5, column: 20 } } },
              { id: '3', status: 'CompileError', mutatorName: 'BlockStatement', replacement: '{}', location: { start: { line: 8, column: 0 }, end: { line: 8, column: 5 } } }
            ]
          }
        }
      })
    )

    const calls = []
    const runner = {
      runJsCoverage({ cwd, lcovDir }) {
        calls.push({ kind: 'js', cwd, lcovDir })
        writeFileSync(join(lcovDir, 'lcov.info'), ['LF:100', 'LH:50', 'FNF:20', 'FNH:10', ''].join('\n'))
        return 0
      },
      runStryker({ cwd }) {
        calls.push({ kind: 'stryker', cwd })
        return 0
      }
    }

    const rows = await collect(dir, { runner })
    expect(rows).toEqual([
      {
        area: 'JS',
        coverage: { lines: { covered: 50, total: 100 }, functions: { covered: 10, total: 20 } },
        mutation: { caught: 2, total: 3 },
        survived: [
          { file: 'src/a.js', line: 5, type: 'ConditionalExpression', replacement: 'false' }
        ]
      }
    ])
    expect(calls[0].kind).toBe('js')
    expect(calls[1].kind).toBe('stryker')

    rmSync(dir, { recursive: true, force: true })
  })

  test('повертає survived: [] коли всі мутанти вбиті', async () => {
    const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })

    const reportDir = join(dir, 'reports', 'stryker')
    mkdirSync(reportDir, { recursive: true })
    writeFileSync(
      join(reportDir, 'mutation.json'),
      JSON.stringify({
        files: {
          'src/a.js': {
            mutants: [
              { id: '0', status: 'Killed', mutatorName: 'StringLiteral', replacement: '""', location: { start: { line: 1, column: 0 }, end: { line: 1, column: 5 } } }
            ]
          }
        }
      })
    )

    const runner = {
      runJsCoverage({ lcovDir }) {
        writeFileSync(join(lcovDir, 'lcov.info'), 'LF:10\nLH:10\nFNF:5\nFNH:5\n')
        return 0
      },
      runStryker() { return 0 }
    }

    const rows = await collect(dir, { runner })
    expect(rows[0].survived).toEqual([])

    rmSync(dir, { recursive: true, force: true })
  })

  test('падає з explainer-ом якщо JS-coverage exit ≠ 0', async () => {
    const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })
    const runner = {
      runJsCoverage() { return 1 },
      runStryker() { return 0 }
    }
    await expect(collect(dir, { runner })).rejects.toThrow(JS_COVERAGE_EXIT_RE)
    rmSync(dir, { recursive: true, force: true })
  })

  test('падає якщо Stryker не залишив mutation.json', async () => {
    const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })
    const runner = {
      runJsCoverage({ lcovDir }) {
        writeFileSync(join(lcovDir, 'lcov.info'), 'LF:0\nLH:0\nFNF:0\nFNH:0\n')
        return 0
      },
      runStryker() { return 0 }
    }
    await expect(collect(dir, { runner })).rejects.toThrow(MUTATION_JSON_RE)
    rmSync(dir, { recursive: true, force: true })
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

```bash
bun test rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | tail -10
```

Expected: FAIL — `Property "survived" is missing`.

- [ ] **Step 3: Update parseStrykerReport to extract survived mutants**

In `rules/js-lint/coverage/coverage.mjs`, replace the `parseStrykerReport` function:

```js
/**
 * Парс Stryker mutation.json: Killed+Timeout → caught; Survived+NoCoverage → до total.
 * Compile/Runtime errors виключаються з total.
 * @param {{files:Record<string,{mutants:Array<{status:string, mutatorName?:string, replacement?:string, location?:{start:{line:number}}}>}>}} report розпарсений mutation.json
 * @returns {{caught:number, total:number, survived:Array<{file:string, line:number, type:string, replacement:string}>}} mutation score і вижилі мутанти
 */
function parseStrykerReport(report) {
  let caught = 0
  let total = 0
  const survived = []
  for (const [filePath, file] of Object.entries(report.files)) {
    for (const mutant of file.mutants) {
      if (mutant.status === 'Killed' || mutant.status === 'Timeout') {
        caught += 1
        total += 1
      } else if (mutant.status === 'Survived' || mutant.status === 'NoCoverage') {
        total += 1
        if (mutant.status === 'Survived') {
          survived.push({
            file: filePath,
            line: mutant.location?.start?.line ?? 0,
            type: mutant.mutatorName ?? 'Unknown',
            replacement: mutant.replacement ?? ''
          })
        }
      }
    }
  }
  return { caught, total, survived }
}
```

- [ ] **Step 4: Update collect() to include survived in returned row**

In `rules/js-lint/coverage/coverage.mjs`, replace the last two lines of `collect()`:

```js
  // Before (last 2 lines of collect):
  const mutation = parseStrykerReport(mutationReport)
  return [{ area: 'JS', coverage, mutation }]
```

```js
  // After:
  const { caught, total, survived } = parseStrykerReport(mutationReport)
  return [{ area: 'JS', coverage, mutation: { caught, total }, survived }]
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
bun test rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | tail -5
```

Expected: `5 pass, 0 fail`

---

### Task B2: renderMarkdown appends recommendations section

Working directory: `cursor/npm/`

- [ ] **Step 1: Write the failing test for renderMarkdown with survived**

In `rules/test/coverage/tests/coverage.test.mjs`, **add** a new test block after the existing `describe('renderMarkdown', ...)`:

```js
describe('renderMarkdown з survived мутантами', () => {
  test('додає секцію рекомендацій коли є вижилі мутанти', () => {
    const rows = [
      {
        area: 'JS',
        coverage: { lines: { covered: 50, total: 100 }, functions: { covered: 10, total: 20 } },
        mutation: { caught: 7, total: 10 },
        survived: [
          { file: 'src/a.js', line: 5, type: 'ConditionalExpression', replacement: 'false' },
          { file: 'src/a.js', line: 5, type: 'LogicalOperator', replacement: 'x && y' },
          { file: 'src/b.js', line: 12, type: 'BooleanLiteral', replacement: 'true' }
        ]
      }
    ]
    const md = renderMarkdown(rows)
    expect(md).toContain('## Вижилі мутанти')
    expect(md).toContain('### `src/a.js`')
    expect(md).toContain('**Рядок 5**')
    expect(md).toContain('| `false` | ConditionalExpression |')
    expect(md).toContain('| `x && y` | LogicalOperator |')
    expect(md).toContain('### `src/b.js`')
    expect(md).toContain('**Рядок 12**')
    expect(md).toContain('| `true` | BooleanLiteral |')
    expect(md.endsWith('\n')).toBe(true)
  })

  test('не додає секцію рекомендацій коли survived порожній', () => {
    const rows = [
      {
        area: 'JS',
        coverage: { lines: { covered: 100, total: 100 }, functions: { covered: 10, total: 10 } },
        mutation: { caught: 10, total: 10 },
        survived: []
      }
    ]
    const md = renderMarkdown(rows)
    expect(md).not.toContain('## Вижилі мутанти')
  })

  test('не додає секцію рекомендацій коли survived відсутній у rows', () => {
    const rows = [
      {
        area: 'Rust',
        coverage: { lines: { covered: 100, total: 100 }, functions: { covered: 10, total: 10 } },
        mutation: { caught: 10, total: 10 }
      }
    ]
    const md = renderMarkdown(rows)
    expect(md).not.toContain('## Вижилі мутанти')
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

```bash
bun test rules/test/coverage/tests/coverage.test.mjs -t "survived" 2>&1 | tail -10
```

Expected: FAIL — `## Вижилі мутанти` not found in output.

- [ ] **Step 3: Update renderMarkdown in orchestrator**

In `rules/test/coverage/coverage.mjs`, replace the `renderMarkdown` function:

```js
/**
 * Рендерить таблицю покриття + мутаційного тестування як Markdown.
 * Якщо будь-який row містить survived[], додає секцію вижилих мутантів.
 * @param {Array<{area:string, coverage:{lines:{covered:number,total:number},functions:{covered:number,total:number}}, mutation:{caught:number,total:number}, survived?:Array<{file:string,line:number,type:string,replacement:string}>}>} rows рядки провайдерів
 * @returns {string} Markdown-вміст для COVERAGE.md
 */
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

  const allSurvived = rows.flatMap(r => r.survived ?? [])
  if (allSurvived.length > 0) {
    lines.push('')
    lines.push('## Вижилі мутанти — рекомендації для дописування тестів')
    lines.push('')
    lines.push('<!-- Автогенеровано n-cursor coverage. Передай LLM як контекст для написання тестів. -->')

    const byFile = /** @type {Record<string, Array<{file:string,line:number,type:string,replacement:string}>>} */ ({})
    for (const m of allSurvived) {
      byFile[m.file] = byFile[m.file] ?? []
      byFile[m.file].push(m)
    }

    for (const [file, mutants] of Object.entries(byFile)) {
      lines.push('')
      lines.push(`### \`${file}\``)

      const byLine = /** @type {Record<number, Array<{type:string,replacement:string}>>} */ ({})
      for (const m of mutants) {
        byLine[m.line] = byLine[m.line] ?? []
        byLine[m.line].push({ type: m.type, replacement: m.replacement })
      }

      for (const line of Object.keys(byLine).map(Number).sort((a, b) => a - b)) {
        lines.push('')
        lines.push(`**Рядок ${line}**`)
        lines.push('')
        lines.push('| Вижив варіант | Тип мутації |')
        lines.push('|---|---|')
        for (const m of byLine[line]) {
          lines.push(`| \`${m.replacement}\` | ${m.type} |`)
        }
      }
    }
  }

  return `${lines.join('\n')}\n`
}
```

- [ ] **Step 4: Run all coverage tests**

```bash
bun test rules/js-lint/coverage/tests/ rules/test/coverage/tests/ 2>&1 | tail -10
```

Expected: `27+ pass, 0 fail` (was 27 before new tests; will be 32+ after).

- [ ] **Step 5: Run full test suite to confirm no regressions**

```bash
bun test --parallel scripts/tests/ rules/ 2>&1 | tail -5
```

Expected: `770+ pass, 0 fail`

---

## Part C — @nitra/cursor: version bump + mlmail update

Working directory: `cursor/npm/`

---

### Task C1: Bump @nitra/cursor to 1.19.3

- [ ] **Step 1: Update package.json version**

In `cursor/npm/package.json`, change `"version": "1.19.2"` to `"version": "1.19.3"`.

- [ ] **Step 2: Add CHANGELOG entry**

In `cursor/npm/CHANGELOG.md`, add above `## [1.19.2]`:

```markdown
## [1.19.3] - 2026-05-25

### Added

- **`js-lint` coverage провайдер**: `collect()` тепер повертає `survived[]` — масив вижилих мутантів `{file, line, type, replacement}` із Stryker `mutation.json`.
- **`coverage` оркестратор**: `renderMarkdown()` додає секцію `## Вижилі мутанти` до `COVERAGE.md` коли є вижилі мутанти, відформатовану як LLM-промпт для написання тестів.
```

- [ ] **Step 3: Verify integration test passes (detects version bump)**

```bash
bun test tests/integration-repo-checks.test.mjs 2>&1 | grep -E "npm.*version|pass|fail" | tail -5
```

Expected: integration test passes with version bump detection.

---

### Task C2: Update mlmail to use @nitra/cursor@1.19.3

Working directory: `mlmail/`

- [ ] **Step 1: Update package.json dependency**

In `mlmail/package.json`, change `"@nitra/cursor": "^1.19.2"` to `"@nitra/cursor": "^1.19.3"`.

- [ ] **Step 2: Install updated package**

```bash
bun install 2>&1 | grep "@nitra/cursor"
```

Expected: `+ @nitra/cursor@1.19.3` (or higher if already cached).

- [ ] **Step 3: Verify coverage still runs correctly**

```bash
node npm/bin/n-cursor.js coverage 2>&1 | tail -5
```

Wait for completion (~20 min with 142 mutants). Expected final lines:
```
✓ COVERAGE.md
```

- [ ] **Step 4: Verify COVERAGE.md has recommendations section**

```bash
grep "Вижилі мутанти" COVERAGE.md && echo "FOUND" || echo "NOT FOUND"
```

Expected: `FOUND`

---

## Part D — mlmail: n-fix-tests skill

Working directory: `mlmail/`

**Files:**
- Create: `.cursor/skills/n-fix-tests/SKILL.md`
- Modify: `CLAUDE.md` (register skill)

---

### Task D1: Create n-fix-tests skill

- [ ] **Step 1: Create skill directory and SKILL.md**

```bash
mkdir -p .cursor/skills/n-fix-tests
```

Create `.cursor/skills/n-fix-tests/SKILL.md`:

````markdown
---
name: n-fix-tests
description: Ітеративно дописує тести для вижилих мутантів у mlmail/app/src/. Запускає bun test → bun run coverage в циклі поки mutation score не покращується або не досягає цілі.
trigger: /n-fix-tests [target_score]
---

# n-fix-tests — Автоматичне посилення тестів

## Призначення

Читає `mutation.json`, визначає вижилі мутанти, дописує тести, що їх вбивають, і повторює поки score не конвергує.

## Аргументи

- `target_score` (опційно): ціль у відсотках, наприклад `/n-fix-tests 80`. Default: `max(current_score + 10, 80)`.

## Покрокова інструкція

### 0. Підготовка

Знайди `mutation.json`:
```bash
ls mlmail/app/reports/stryker/mutation.json 2>/dev/null || echo "NOT FOUND"
```

Якщо не знайдено — запропонуй спочатку виконати `bun run coverage` з кореня mlmail.

Зчитай поточний score з COVERAGE.md:
```bash
grep "JS" mlmail/COVERAGE.md
```

Визнач `target_score`:
- Якщо аргумент переданий — використай його.
- Інакше: `target = max(current_js_score + 10, 80)`.

Зчитай вижилі мутанти:
```js
const r = JSON.parse(fs.readFileSync('mlmail/app/reports/stryker/mutation.json', 'utf8'))
const survived = []
for (const [file, data] of Object.entries(r.files))
  for (const m of data.mutants)
    if (m.status === 'Survived')
      survived.push({ file, line: m.location.start.line, type: m.mutatorName, replacement: m.replacement, id: m.id })
```

Збережи поточний score як `prev_score`.

### 1. Написання тестів

Для кожного файлу з вижилими мутантами:

1. **Прочитай вихідний файл** на рядках де є мутанти:
   ```bash
   sed -n '1,30p' mlmail/app/src/<file>
   ```

2. **Знайди тест-файл** за порядком пріоритету:
   - `<same-dir>/<name>.test.js` поруч із джерелом
   - `<same-dir>/__tests__/<name>.test.js`
   - Будь-який файл у `src/` що імпортує джерело

   Якщо не знайдено — створи `<same-dir>/<name>.test.js`.

3. **Прочитай існуючий тест-файл** щоб зрозуміти конвенції (imports, describe структура, mocks).

4. **Напиши нові тест-кейси** для кожного вижилого мутанту:

   | Тип мутації | Що дописати |
   |---|---|
   | `ConditionalExpression` (`false` або `true`) | Тест що перевіряє обидва результати умови (truthy і falsy шляхи) |
   | `LogicalOperator` (`&&` → `\|\|` або навпаки) | Тести з крайніми випадками: `null && undefined`, `null \|\| undefined` |
   | `BooleanLiteral` (`true` → `false`) | Тест початкового стану або флагу |
   | `StringLiteral` (`""` замість рядка) | Тест що перевіряє конкретне значення рядка |
   | `BlockStatement` (`{}`) | Тест що перевіряє що код у блоці виконується |
   | `ArrowFunction` (`() => undefined`) | Тест що перевіряє повернуте значення функції |

   **Для Vue SFC** — використовуй `mountWithQuasar` і `flushPromises` з `./test-utils/quasar.js`, як у `src/views/Login.test.js`.

### 2. Перевірка тестів

```bash
cd mlmail && bun test app/src/<changed-test-file> 2>&1 | tail -5
```

Якщо тести падають:
- Виправ конкретну помилку (не більше 2 спроб на файл).
- Якщо після 2 спроб ще падають — залиш мутант із коментарем `# SKIP: не вдалося вбити` і переходь до наступного.

### 3. Ітерація coverage

Після дописування тестів для всіх файлів:

```bash
cd mlmail && bun run coverage 2>&1 | tail -10
```

Зачекай завершення (~20-25 хвилин з incremental).

Зчитай новий score:
```bash
grep "JS" mlmail/COVERAGE.md
```

### 4. Критерії зупинки

- `new_score >= target_score` → **ЗУПИНИСЬ**, повідом про успіх.
- `new_score <= prev_score` → **ЗУПИНИСЬ**, конвергенція — більше не можна покращити без рефакторингу тестів.
- Інакше → `prev_score = new_score`, повернись до кроку 1 з оновленим `mutation.json`.

### 5. Фінальний звіт

```
## Результат n-fix-tests

**Score до:** X.XX%  
**Score після:** Y.YY% (N→M вбитих мутантів)  
**Ітерацій:** K

### Залишкові вижилі мутанти
| Файл | Рядок | Тип | Варіант |
|---|---|---|---|
| ... | ... | ... | ... |

### Наступні кроки
- Залишкові мутанти потребують рефакторингу логіки або нових інтеграційних сценаріїв.
```
````

- [ ] **Step 2: Verify SKILL.md is valid**

```bash
head -5 .cursor/skills/n-fix-tests/SKILL.md
```

Expected: `---` frontmatter.

- [ ] **Step 3: Register skill in CLAUDE.md**

In `mlmail/CLAUDE.md`, add skill entry to the Skills section:

```markdown
- `.cursor/skills/n-fix-tests/SKILL.md` — Ітеративне посилення тестів на основі вижилих мутантів Stryker
  Команда: `/n-fix-tests [target_score]`
```

- [ ] **Step 4: Verify skill is discoverable**

```bash
grep "n-fix-tests" CLAUDE.md && echo "REGISTERED"
```

Expected: path + `REGISTERED`.

---

## Self-review checklist

- [x] Task A covers spec requirement A (incremental: true + gitignore)
- [x] Task B1 covers spec B — `collect()` returns `survived[]` with TDD
- [x] Task B2 covers spec B — `renderMarkdown` appends recommendations with TDD
- [x] Task C covers publish + mlmail update
- [x] Task D covers spec C — n-fix-tests skill with full algorithm
- [x] All code steps have complete code (no TBDs)
- [x] Type signatures consistent: `survived: Array<{file:string,line:number,type:string,replacement:string}>` used in B1, B2, and SKILL.md
- [x] No commit steps (per user preference from memory)
