# Design: /n-coverage-fix skill

## Overview

Autonomous skill `n-coverage-fix` that runs coverage analysis, reads survived Stryker mutants, and iteratively writes tests until mutation score stops improving (convergence).

Scope v1: JS mutants only (Stryker `mutation.json`). Rust (`cargo-mutants`) is v2.

## Architecture

The skill lives in `@nitra/cursor` (canonical) and is deployed to `.cursor/skills/n-coverage-fix/SKILL.md` via `n-cursor fix`.

### Top-level flow (max 3 iterations)

```
/n-coverage-fix
  ↓
[loop, max 3 iterations]
  1. n-cursor coverage           → COVERAGE.md + reports/stryker/mutation.json
  2. parse mutation.json         → survived mutants, grouped by source file
  3. for each file (with survivors):
       read source file + test file
       write/extend tests to kill mutants
       bun test <testfile>       → verify pass (fix if needed, max 2 attempts)
  4. compare mutation score in COVERAGE.md
  5. if score improved → next iteration
     if converged (score unchanged) or max iterations hit → summary + stop
```

## LLM Prompt Format (per file)

For each source file with survived mutants, the agent receives:

```markdown
## Завдання: дописати тести для вбивства мутантів

**Файл:** src/i18n/auth-errors.js
**Тест-файл:** src/i18n/__tests__/auth-errors.test.js

**Вижилі мутанти** (Stryker не вбив — значить тестів не вистачає):

| # | Рядок | Тип | Оригінал | Мутація що вижила |
|---|-------|-----|----------|-------------------|
| 1 | 19 | ConditionalExpression | `kind === 'token'` | `false` |
| 2 | 19 | LogicalOperator | `kind === null || kind === undefined` | `kind === null && kind === undefined` |

**Вихідний код:**
[source file content]

**Існуючі тести:**
[test file content]

**Твоя задача:** дописати тест-кейси, які:
1. Проходять з оригінальним кодом
2. Провалюються якщо код змінено відповідно до колонки «Мутація що вижила»

Після написання — запусти `bun test <testfile>` щоб підтвердити.
```

## Convergence Detection

```
prev_score = score from initial COVERAGE.md (before first iteration)
after each iteration:
  new_score = JS mutation score from COVERAGE.md
  if new_score > prev_score: prev_score = new_score; continue
  else: stop
```

Max iterations: 3.

## Test File Discovery

Priority order:
1. Co-located: `src/foo.js` → `src/foo.test.js`
2. `__tests__` sibling: `src/foo.js` → `src/__tests__/foo.test.js`
3. Top-level `test/`: `src/foo.js` → `test/foo.test.js`
4. Not found → create `src/foo.test.js` (new file)

## Error Handling

| Situation | Action |
|-----------|--------|
| Test file not found | Create new `<name>.test.js` adjacent to source |
| `bun test` fails after 2 fix attempts | Skip file, log in summary |
| `n-cursor coverage` killed (SIGURG/exit 144) | Retry once; if again → stop with warning "coverage killed by OS, incremental results saved" |
| All 3 iterations exhaust without convergence | Stop, print summary with remaining survivors |

## Summary Output

```
/n-coverage-fix завершено після N ітерацій.

Mutation score: 67.61% → 84.50% (JS)
Написано тестів: +12 (auth-errors.js +5, auth-store.js +7)

Залишилось вижилих мутантів: 8
  auth-store.js:4 — BooleanLiteral — потребує mock Tauri API
  ...
```

## Skill Location

- Canonical: `@nitra/cursor` package → `npm/skills/n-coverage-fix/SKILL.md`
- Deployed to projects: `.cursor/skills/n-coverage-fix/SKILL.md` via `n-cursor fix`

## Out of Scope (v1)

- Rust mutants (cargo-mutants output format differs, needs separate workflow)
- Threshold-based stopping (user chose convergence criterion)
- Parallel file processing (sequential is simpler and avoids test runner conflicts)
