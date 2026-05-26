# Coverage Fix Agent — Implementation Plan

## Context

Реалізація трьох компонентів зі spec `docs/superpowers/specs/2026-05-25-coverage-fix-agent-design.md`:
1. Stryker incremental mode (стабільність)
2. Розділ `## Рекомендації` у COVERAGE.md з вижилими мутантами
3. `n-cursor coverage --fix` режим через Claude Code SDK

Зміни в двох репозиторіях: `@nitra/cursor` (`/Users/vitaliytv/www/nitra/cursor/npm/`) і `mlmail` (поточний).

---

## Phase 1: Stryker incremental mode

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/scripts/js-lint.mjs` (рядки 32-40), у шаблонному тексті `stryker.config.mjs` що виводиться через `console.log`, додати після `reporters: ['json'],`:
  ```
  incremental: true,
  incrementalFile: 'reports/stryker/stryker-incremental.json',
  ```
  - Шаблон виводиться як рядок у `console.log` — редагувати той самий рядковий літерал

- [ ] У `/Users/vitaliytv/www/vitaliytv/mlmail/app/stryker.config.mjs`, після рядка `reporters: ['json'],`, додати:
  ```js
  incremental: true,
  incrementalFile: 'reports/stryker/stryker-incremental.json',
  ```

- [ ] У `/Users/vitaliytv/www/vitaliytv/mlmail/.gitignore`, після рядка `reports/stryker/.tmp/`, додати:
  ```
  reports/stryker/stryker-incremental.json
  ```

- [ ] Перевірка: `bunx @stryker-mutator/core run` у `mlmail/app/` створює `app/reports/stryker/stryker-incremental.json` після першого прогону; другий прогін помітно швидший (пропускає вже протестовані мутанти)

---

## Phase 2: Survived mutants data layer

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/rules/js-lint/coverage/coverage.mjs`, змінити `runStryker(jsRoot)` — передати `jsRoot` до `parseStrykerReport`:
  ```js
  function runStryker(jsRoot) {
    execSync('bunx @stryker-mutator/core run', { cwd: jsRoot, stdio: 'inherit' })
    const reportPath = join(jsRoot, 'reports', 'stryker', 'mutation.json')
    const report = JSON.parse(readFileSync(reportPath, 'utf8'))
    return parseStrykerReport(report, jsRoot)  // ← додати jsRoot
  }
  ```

- [ ] У тому самому файлі, змінити `parseStrykerReport(report)` → `parseStrykerReport(report, jsRoot)` та додати збір вижилих:
  ```js
  function parseStrykerReport(report, jsRoot) {
    let caught = 0
    let total = 0
    const survived = []
    for (const [relPath, file] of Object.entries(report.files)) {
      const srcLines = readFileSync(join(jsRoot, relPath), 'utf8').split('\n')
      for (const mutant of file.mutants) {
        if (mutant.status === 'Killed' || mutant.status === 'Timeout') {
          caught += 1
          total += 1
        } else if (mutant.status === 'Survived' || mutant.status === 'NoCoverage') {
          total += 1
          if (mutant.status === 'Survived') {
            survived.push({
              file: relPath,
              line: mutant.location.start.line,
              original: srcLines[mutant.location.start.line - 1]?.trim() ?? '',
              replacement: mutant.replacement,
              type: mutant.mutatorName,
            })
          }
        }
      }
    }
    return {
      coverage: { lines: null, functions: null },
      mutations: { killed: caught, total },
      survived,
    }
  }
  ```
  - `location.start.line` — 1-індексований (перевірено: рядок 3 = третій рядок файлу)
  - `readFileSync` вже імпортований у файлі

- [ ] У `collect(jsRoot)`, включити `survived` у return:
  ```js
  return {
    area: 'JS',
    coverage: merge(bunCov, strykerMutation.coverage),
    mutations: strykerMutation.mutations,
    survived: strykerMutation.survived,
  }
  ```

---

## Phase 3: Recommendations у COVERAGE.md

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/coverage.mjs`, змінити `renderMarkdown(rows)` — після основної таблиці, якщо є вижилі мутанти, додати `## Рекомендації`:
  ```js
  export function renderMarkdown(rows) {
    const lines = [
      '# Coverage',
      '',
      '| Область | Рядки | Функції | Вбито мутацій | Score |',
      '| --- | --- | --- | --- | --- |'
    ]
    for (const row of rows) {
      lines.push(`| ${row.area} | ... |`)
    }
    lines.push(`| **Разом** | ... |`)

    const allSurvived = rows.flatMap(r => r.survived ?? [])
    if (allSurvived.length > 0) {
      lines.push('', `## Рекомендації (${allSurvived.length} вижилих мутантів)`, '')
      const byFile = Map.groupBy(allSurvived, m => m.file)
      for (const [file, mutants] of byFile) {
        lines.push(`### \`${file}\``, '')
        lines.push('| Рядок | Оригінал | Мутант | Тип |')
        lines.push('|-------|----------|--------|-----|')
        for (const m of mutants) {
          const orig = m.original.replace(/\|/g, '\\|').replace(/`/g, "'")
          const repl = m.replacement.replace(/\|/g, '\\|').replace(/`/g, "'")
          lines.push(`| ${m.line} | \`${orig}\` | \`${repl}\` | ${m.type} |`)
        }
        lines.push('')
      }
    }

    return lines.join('\n') + '\n'
  }
  ```
  - `Map.groupBy` доступний у Node 21+ / Bun 1.x (перевірено у bun 1.3.14)
  - Якщо `Map.groupBy` недоступний — замінити на `reduce` з об'єктом

- [ ] Перевірка: `bun test rules/test/coverage/tests/coverage.test.mjs` — всі поточні тести проходять

---

## Phase 4: Tests

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/rules/js-lint/coverage/tests/coverage.test.mjs`, додати unit-тест для `parseStrykerReport` з fixture:
  ```js
  import { readFileSync } from 'node:fs'
  // ... або використати internal функцію через мок

  // Новий describe-блок:
  describe('parseStrykerReport', () => {
    test('повертає survived мутанти з файлу і рядка', async () => {
      await withTmpCwd(async () => {
        // Створити src/foo.js з рядком `if (x === 1)`
        await Bun.write('src/foo.js', 'export function f(x) {\n  if (x === 1) return true\n}\n')
        const report = {
          files: {
            'src/foo.js': {
              mutants: [
                { id: '0', mutatorName: 'ConditionalExpression', replacement: 'false', status: 'Survived', location: { start: { line: 2, column: 2 }, end: { line: 2, column: 14 } } },
                { id: '1', mutatorName: 'EqualityOperator', replacement: 'x !== 1', status: 'Killed', location: { start: { line: 2, column: 6 }, end: { line: 2, column: 13 } } },
              ]
            }
          }
        }
        // parseStrykerReport не експортована — тестуємо через runStryker або окремий export
        // Варіант: зробити parseStrykerReport named export (без зміни поведінки)
      })
    })
  })
  ```
  - Якщо `parseStrykerReport` не експортована — додати `export` до її визначення (вона вже `function`, зробити `export function parseStrykerReport`)

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/tests/coverage.test.mjs`, додати тест для рекомендацій:
  ```js
  test('COVERAGE.md з розділом Рекомендації', () => {
    const rows = [
      {
        area: 'JS',
        coverage: { lines: { covered: 100, total: 120 }, functions: { covered: 50, total: 60 } },
        mutations: { killed: 80, total: 100 },
        survived: [
          { file: 'src/foo.js', line: 5, original: 'if (x === 1)', replacement: 'false', type: 'ConditionalExpression' }
        ]
      }
    ]
    const result = renderMarkdown(rows)
    expect(result).toContain('## Рекомендації (1 вижилих мутантів)')
    expect(result).toContain('### `src/foo.js`')
    expect(result).toContain('| 5 |')
    expect(result).toContain('ConditionalExpression')
  })
  ```

- [ ] Запустити: `bun test rules/test/coverage/tests/ rules/js-lint/coverage/tests/` — всі тести проходять, 0 fail

---

## Phase 5: `--fix` CLI mode

- [ ] Створити `/Users/vitaliytv/www/nitra/cursor/npm/scripts/coverage-fix.mjs`:
  ```js
  import { readFileSync } from 'node:fs'
  import { join } from 'node:path'
  import { query } from '@anthropic-ai/claude-code'

  export async function fixSurvivedMutants(survived, projectRoot) {
    if (survived.length === 0) {
      console.log('✓ Всі мутанти вбиті — доповнення тестів не потрібне')
      return
    }
    const prompt = buildFixPrompt(survived, projectRoot)
    console.log(`\n🤖 coverage --fix: запускаю агента для ${survived.length} вижилих мутантів...\n`)
    for await (const msg of query({
      prompt,
      options: {
        cwd: projectRoot,
        maxTurns: 20,
        allowedTools: ['Read', 'Edit', 'Bash'],
      }
    })) {
      if (msg.type === 'text') process.stdout.write(msg.text)
    }
  }

  function buildFixPrompt(survived, projectRoot) {
    const byFile = {}
    for (const m of survived) {
      if (!byFile[m.file]) byFile[m.file] = []
      byFile[m.file].push(m)
    }
    const sections = []
    for (const [file, mutants] of Object.entries(byFile)) {
      const src = readFileSync(join(projectRoot, file), 'utf8').split('\n')
      const excerpts = mutants.map(m => {
        const startLine = Math.max(0, m.line - 3)
        const endLine = Math.min(src.length, m.line + 2)
        const context = src.slice(startLine, endLine).map((l, i) => `${startLine + i + 1}: ${l}`).join('\n')
        return `  - Рядок ${m.line}, тип ${m.type}: оригінал \`${m.original}\`, вижив варіант \`${m.replacement}\`\n    Контекст:\n    \`\`\`\n${context}\n    \`\`\``
      }).join('\n')
      sections.push(`### ${file}\n${excerpts}`)
    }
    return [
      'Твоє завдання — написати unit-тести що вбивають наступні вижилі мутанти Stryker.',
      'Для кожного мутанта: знайди або створи відповідний test-файл, додай тест-кейс що перевіряє саме цю гілку логіки.',
      'Після написання тестів — запусти `bun test` щоб переконатись що вони проходять.',
      '',
      '## Вижилі мутанти',
      '',
      ...sections,
      '',
      'Правила:',
      '- Не змінюй source-файли, тільки test-файли',
      '- Один мутант = один або кілька нових `test()` або `expect()` викликів',
      '- Запусти `bun test` в кінці і переконайся що 0 fail',
    ].join('\n')
  }
  ```

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/rules/test/coverage/coverage.mjs`, змінити `runCoverage`:
  - Сигнатура: `export async function runCoverage(configPath, opts = {})`
  - Після запису COVERAGE.md, якщо `opts.fix === true`:
    ```js
    if (opts.fix) {
      const allSurvived = rows.flatMap(r => r.survived ?? [])
      const { fixSurvivedMutants } = await import('../../scripts/coverage-fix.mjs')
      await fixSurvivedMutants(allSurvived, dirname(configPath))
      // Повторний прогін без --fix
      await runCoverage(configPath)
    }
    ```
  - Додати `import { dirname } from 'node:path'` якщо ще не імпортований (вже є)

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/bin/n-cursor.js`, у блоці `else if (cmd === 'coverage')`:
  ```js
  } else if (cmd === 'coverage') {
    const configPath = join(process.cwd(), '.n-cursor.json')
    const fix = args.includes('--fix')
    runCoverage(configPath, { fix })
  }
  ```

---

## Phase 6: Dependencies та версія

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/package.json`:
  - Додати до `"dependencies"` (або `"devDependencies"` якщо CLI-only): `"@anthropic-ai/claude-code": "latest"`
  - Змінити `"version"` з `"1.19.2"` → `"1.20.0"`

- [ ] У `/Users/vitaliytv/www/nitra/cursor/npm/CHANGELOG.md`, додати запис `## [1.20.0] - 2026-05-25`:
  ```
  ### Added
  - **`coverage --fix`**: новий режим що запускає Claude Code агента для написання тестів по вижилих мутантах Stryker, після чого повторно валідує coverage
  - **`COVERAGE.md`**: розділ `## Рекомендації` з таблицею вижилих мутантів (file, line, original, replacement, type) — завжди якщо є вижилі
  - **`stryker.config.mjs` шаблон**: `incremental: true` — відновлення прогону після переривання

  ### Changed
  - `js-lint` coverage provider: `collect()` тепер повертає `survived: [{file, line, original, replacement, type}]`
  ```

- [ ] `bun install` у `npm/` щоб встановити `@anthropic-ai/claude-code`

---

## Phase 7: mlmail оновлення

- [ ] `bun install` у `mlmail/` після публікації `@nitra/cursor@1.20.0`:
  ```bash
  # Після npm publish у cursor/npm/
  cd /Users/vitaliytv/www/vitaliytv/mlmail
  bun update @nitra/cursor
  ```
  - Або тимчасово: `"@nitra/cursor": "file:../nitra/cursor/npm"` для локального тестування

- [ ] Кінцева перевірка: `bun run coverage` у mlmail → COVERAGE.md містить `## Рекомендації`; `bun run coverage -- --fix` → агент пише тести → coverage оновлюється

---

## Hardening checklist

- [x] Всі file paths перевірені проти реального коду
- [x] `parseStrykerReport` — існуюча функція, сигнатура розширюється (не breaking)
- [x] `renderMarkdown` — існуюча функція, `rows` без `survived` поле — `r.survived ?? []` → пустий масив, рекомендації не рендеряться
- [x] `runCoverage` — другий параметр `opts = {}` — backwards compatible
- [x] `Map.groupBy` — доступний у Bun 1.3.14 (ECMAScript 2024)
- [x] `@anthropic-ai/claude-code` auth через Claude Code credentials — не потребує ANTHROPIC_API_KEY окремо
- [x] Incremental file у `.gitignore` — не потрапить у репо
- [x] Тести покривають нові гілки у `parseStrykerReport` і `renderMarkdown`
