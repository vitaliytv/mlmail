# Design: /n-coverage-fix skill (v2)

Refinement of 2026-05-25-n-coverage-fix-design.md based on clarified requirements.

## Overview

Autonomous skill `/n-coverage-fix` — runs coverage, reads Stryker survived mutants, writes test cases directly into existing test files (all files in one pass), then verifies with `bun test` + `n-cursor coverage`. Loops until no improvement (convergence) or no survivors.

**Key constraint:** No streaming API available — agent receives only SKILL.md + project files + mutation.json and must produce correct tests in one LLM pass per source file. The skill must provide `file:line` hints so the agent can navigate to mutation locations without searching.

**Scope v1:** JS mutants only (Stryker `mutation.json`). Rust — v2.

## Architecture

Skill lives in `@nitra/cursor` → `npm/skills/coverage-fix/SKILL.md` (directory already exists).  
Deployed to projects via `n-cursor fix` → `.cursor/skills/n-coverage-fix/SKILL.md`.

## Flow (max 3 iterations)

```
/n-coverage-fix
│
├── 1. n-cursor coverage                    ← withLock('coverage') — serialized
│      → COVERAGE.md (score_before)
│      → <jsRoot>/reports/stryker/mutation.json
│
├── [loop until convergence, max 3 iterations]
│   │
│   ├── 2. Parse mutation.json
│   │      Filter: status === 'Survived' | 'NoCoverage'
│   │      Group: Map<sourceFile → mutant[]>
│   │      Format each: { file, line, col, mutatorName, original, replacement }
│   │
│   ├── 3. For each sourceFile with survivors:
│   │      a. Locate testFile (see Test File Discovery below)
│   │      b. Read sourceFile — note file:line of every mutant
│   │      c. Read testFile — understand existing coverage
│   │      d. Write all new test cases into testFile at once
│   │         (one LLM pass per source file, all its mutants together)
│   │
│   ├── 4. bun test                         ← verify ALL files pass (not per-file)
│   │      if fail → diagnose which test broke → fix → retry once → if still fail: stop
│   │
│   ├── 5. n-cursor coverage                ← withLock('coverage')
│   │      → COVERAGE.md (score_after)
│   │
│   └── 6. Convergence check
│          score_after > score_before:
│            no survivors remaining → ✅ Success, print summary
│            survivors remain → score_before = score_after, go to step 2
│          score_after == score_before:
│            ❌ Report hard mutants (file:line for each), exit
│
└── max 3 iterations hit → same as "score unchanged" branch
```

## Writing Tests (LLM Instructions in SKILL.md)

For each source file, the agent receives:

```
Вижилі мутанти у src/i18n/auth-errors.js:

  19:1  ConditionalExpression  `kind === null || kind === undefined` → `false`
  19:1  LogicalOperator        `kind === null || kind === undefined` → `kind === null && kind === undefined`
  23:5  StringLiteral          `'token'` → `""`

Вихідний код src/i18n/auth-errors.js (рядки 15-30):
  <snippet>

Наявний тест-файл src/i18n/auth-errors.test.js:
  <full content>

Завдання: дописати нові тест-кейси в кінець describe-блоку, що:
  1. Проходять з оригінальним кодом
  2. Провалюються коли код змінено відповідно до кожної мутації вище

Для кожного мутанта: поясни (коментарем) яку умову він перевіряє, тоді пиши тест.
Пиши всі тести для цього файлу за один раз.
```

**Принцип якості:** skill явно інструктує — спочатку зрозуміти _що_ мутант перевіряє (яка гілка не тестується), потім написати тест що її покриває. Це уникає тестів які проходять але не вбивають мутант.

## Test File Discovery

Порядок пошуку для `src/foo/bar.js`:

1. Co-located: `src/foo/bar.test.js`
2. `__tests__` sibling: `src/foo/__tests__/bar.test.js`
3. Top-level `test/`: `test/foo/bar.test.js`

Якщо testFile відсутній — detect convention з існуючих файлів:

```
find src -name "*.test.*" | head -5
```

→ визначити pattern (co-located чи директорія) → створити новий файл за тим самим pattern.

## Parallel Runs Prevention

`n-cursor coverage` всередині захищений `withLock('coverage')` — паралельні CLI-виклики серіалізуються або дедуплікуються. Але skill цілком **не можна запускати паралельно** в різних агентах, тому що:

- Stryker пише `mutation.json` і `incremental.json` в одну директорію
- тест-файли редагуються одночасно — можливий data race

SKILL.md матиме явне попередження аналогічне `n-lint`.

## Convergence and Hard Mutants Report

Якщо score не покращився після ітерації, вивести:

```
/n-coverage-fix: покращення не досягнуто. Складні мутанти або dead code:

  src/i18n/auth-errors.js:19:1  ConditionalExpression — можливо dead code або guard без branch
  src/services/auth-store.js:4:3  BooleanLiteral — потребує mock Tauri invoke API

Mutation score залишився: 67.61%
```

## Error Handling

| Ситуація                            | Поведінка                                                         |
| ----------------------------------- | ----------------------------------------------------------------- |
| `mutation.json` не знайдено         | Повідомити: "спочатку запусти `bun run coverage`"; abort          |
| testFile не знайдено                | Detect convention → створити новий файл                           |
| `bun test` fail після retry         | Stop — не запускати coverage; повідомити який test-файл зламано   |
| `n-cursor coverage` killed (SIGURG) | `incremental.json` зберігає прогрес → retry автоматично підхопить |
| Mutant у `node_modules`, `dist`     | Skip                                                              |

## Summary on Success

```
✅ /n-coverage-fix завершено після N ітерацій

Mutation score (JS): 67.61% → 84.50%
Тести дописано: auth-errors.js (+5), auth-store.js (+7)
Вижилих мутантів: 0
```

## Out of Scope (v1)

- Rust mutants (cargo-mutants format differs)
- Threshold-based stopping (convergence was chosen)
- Per-file `bun test` between writes (batch approach: write all → test once)
- Streaming/interactive agent protocol (not available in this environment)
