# n-coverage-fix Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `n-coverage-fix` skill to `@nitra/cursor` that autonomously runs coverage, identifies survived Stryker mutants, writes targeted tests file-by-file, and iterates until score converges.

**Architecture:** Skill is a markdown instruction file (`npm/skills/n-coverage-fix/SKILL.md`) in `@nitra/cursor`, auto-deployed to projects with `js-lint` rule. It instructs Claude to: run `n-cursor coverage` → parse `reports/stryker/mutation.json` → write tests per file → re-run coverage until convergence (max 3 iterations).

**Tech Stack:** Bun, Stryker (`@stryker-mutator/core`), `mutation.json` format, `auto-skills.mjs` detection pattern.

---

## File Map

| File | Action | Purpose |
|------|--------|---------|
| `npm/skills/n-coverage-fix/SKILL.md` | Create | Skill instructions for Claude |
| `npm/skills/n-coverage-fix/auto.md` | Create | Auto-deploy condition: `[js-lint]` |
| `npm/scripts/tests/auto-skills.test.mjs` | Modify | Add `n-coverage-fix` to `ALL_SKILLS` + test for js-lint detection |
| `npm/CHANGELOG.md` | Modify | Add `[1.19.4]` entry |
| `npm/package.json` | Modify | Bump `1.19.3 → 1.19.4` |

> Note: Work happens in repo `/Users/vitaliytv/www/nitra/cursor`. Create a new git worktree `feat/n-coverage-fix` off `main` before starting.

---

### Task 1: Create worktree and verify baseline tests pass

**Files:**
- Worktree: `/Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix`

- [ ] **Step 1: Create the worktree**

```bash
git -C /Users/vitaliytv/www/nitra/cursor worktree add \
  .claude/worktrees/feat/n-coverage-fix \
  -b feat/n-coverage-fix
```

- [ ] **Step 2: Verify baseline tests pass**

```bash
cd /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix && \
bun test scripts/tests/auto-skills.test.mjs
```

Expected output: all tests pass (5 tests, 0 fail).

---

### Task 2: Write failing test for n-coverage-fix skill detection

**Files:**
- Modify: `npm/scripts/tests/auto-skills.test.mjs`

- [ ] **Step 1: Read the current test file**

```bash
cat /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix/npm/scripts/tests/auto-skills.test.mjs
```

- [ ] **Step 2: Update ALL_SKILLS and add js-lint test**

In `auto-skills.test.mjs`, update `ALL_SKILLS` to include `'n-coverage-fix'` and add a new test:

```js
const ALL_SKILLS = ['adr-normalize', 'fix', 'lint', 'llm-patch', 'n-coverage-fix', 'publish-telegram', 'taze']
```

Add after the existing tests:

```js
test('n-coverage-fix додається, коли правило js-lint виявлене', () => {
  const actual = detectAutoSkills({
    availableSkills: ALL_SKILLS,
    detectedRules: ['js-lint']
  })

  expect(actual.skills).toContain('n-coverage-fix')
})

test('n-coverage-fix НЕ додається без правила js-lint', () => {
  const actual = detectAutoSkills({
    availableSkills: ALL_SKILLS,
    detectedRules: []
  })

  expect(actual.skills).not.toContain('n-coverage-fix')
})
```

Also update the existing `ALL_SKILLS` tests — any test that uses `toEqual` on the full skills list must include `'n-coverage-fix'` at the right alphabetical position (between `'lint'` and `'publish-telegram'`). Update:

```js
// "завжди-додавані скіли" test — n-coverage-fix has js-lint condition, NOT always, so NOT in this result
// No change needed for that test.

// "повний набір: adr + bun" test — still no js-lint, so n-coverage-fix absent. No change.
```

- [ ] **Step 3: Run test to confirm it fails**

```bash
cd /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix && \
bun test npm/scripts/tests/auto-skills.test.mjs
```

Expected: FAIL — `n-coverage-fix` not found in skills directory yet.

---

### Task 3: Create the skill files

**Files:**
- Create: `npm/skills/n-coverage-fix/SKILL.md`
- Create: `npm/skills/n-coverage-fix/auto.md`

- [ ] **Step 1: Create auto.md**

File: `npm/skills/n-coverage-fix/auto.md`

```
[js-lint]
```

- [ ] **Step 2: Create SKILL.md**

File: `npm/skills/n-coverage-fix/SKILL.md`

```markdown
---
name: n-coverage-fix
description: >-
  Запустити coverage → дописати тести для вижилих мутантів → повторювати до конвергенції
---

# n-coverage-fix — автоматичне покращення coverage через mutation testing

## Мета

Запустити `n-cursor coverage`, прочитати вижилих мутантів із `reports/stryker/mutation.json`,
для кожного вихідного файлу дописати тести, що вбивають цих мутантів, і повторювати до
конвергенції (score не покращується між ітераціями).

Scope v1: JS мутанти (Stryker). Rust — v2.

## Передумови

- CWD — корінь репозиторію (або директорія де є `reports/stryker/mutation.json` після coverage).
- `bun run coverage` доступний.
- `bun test` запускає тести.

## Workflow

### Крок 1: baseline coverage

```bash
bun run coverage 2>&1
```

Зчитай JS mutation score з `COVERAGE.md` (колонка "Score"), запам'ятай як `baseline_score`.

Якщо coverage впав через SIGURG/exit 144 — спробуй ще раз. Якщо знову — зупинись:
"coverage killed by OS, incremental results saved".

### Крок 2: парсинг вижилих мутантів

```bash
node -e "
const r=JSON.parse(require('fs').readFileSync('reports/stryker/mutation.json','utf8'));
const g={};
for(const [f,d] of Object.entries(r.files))
  for(const m of d.mutants)
    if(m.status==='Survived'){
      g[f]??=[];
      g[f].push({line:m.location.start.line,type:m.mutatorName,rep:m.replacement,orig:m.original});
    }
console.log(JSON.stringify(g,null,2));
"
```

Якщо `g` порожній — зупинись, score вже 100%.

### Крок 3: дописати тести (file-by-file)

Для кожного файлу з вижилими мутантами:

**3a. Знайди тест-файл** (пріоритет):
1. Co-located: `src/foo.js` → `src/foo.test.js`
2. `__tests__` сусід: `src/foo.js` → `src/__tests__/foo.test.js`
3. Top-level `test/`: → `test/foo.test.js`
4. Не знайдено → створи `src/foo.test.js`

**3b. Прочитай** вихідний файл і тест-файл.

**3c. Напиши тести** — правила для кожного типу мутації:
- `ConditionalExpression`: тестуй обидва шляхи (умова true і false)
- `LogicalOperator`: тестуй граничні значення кожного операнда окремо (null, undefined, false, '')
- `BooleanLiteral`: тестуй і true, і false як вхід/вихід
- `StringLiteral`: тестуй що результат не порожній рядок при валідному вході
- `ArithmeticOperator`: тестуй конкретні числові значення, не лише знак результату
- `EqualityOperator`: тестуй значення на межі рівності

Тест ПОВИНЕН:
1. Проходити з оригінальним кодом
2. Провалюватися якщо код замінений мутацією зі стовпця "rep"

**3d. Верифікуй**:
```bash
bun test <testfile> 2>&1
```
Якщо падає — виправ (максимум 2 спроби). Якщо після 2 спроб — пропусти файл, запиши у список "skipped".

### Крок 4: перевірка покращення

```bash
bun run coverage 2>&1
```

Прочитай новий JS mutation score з `COVERAGE.md`.

- `new_score > baseline_score` → `baseline_score = new_score`, повтори з Кроку 2 (загалом max 3 ітерації)
- `new_score == baseline_score` або ≥ 3 ітерацій → зупинись

### Крок 5: summary

```
/n-coverage-fix завершено після N ітерацій.

Mutation score: X% → Y% (JS)
Написано тестів: +N (<файл1> +N1, <файл2> +N2)
Пропущено файлів: <список або "немає">

Залишилось вижилих мутантів: M
  <файл>:<рядок> — <тип> — [причина: потребує E2E/mock/side-effect]
```
```

- [ ] **Step 3: Run test to confirm it passes**

```bash
cd /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix && \
bun test npm/scripts/tests/auto-skills.test.mjs
```

Expected: all tests pass including 2 new `n-coverage-fix` tests.

---

### Task 4: Run full test suite

**Files:** none (verification only)

- [ ] **Step 1: Run all tests**

```bash
cd /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix && \
bun test scripts/tests/ rules/
```

Expected: all tests pass, 0 fail.

---

### Task 5: Bump version and update CHANGELOG

**Files:**
- Modify: `npm/package.json` (version `1.19.3 → 1.19.4`)
- Modify: `npm/CHANGELOG.md` (prepend new section)

- [ ] **Step 1: Read current CHANGELOG header**

```bash
head -15 /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix/npm/CHANGELOG.md
```

- [ ] **Step 2: Update package.json version**

Change `"version": "1.19.3"` → `"version": "1.19.4"` in `npm/package.json`.

- [ ] **Step 3: Prepend CHANGELOG entry**

Add after the header block (after the `## [1.19.3]` section start):

```markdown
## [1.19.4] - 2026-05-25

### Added

- `n-coverage-fix` skill — запускає coverage, читає вижилих мутантів Stryker, дописує тести file-by-file до конвергенції (max 3 ітерації). Автодеплой для проєктів із правилом `js-lint`.
```

- [ ] **Step 4: Verify version consistency**

```bash
node -e "const p=require('./npm/package.json'); console.log(p.version)"
# should print: 1.19.4
head -10 npm/CHANGELOG.md
# should show [1.19.4] at top
```

---

### Task 6: Smoke test skill in mlmail

**Files:** none (verification in mlmail repo)

- [ ] **Step 1: Link local @nitra/cursor to mlmail for testing**

```bash
cd /Users/vitaliytv/www/nitra/cursor/.claude/worktrees/feat/n-coverage-fix/npm && \
bun link
```

```bash
cd /Users/vitaliytv/www/vitaliytv/mlmail && \
bun link @nitra/cursor
```

- [ ] **Step 2: Run n-cursor to verify skill deploys**

```bash
cd /Users/vitaliytv/www/vitaliytv/mlmail && \
bunx n-cursor check 2>&1 | grep -i coverage-fix
```

Expected: skill listed / recognized.

- [ ] **Step 3: Unlink local package**

```bash
cd /Users/vitaliytv/www/vitaliytv/mlmail && \
bun install @nitra/cursor
```

---

## Self-Review Checklist

After completing all tasks:
- [ ] `npm/skills/n-coverage-fix/SKILL.md` has valid frontmatter (name + description)
- [ ] `auto.md` contains exactly `[js-lint]`
- [ ] `auto-skills.test.mjs` has 2 new tests for n-coverage-fix
- [ ] All tests pass (`bun test scripts/tests/ rules/`)
- [ ] `package.json` version is `1.19.4`
- [ ] CHANGELOG has `[1.19.4]` entry
