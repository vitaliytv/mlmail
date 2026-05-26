# /n-coverage-fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Recommendations section to COVERAGE.md + `/n-coverage-fix` skill that auto-writes tests for survived mutants until convergence.

**Architecture:** Extend js-lint coverage provider to extract survived mutants; orchestrator renders Recommendations section; new skill guides Claude through iterative test-writing loop.

**Tech Stack:** Bun, `@stryker-mutator/core`, `@nitra/cursor` CLI, Claude Code skills

---

## Передумови

- Курсор-пакет: `/Users/vitaliytv/www/nitra/cursor` (основне дерево), поточна версія `1.19.2` у `npm/package.json`
- Воркдерево з incremental-фіксом: `/Users/vitaliytv/www/nitra/cursor/.claude/worktrees/fix/stryker-incremental`, версія там `1.19.3`
- mlmail: `/Users/vitaliytv/www/vitaliytv/mlmail`
- У mlmail `app/stryker.config.mjs` вже містить `incremental: true` + `incrementalFile` — Task 1 лише верифікує це (якщо ні — додає)
- У mlmail `.gitignore` рядок `**/reports/stryker/` вже покриває `incremental.json` — нова строка не потрібна
- 46 вижилих мутантів у `app/reports/stryker/mutation.json`: 41 у `src/services/auth-store.js`, 4 у `src/i18n/auth-errors.js`, 1 у `src/views/Login.vue`
- Тестові файли-приклади існують: `app/src/i18n/auth-errors.test.js`, `app/src/services/auth-store.test.js`

---

## Task 1: Застосувати incremental-фікс із воркдерева + верифікувати mlmail stryker config

**Репо:** `@nitra/cursor` (основне дерево) + mlmail

### Кроки

- [ ] Перейти до основного дерева cursor: `cd /Users/vitaliytv/www/nitra/cursor`
- [ ] Порівняти воркдерево з main:
  ```bash
  diff \
    .claude/worktrees/fix/stryker-incremental/npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs \
    npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs
  ```
- [ ] Скопіювати файл із воркдерева в основне дерево:
  ```bash
  cp \
    .claude/worktrees/fix/stryker-incremental/npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs \
    npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs
  ```
- [ ] Оновити `npm/package.json` — версія `1.19.2` → `1.19.3` (патч за incremental fix):
  - Файл: `/Users/vitaliytv/www/nitra/cursor/npm/package.json`
  - Змінити: `"version": "1.19.2"` → `"version": "1.19.3"`
- [ ] Перевірити, що `app/stryker.config.mjs` у mlmail вже містить `incremental: true` та `incrementalFile`:
  ```bash
  grep -n "incremental" /Users/vitaliytv/www/vitaliytv/mlmail/app/stryker.config.mjs
  ```
  Очікуваний вивід (обидва рядки вже присутні — тоді нічого не змінювати):
  ```
  24:  incremental: true,
  25:  incrementalFile: 'reports/stryker/stryker-incremental.json',
  ```
  Якщо рядків немає — додати до `app/stryker.config.mjs` перед останнім `}` (використовуємо ту ж назву, що і в baseline):
  ```js
  // incremental: зберігає результати між запусками, відновлює після краш/kill.
  incremental: true,
  incrementalFile: 'reports/stryker/stryker-incremental.json',
  ```
  Примітка: назва файлу (`stryker-incremental.json`) відрізняється від `incremental.json` у baseline — це нормально, mlmail має власний стряйкер-конфіг із кастомними `mutate` патернами.
- [ ] Перевірити `.gitignore`:
  ```bash
  grep "reports/stryker" /Users/vitaliytv/www/vitaliytv/mlmail/.gitignore
  ```
  Якщо рядок `**/reports/stryker/` або `app/reports/` присутній — нічого не додавати (вже покрито).
- [ ] Запустити тести cursor-пакета (лише js-lint coverage):
  ```bash
  cd /Users/vitaliytv/www/nitra/cursor/npm && bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs
  ```
- [ ] Перевірити стан:
  ```bash
  cd /Users/vitaliytv/www/nitra/cursor && git status && git diff --stat
  ```

---

## Task 2: Розширити js-lint coverage provider

**Файл:** `/Users/vitaliytv/www/nitra/cursor/npm/rules/js-lint/coverage/coverage.mjs`
**Тести:** `/Users/vitaliytv/www/nitra/cursor/npm/rules/js-lint/coverage/tests/coverage.test.mjs`

### Контекст поточного стану

- `parseStrykerReport(report, jsRoot)` вже повертає `{ caught, total, survived }`, де `survived` — масив `{ file, line, original, replacement, type }`
- `collect()` повертає `[{ area: 'JS', coverage, mutation: { caught, total }, survived }]`
- `survived` вже пробрасується у рядок результату — оркестратор вже рендерить таблицю мутантів

### Що змінюється

Провайдер потребує нової функції `findExampleTest(jsRoot, sourceFilePath)` та нової функції `generateRecommendationsMarkdown(jsRoot, survived)`.
`collect()` розширить тип повернення полем `recommendations: string`.

### TDD-послідовність

**Крок 2.1 — написати failing тести**

Спочатку оновити рядок import у тест-файлі `/Users/vitaliytv/www/nitra/cursor/npm/rules/js-lint/coverage/tests/coverage.test.mjs`:

- Поточний: `import { collect, detect, parseStrykerReport } from '../coverage.mjs'`
- Замінити на: `import { collect, detect, findExampleTest, generateRecommendationsMarkdown, parseStrykerReport } from '../coverage.mjs'`

Потім додати нові `describe` блоки:

```js
describe('findExampleTest', () => {
  test('знаходить .test.js поруч із source-файлом', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'find-example-'))
    mkdirSync(join(dir, 'src'), { recursive: true })
    writeFileSync(join(dir, 'src', 'foo.js'), 'export function foo() {}')
    writeFileSync(join(dir, 'src', 'foo.test.js'), "it('works', () => {})\n")
    const result = await findExampleTest(dir, 'src/foo.js')
    expect(result).toContain("it('works'")
    rmSync(dir, { recursive: true, force: true })
  })

  test('знаходить .spec.js як альтернативу', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'find-example-'))
    mkdirSync(join(dir, 'src'), { recursive: true })
    writeFileSync(join(dir, 'src', 'bar.js'), '')
    writeFileSync(join(dir, 'src', 'bar.spec.js'), "test('bar', () => {})\n")
    const result = await findExampleTest(dir, 'src/bar.js')
    expect(result).toContain("test('bar'")
    rmSync(dir, { recursive: true, force: true })
  })

  test('знаходить тест у test/<basename>.test.js', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'find-example-'))
    mkdirSync(join(dir, 'src'), { recursive: true })
    mkdirSync(join(dir, 'test'), { recursive: true })
    writeFileSync(join(dir, 'src', 'baz.js'), '')
    writeFileSync(join(dir, 'test', 'baz.test.js'), "it('baz runs', () => {})\n")
    const result = await findExampleTest(dir, 'src/baz.js')
    expect(result).toContain("it('baz runs'")
    rmSync(dir, { recursive: true, force: true })
  })

  test('повертає null якщо тестовий файл не знайдено', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'find-example-'))
    mkdirSync(join(dir, 'src'), { recursive: true })
    writeFileSync(join(dir, 'src', 'notest.js'), '')
    const result = await findExampleTest(dir, 'src/notest.js')
    expect(result).toBeNull()
    rmSync(dir, { recursive: true, force: true })
  })
})

describe('generateRecommendationsMarkdown', () => {
  test('повертає порожній рядок якщо survived порожній', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'gen-rec-'))
    const result = await generateRecommendationsMarkdown(dir, [])
    expect(result).toBe('')
    rmSync(dir, { recursive: true, force: true })
  })

  test('рендерить заголовок ## Recommendations і таблицю мутантів', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'gen-rec-'))
    mkdirSync(join(dir, 'src'), { recursive: true })
    writeFileSync(join(dir, 'src', 'auth.js'), 'if (x) return\n')
    const survived = [
      { file: 'src/auth.js', line: 1, type: 'ConditionalExpression', original: 'if (x) return', replacement: 'false' }
    ]
    const result = await generateRecommendationsMarkdown(dir, survived)
    expect(result).toContain('## Recommendations')
    expect(result).toContain('### src/auth.js')
    expect(result).toContain('**Вижило мутантів: 1**')
    expect(result).toContain('| 1 | ConditionalExpression |')
    rmSync(dir, { recursive: true, force: true })
  })

  test('додає блок "Приклад наявного тесту" якщо test-файл знайдено', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'gen-rec-'))
    mkdirSync(join(dir, 'src'), { recursive: true })
    writeFileSync(join(dir, 'src', 'calc.js'), 'export function add(a, b) { return a + b }\n')
    writeFileSync(join(dir, 'src', 'calc.test.js'), "it('adds', () => { expect(add(1,2)).toBe(3) })\n")
    const survived = [
      { file: 'src/calc.js', line: 1, type: 'ArithmeticOperator', original: 'a + b', replacement: 'a - b' }
    ]
    const result = await generateRecommendationsMarkdown(dir, survived)
    expect(result).toContain('**Приклад наявного тесту:**')
    expect(result).toContain("it('adds'")
    rmSync(dir, { recursive: true, force: true })
  })
})

describe('collect() — поле recommendations', () => {
  test('collect повертає рядок recommendations у результаті', async () => {
    const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })

    const reportDir = join(dir, 'reports', 'stryker')
    mkdirSync(reportDir, { recursive: true })
    mkdirSync(join(dir, 'src'), { recursive: true })
    writeFileSync(join(dir, 'src', 'a.js'), 'if (x) return\n')
    writeFileSync(
      join(reportDir, 'mutation.json'),
      JSON.stringify({
        files: {
          'src/a.js': {
            mutants: [
              {
                status: 'Survived',
                mutatorName: 'ConditionalExpression',
                replacement: 'false',
                location: { start: { line: 1 } }
              }
            ]
          }
        }
      })
    )

    const runner = {
      runJsCoverage({ lcovDir }) {
        writeFileSync(join(lcovDir, 'lcov.info'), 'LF:1\nLH:0\nFNF:1\nFNH:0\n')
        return 0
      },
      runStryker() {
        return 0
      }
    }

    const rows = await collect(dir, { runner })
    expect(typeof rows[0].recommendations).toBe('string')
    rmSync(dir, { recursive: true, force: true })
  })

  test('recommendations порожній рядок якщо немає survived', async () => {
    const dir = makeFixture({ scripts: { 'test:coverage': 'bun test --coverage' } })
    const reportDir = join(dir, 'reports', 'stryker')
    mkdirSync(reportDir, { recursive: true })
    writeFileSync(
      join(reportDir, 'mutation.json'),
      JSON.stringify({
        files: {
          'src/b.js': { mutants: [{ status: 'Killed' }, { status: 'Killed' }] }
        }
      })
    )

    const runner = {
      runJsCoverage({ lcovDir }) {
        writeFileSync(join(lcovDir, 'lcov.info'), 'LF:10\nLH:10\nFNF:2\nFNH:2\n')
        return 0
      },
      runStryker() {
        return 0
      }
    }

    const rows = await collect(dir, { runner })
    expect(rows[0].recommendations).toBe('')
    rmSync(dir, { recursive: true, force: true })
  })
})
```

**Крок 2.2 — переконатися що тести падають**

```bash
cd /Users/vitaliytv/www/nitra/cursor/npm && bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | tail -20
```

Очікується: `findExampleTest is not a function` / `generateRecommendationsMarkdown is not a function` — тести red.

**Крок 2.3 — реалізувати функції в `coverage.mjs`**

Додати до `/Users/vitaliytv/www/nitra/cursor/npm/rules/js-lint/coverage/coverage.mjs`:

1. Розширити статичний import `node:path` — додати `basename` та `dirname`:
   - Поточний рядок у файлі: `import { join } from 'node:path'`
   - Замінити на: `import { basename, dirname, join } from 'node:path'`

2. Функція `findExampleTest(jsRoot, sourceFilePath)`:

   ```js
   /**
    * Шукає тестовий файл для source-файла; повертає перші 10 рядків першого it()/test() блоку або null.
    * @param {string} jsRoot корінь JS-проєкту
    * @param {string} sourceFilePath відносний шлях до source-файла від jsRoot
    * @returns {Promise<string|null>} перші рядки test-блоку або null
    */
   export async function findExampleTest(jsRoot, sourceFilePath) {
     // Кандидати: поруч .test.js / .spec.js, test/<base>.test.js, tests/<base>.test.js
     const base = basename(sourceFilePath, '.js')
     const dir = dirname(sourceFilePath)
     const candidates = [
       join(jsRoot, dir, `${base}.test.js`),
       join(jsRoot, dir, `${base}.spec.js`),
       join(jsRoot, 'test', `${base}.test.js`),
       join(jsRoot, 'tests', `${base}.test.js`)
     ]
     for (const candidate of candidates) {
       if (!existsSync(candidate)) continue
       const content = await readFile(candidate, 'utf8')
       // Знайти перший it( або test(
       const idx = content.search(/\bit\s*\(|\btest\s*\(/)
       if (idx === -1) continue
       // Повернути 10 рядків починаючи з цього місця
       return content.slice(idx).split('\n').slice(0, 10).join('\n')
     }
     return null
   }
   ```

   Примітка: `join`, `existsSync`, `readFile` вже імпортовані у файлі. Потрібно лише додати `basename` та `dirname` до рядка імпорту.

3. Функція `generateRecommendationsMarkdown(jsRoot, survived)`:

   ````js
   /**
    * Генерує Markdown-рядок для розділу ## Recommendations.
    * @param {string} jsRoot корінь JS-проєкту
    * @param {Array<{file:string,line:number,type:string,original:string,replacement:string}>} survived вижилі мутанти
    * @returns {Promise<string>} markdown або '' якщо survived порожній
    */
   export async function generateRecommendationsMarkdown(jsRoot, survived) {
     if (survived.length === 0) return ''

     // Групуємо мутанти по файлу
     const byFile = {}
     for (const m of survived) {
       if (!byFile[m.file]) byFile[m.file] = []
       byFile[m.file].push(m)
     }

     const lines = [
       '## Recommendations',
       '',
       '> Автоматично генерується `bun coverage`. Використовується `/n-coverage-fix` для автоматичного виправлення.',
       ''
     ]

     for (const [file, mutants] of Object.entries(byFile)) {
       lines.push(`### ${file}`, '')
       lines.push(`**Вижило мутантів: ${mutants.length}**`, '')
       lines.push('| Рядок | Тип | Оригінал | Вижив |')
       lines.push('|---|---|---|---|')
       for (const m of mutants) {
         const orig = (m.original ?? '').replace(/\|/g, '\\|').replace(/`/g, "'")
         const repl = (m.replacement ?? '').replace(/\|/g, '\\|').replace(/`/g, "'")
         lines.push(`| ${m.line} | ${m.type} | \`${orig}\` | \`${repl}\` |`)
       }
       lines.push('')

       // Приклад тесту
       const example = await findExampleTest(jsRoot, file)
       if (example) {
         lines.push('**Приклад наявного тесту:**')
         lines.push('```js')
         lines.push(example)
         lines.push('```')
         lines.push('')
       }
     }

     return lines.join('\n')
   }
   ````

4. Оновити `collect()` — розширити тип повернення та додати виклик:
   - Після `const { caught, total, survived } = await parseStrykerReport(mutationReport, jsRoot)` додати:
     ```js
     const recommendations = await generateRecommendationsMarkdown(jsRoot, survived)
     ```
   - Змінити рядок return:
     ```js
     return [{ area: 'JS', coverage, mutation: { caught, total }, survived, recommendations }]
     ```

**Крок 2.4 — запустити тести: мають пройти**

```bash
cd /Users/vitaliytv/www/nitra/cursor/npm && bun test npm/rules/js-lint/coverage/tests/coverage.test.mjs 2>&1 | tail -20
```

Очікується: всі тести green.

**Крок 2.5 — перевірити стан**

```bash
cd /Users/vitaliytv/www/nitra/cursor && git status && git diff --stat
```

---

## Task 3: Оновити оркестратор

**Файл:** `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/coverage.mjs`
**Тести:** `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/tests/coverage.test.mjs`

### Контекст поточного стану

- `renderMarkdown(rows)` вже має логіку для `allSurvived` і рендерить рядок `## Рекомендації` з таблицею
- Нова схема: провайдер тепер повертає готовий `recommendations: string` — оркестратор більше не будує Рекомендації сам, а агрегує ready-made markdown від провайдерів
- `runCoverageSteps` збирає `rows` і передає їх у `renderMarkdown`

### Що змінюється

`renderMarkdown` прибирає власну логіку генерації Рекомендацій на основі `allSurvived`. Натомість агрегує рядки `row.recommendations` від провайдерів і додає їх після таблиці. `runCoverageSteps` не змінюється (він вже пробрасує rows у renderMarkdown).

### TDD-послідовність

**Крок 3.1 — написати failing тести**

Додати до `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/tests/coverage.test.mjs`:

```js
describe('renderMarkdown — recommendations від провайдера', () => {
  test('додає recommendations-рядок після таблиці', () => {
    const rows = [
      {
        area: 'JS',
        coverage: { lines: { covered: 5, total: 10 }, functions: { covered: 2, total: 4 } },
        mutation: { caught: 3, total: 5 },
        recommendations: '## Recommendations\n\n> generated\n\n### src/foo.js\n'
      }
    ]
    const md = renderMarkdown(rows)
    expect(md).toContain('## Recommendations')
    expect(md).toContain('### src/foo.js')
    // Рядок має стояти після таблиці
    const tableEnd = md.indexOf('| **Разом** |')
    const recStart = md.indexOf('## Recommendations')
    expect(recStart).toBeGreaterThan(tableEnd)
  })

  test('не додає ## Recommendations якщо всі recommendations порожні', () => {
    const rows = [
      {
        area: 'JS',
        coverage: { lines: { covered: 10, total: 10 }, functions: { covered: 5, total: 5 } },
        mutation: { caught: 5, total: 5 },
        recommendations: ''
      }
    ]
    const md = renderMarkdown(rows)
    expect(md).not.toContain('## Recommendations')
  })

  test("об'єднує recommendations від двох провайдерів", () => {
    const rows = [
      {
        area: 'JS',
        coverage: { lines: { covered: 5, total: 10 }, functions: { covered: 2, total: 4 } },
        mutation: { caught: 3, total: 5 },
        recommendations: '## Recommendations\n\n### src/a.js\n'
      },
      {
        area: 'Rust',
        coverage: { lines: { covered: 8, total: 10 }, functions: { covered: 4, total: 5 } },
        mutation: { caught: 5, total: 5 },
        recommendations: '### src/b.rs\n'
      }
    ]
    const md = renderMarkdown(rows)
    expect(md).toContain('### src/a.js')
    expect(md).toContain('### src/b.rs')
  })
})

describe('runCoverageSteps — пробрасує recommendations у COVERAGE.md', () => {
  test('COVERAGE.md містить Recommendations якщо провайдер повертає непорожній рядок', async () => {
    const WITH_RECOMMENDATIONS = `
      export async function detect() { return true }
      export async function collect() {
        return [{
          area: 'Test',
          coverage: { lines: { covered: 5, total: 10 }, functions: { covered: 2, total: 4 } },
          mutation: { caught: 3, total: 5 },
          recommendations: '## Recommendations\\n\\n### src/foo.js\\n'
        }]
      }
    `
    const fx = makeOrchestratorFixture({ rules: ['js-lint'], providers: { 'js-lint': WITH_RECOMMENDATIONS } })
    await runCoverageSteps({ cwd: fx.cwd, rulesDir: fx.rulesDir })
    const md = readFileSync(join(fx.cwd, 'COVERAGE.md'), 'utf8')
    expect(md).toContain('## Recommendations')
    expect(md).toContain('### src/foo.js')
    fx.cleanup()
  })
})
```

**Крок 3.2 — переконатися що тести падають (або що стара логіка survived конфліктує)**

```bash
cd /Users/vitaliytv/www/nitra/cursor/npm && bun test npm/rules/test/coverage/tests/coverage.test.mjs 2>&1 | tail -20
```

**Крок 3.3 — реалізувати зміни в оркестраторі**

У `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/coverage.mjs`:

1. Замінити тіло `renderMarkdown(rows)` — видалити блок `allSurvived` (рядки 91–113) і замість нього додати агрегацію `recommendations`:

   ```js
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

     // Агрегуємо recommendations від провайдерів (рядки або undefined → фільтруємо порожні)
     const allRecommendations = rows
       .map(r => r.recommendations ?? '')
       .filter(s => s.length > 0)
       .join('\n')

     if (allRecommendations.length > 0) {
       lines.push('', allRecommendations)
     }

     return `${lines.join('\n')}\n`
   }
   ```

   Важливо: рядки `rows.push({ area: '**Разом**', ... })` у `buildTotalsRow` не мають `survived` — це нормально, бо `recommendations ?? ''` безпечний.

**Крок 3.4 — оновити існуючий тест `renderMarkdown` що перевіряє survived**

Старі тести `додає розділ Рекомендації коли є survived мутанти` і `рядки без survived не додають розділ Рекомендації` тепер перевіряють поведінку через поле `recommendations`, а не через `survived`. Адаптувати ці тести:

- `рядки без survived не додають розділ Рекомендації` — залишити (стане true, бо `recommendations: undefined` → `''`)
- `додає розділ Рекомендації коли є survived мутанти` — змінити: прибрати `survived:` з row, додати `recommendations: '## Рекомендації...'`
- `екранує | і ...` — аналогічно — тест перевіряє вивід що приходить від провайдера; якщо він уже покриває це у js-lint coverage тестах — видалити тут

**Крок 3.5 — запустити тести: мають пройти**

```bash
cd /Users/vitaliytv/www/nitra/cursor/npm && bun test npm/rules/test/coverage/tests/coverage.test.mjs 2>&1 | tail -20
```

**Крок 3.6 — перевірити стан**

```bash
cd /Users/vitaliytv/www/nitra/cursor && git status && git diff --stat
```

---

## Task 4: Створити скіл `/n-coverage-fix`

**Файл:** `/Users/vitaliytv/www/nitra/cursor/npm/skills/coverage-fix/SKILL.md`

### Кроки

- [ ] Перевірити формат існуючого скілу для зразка:

  ```bash
  head -20 /Users/vitaliytv/www/nitra/cursor/npm/skills/fix/SKILL.md
  ```

- [ ] Створити директорію та SKILL.md:

  ```bash
  mkdir -p /Users/vitaliytv/www/nitra/cursor/npm/skills/coverage-fix
  ```

- [ ] Написати `/Users/vitaliytv/www/nitra/cursor/npm/skills/coverage-fix/SKILL.md`:

```markdown
---
name: n-coverage-fix
description: >-
  Автономна команда: запускає coverage, читає Recommendations, пише тести для вижилих мутантів, ітерує до конвергенції.
---

# /n-coverage-fix

Автономна команда: запускає `bun coverage`, читає секцію `## Recommendations` у `COVERAGE.md`, пише нові тести для кожного вижилого мутанта, повторює цикл поки mutation score зростає (конвергенція).

## Algorithm

1. Запусти `bun coverage` (або `npx @nitra/cursor coverage` з кореня проєкту) — чекай завершення
2. Прочитай `COVERAGE.md` секцію `## Recommendations`
3. Якщо секції немає або вона порожня → вивести "Нема вижилих мутантів" → DONE
4. Запам'ятай поточний mutation score як `baseline_score` (з рядка `| **Разом** |` таблиці COVERAGE.md)
5. Для кожного файлу у Recommendations:
   a. Прочитай source-файл (mutated lines ± 5 рядків контексту)
   b. Прочитай тестовий файл, показаний у **Приклад наявного тесту**
   c. Напиши нові тест-кейси що зловлять кожен вижилий мутант:
   - `ConditionalExpression` (`false`/`true`): протестуй branch явно з тригерним значенням
   - `BooleanLiteral` (`true`→`false`): перевір початковий стан (initial value = false)
   - `LogicalOperator` (`&&`↔`||`): передай `null` та `undefined` **окремо**, перевір що результат різний
   - `StringLiteral`/`EqualityOperator`: перевір точний рядок/значення, а не лише happy path
     d. Використовуй приклад тесту як style guide (той самий `describe`/`it`/`expect`, мова коментарів)
6. Запусти `bun test` (повний suite)
   - Якщо FAIL:
     - Не відкочувати зміни
     - Показати: яка помилка, які файли змінені, що вже покращено
     - Очікувати рішення від user:
       - [виправити вручну → продовжити]
       - [пропустити файл]
       - [зупинити]
   - Якщо PASS: продовжити
7. Запусти `bun coverage` знову
   - Якщо CRASH (SIGURG, memory pressure): нагадати user що Stryker incremental зберіг прогрес → перезапустити `bun coverage`
8. Порівняй новий mutation score з `baseline_score`
9. Якщо НЕ покращився → вивести підсумок → DONE (конвергенція досягнута)
10. Інакше: `baseline_score = новий score` → перейти до кроку 5 (наступна ітерація)

## Notes

- Stryker incremental (`incrementalFile`) зберігає прогрес між запусками — crash не означає перезапуск з нуля
- Не комітити зміни автоматично — user вирішує коли комітити
- Пріоритет: файли з найбільшою кількістю вижилих мутантів (перший у списку = найважливіший)
- Якщо `COVERAGE.md` відсутній — запустити `bun coverage` спочатку
```

- [ ] Перевірити стан:
  ```bash
  cd /Users/vitaliytv/www/nitra/cursor && git status && git diff --stat
  ```

---

## Task 5: Bumped версія до 1.20.0 + CHANGELOG

**Файли:**

- `/Users/vitaliytv/www/nitra/cursor/npm/package.json`
- `/Users/vitaliytv/www/nitra/cursor/npm/CHANGELOG.md`

### Кроки

- [ ] Оновити версію в `npm/package.json`: `"version": "1.19.3"` → `"version": "1.20.0"`

- [ ] Додати запис у `npm/CHANGELOG.md` **перед** рядком `## [1.19.3]` (або одразу після `# Changelog` блоку якщо 1.19.3 ще не фінальний):

```markdown
## [1.20.0] - 2026-05-25

### Added

- **`/n-coverage-fix` skill** — автономний скіл для ітеративного покращення mutation score: запускає `bun coverage`, читає `## Recommendations` у COVERAGE.md, пише тести для кожного вижилого мутанта, повторює до конвергенції. Реалізовано у `npm/skills/coverage-fix/SKILL.md`.
- **`## Recommendations` у COVERAGE.md** — `js-lint` coverage провайдер тепер генерує секцію з вижилими мутантами: для кожного файлу — таблиця `рядок | тип | оригінал | вижив`, приклад наявного тесту (перші 10 рядків першого `it()`/`test()` блоку з відповідного test-файлу). Нові функції: `findExampleTest`, `generateRecommendationsMarkdown`.

### Changed

- **`stryker.config.baseline.mjs`** — додано `incremental: true` + `incrementalFile: 'reports/stryker/incremental.json'`. Stryker зберігає результати між запусками, відновлює після краш/kill (актуально для macOS memory pressure).
- **Оркестратор `renderMarkdown`** — прибрано власну побудову Рекомендацій з `survived`; тепер агрегує готовий `recommendations: string` від провайдерів.
```

- [ ] Перевірити стан:
  ```bash
  cd /Users/vitaliytv/www/nitra/cursor && git status && git diff --stat
  ```

---

## Task 6: End-to-end верифікація

**Де виконується:** mlmail (`/Users/vitaliytv/www/vitaliytv/mlmail`)

### Передумови

- `@nitra/cursor` задеплоєний або підключений локально через `file:../../nitra/cursor/npm` у mlmail `package.json`
- Або: перевірити з `npx --yes file:/Users/vitaliytv/www/nitra/cursor/npm coverage` з директорії mlmail

### Кроки

- [ ] Перевірити поточну версію `@nitra/cursor` у mlmail:

  ```bash
  cat /Users/vitaliytv/www/vitaliytv/mlmail/package.json | grep nitra
  ```

- [ ] Якщо mlmail використовує локальний шлях або npm-пакет — підв'язати нову версію:

  ```bash
  cd /Users/vitaliytv/www/vitaliytv/mlmail && bun add @nitra/cursor@1.20.0
  # або якщо local link:
  # bun add file:../../nitra/cursor/npm
  ```

- [ ] Запустити coverage у mlmail:

  ```bash
  cd /Users/vitaliytv/www/vitaliytv/mlmail && bun coverage
  ```

  Очікується: `✓ COVERAGE.md` без помилок

- [ ] Перевірити що COVERAGE.md містить `## Recommendations`:

  ```bash
  grep -A 5 "## Recommendations" /Users/vitaliytv/www/vitaliytv/mlmail/COVERAGE.md
  ```

  Очікується: таблиця з 46 вижилими мутантами по трьох файлах (`src/i18n/auth-errors.js`, `src/services/auth-store.js`, `src/views/Login.vue`)

- [ ] Запустити `/n-coverage-fix` (через Claude Code):
  - Це інтерактивний крок — виконується як скіл у Claude Code сесії
  - Перевірити що скіл: зчитав `## Recommendations`, написав тести, запустив `bun test`, запустив `bun coverage`

- [ ] Перевірити що mutation score покращився (baseline: ~67.61%):

  ```bash
  grep "Разом" /Users/vitaliytv/www/vitaliytv/mlmail/COVERAGE.md
  ```

  Очікується: score > 67.61%

- [ ] Перевірити стан mlmail:
  ```bash
  cd /Users/vitaliytv/www/vitaliytv/mlmail && git status && git diff --stat
  ```

---

## Структура змінених файлів

| Репо   | Файл                                                                | Зміна                                                                                        |
| ------ | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| cursor | `npm/rules/test/js/data/stryker_config/stryker.config.baseline.mjs` | `incremental: true` + `incrementalFile` (з воркдерева)                                       |
| cursor | `npm/rules/js-lint/coverage/coverage.mjs`                           | `findExampleTest`, `generateRecommendationsMarkdown`, `collect()` + `recommendations`        |
| cursor | `npm/rules/js-lint/coverage/tests/coverage.test.mjs`                | нові тести для двох нових функцій і поля `recommendations`                                   |
| cursor | `npm/rules/test/coverage/coverage.mjs`                              | `renderMarkdown` агрегує `recommendations` від провайдерів замість власної логіки `survived` |
| cursor | `npm/rules/test/coverage/tests/coverage.test.mjs`                   | нові тести для оновленого `renderMarkdown` і `runCoverageSteps`                              |
| cursor | `npm/skills/coverage-fix/SKILL.md`                                  | новий скіл                                                                                   |
| cursor | `npm/package.json`                                                  | версія `1.19.2` → `1.20.0`                                                                   |
| cursor | `npm/CHANGELOG.md`                                                  | запис для `1.20.0`                                                                           |
| mlmail | `app/stryker.config.mjs`                                            | верифікація `incremental: true` (вже є)                                                      |

---

## Типи і контракти

### `CoverageRow` (оновлений тип)

```ts
interface CoverageRow {
  area: string
  coverage: {
    lines: { covered: number; total: number }
    functions: { covered: number; total: number }
  }
  mutation: { caught: number; total: number }
  survived?: Array<{ file: string; line: number; original: string; replacement: string; type: string }>
  recommendations: string // '' якщо немає вижилих; markdown якщо є
}
```

### `findExampleTest` — пошук тестових файлів

Порядок перевірки кандидатів:

1. `<jsRoot>/<dirname(sourceFilePath)>/<basename>.test.js`
2. `<jsRoot>/<dirname(sourceFilePath)>/<basename>.spec.js`
3. `<jsRoot>/test/<basename>.test.js`
4. `<jsRoot>/tests/<basename>.test.js`

Повертає перші 10 рядків від першого `it(` або `test(` включно, або `null`.

### `generateRecommendationsMarkdown` — формат виводу

````markdown
## Recommendations

> Автоматично генерується `bun coverage`. Використовується `/n-coverage-fix` для автоматичного виправлення.

### src/i18n/auth-errors.js

**Вижило мутантів: 4**

| Рядок | Тип                   | Оригінал                                                         | Вижив                                 |
| ----- | --------------------- | ---------------------------------------------------------------- | ------------------------------------- |
| 19    | ConditionalExpression | `if (kind === null \|\| kind === undefined) return messages.Unk` | `false`                               |
| 19    | LogicalOperator       | `if (kind === null \|\| kind === undefined) return messages.Unk` | `kind === null && kind === undefined` |

**Приклад наявного тесту:**

```js
it('falls back to Unknown message for unknown kinds', () => {
  expect(errorMessage('SomethingNotInTable')).toBe('Невідома помилка.')
})
```
````

```

---

## Ризики і застереження

- **`renderMarkdown` — breaking change для існуючих провайдерів без `recommendations`.** Поле `recommendations ?? ''` у агрегації безпечне — якщо провайдер не повертає поле, оркестратор ігнорує його (не крашить). Тест `рядки без survived не додають розділ Рекомендації` залишається green.
- **`survived` поле у `CoverageRow` застаріло для зовнішніх споживачів.** Оркестратор більше не читає `row.survived` — але він лишається у типі для backward compatibility. Видалити у major-версії (2.x).
- **`findExampleTest` — `.vue` файли.** `src/views/Login.vue` не має парного `.test.js`/`.spec.js` — функція поверне `null`, і секція "Приклад наявного тесту" не додасться. Прийнятна поведінка.
- **`generateRecommendationsMarkdown` — `.vue` розширення.** Шлях `src/views/Login.vue` передається у `findExampleTest` з розширенням `.vue` — basename + `.test.js` = `Login.vue.test.js` не існує. Додати до кандидатів `<base>.vue.test.js` або `<base>.test.js` зі strip `.vue` — покриється у наступній версії (не блокер для 1.20.0, `null` graceful).
```
