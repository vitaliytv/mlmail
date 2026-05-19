# `docs:regen` — ADR-driven C4 documentation regenerator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Запустити `bun run docs:regen` і отримати 5 актуальних C4-файлів у `docs/ci4/` (`01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`), регенерованих LLM-ом з 26 clean ADR. Кожен ADR отримує sentinel-блок `**Опрацьовано**` з посиланнями на проекції, у які він потрапив. Стан tracking — `docs/ci4/manifest.json` (без хешів ADR, тільки `processed_at` + `projections`).

**Architecture:** ESM-скрипт `scripts/docs-regen.js` як entry point + модулі в `scripts/docs-regen/` (discover, marks, manifest, triggers, llm, projection, hash, lock, cli, log). Bun-native (`Bun.write`, `Bun.spawn`), Node std (`node:crypto`, `node:fs/promises`, `node:util.parseArgs`), дві зовнішні залежності: `globby`, `gray-matter`. LLM-виклик через CLI `claude` (fallback `cursor-agent`). Юніт-тести в `scripts/__tests__/docs-regen/` через `bun:test`. Bootstrap-шаблони — окремі `.prompt.md` файли в `scripts/docs-regen/default-templates/`, копіюються в `docs/ci4/_templates/` на першому запуску.

**Tech Stack:** Bun 1.3+, Node 24+, ESM, `globby`, `gray-matter`, `bun:test`, `claude` CLI або `cursor-agent` CLI.

**Spec:** [docs/superpowers/specs/2026-05-18-docs-regen-design.md](../specs/2026-05-18-docs-regen-design.md)

---

## File Structure

**Нові ESM-модулі (у `scripts/docs-regen/`):**

- Create: `scripts/docs-regen.js` — entry point. Парсить CLI args, оркеструє повний flow, логує підсумок.
- Create: `scripts/docs-regen/cli.js` — `parseCliArgs(argv)` через `node:util.parseArgs`.
- Create: `scripts/docs-regen/hash.js` — `sha256(content)`. Один експорт.
- Create: `scripts/docs-regen/marks.js` — `hasMark`, `stripMark`, `formatMark`, `applyMark`. Логіка sentinel-блоку «Опрацьовано».
- Create: `scripts/docs-regen/discover.js` — `discoverCleanAdrs(rootDir)`. Glob `docs/adr/**/*.md`, фільтрує `_inbox/**` і `session:`, повертає `[{ slug, path, body, hasMark, rawContent }]`. Кидає на slug-колізію.
- Create: `scripts/docs-regen/manifest.js` — `loadManifest`, `saveManifest`, `defaultManifest`. Deterministic JSON.
- Create: `scripts/docs-regen/triggers.js` — `detectTriggers({ adrs, manifest, rootDir, templateHashes, ruleHash })`. Повертає `{ unmarked, removed, rulesChanged, templatesChanged }`.
- Create: `scripts/docs-regen/llm.js` — `callLlm(prompt, opts)`, `parseLlmResponse(raw)`. Spawn `claude` → fallback `cursor-agent`.
- Create: `scripts/docs-regen/projection.js` — `regenerateProjection({ name, adrs, currentContent, templates, model })`. Складає prompt, кличе LLM, парсить, повертає `{ content, used_adrs, tokens }`.
- Create: `scripts/docs-regen/lock.js` — `acquireLock(path)`, `releaseLock(path)`. Атомарне створення pid-файлу через `fs.open(path, 'wx')`.
- Create: `scripts/docs-regen/log.js` — `Logger`-class або просто функції `info`, `warn`, `error`. Пише і в stdout, і в `docs/ci4/.regen.log`.
- Create: `scripts/docs-regen/templates.js` — `loadTemplates(rootDir)`, `bootstrapTemplates(rootDir)`, `templateHashes(rootDir)`. Працює з `docs/ci4/_templates/`.

**Bootstrap-шаблони (markdown-файли, дані):**

- Create: `scripts/docs-regen/default-templates/_global.prompt.md`
- Create: `scripts/docs-regen/default-templates/01-context.prompt.md`
- Create: `scripts/docs-regen/default-templates/02-containers.prompt.md`
- Create: `scripts/docs-regen/default-templates/03-components.prompt.md`
- Create: `scripts/docs-regen/default-templates/04-code.prompt.md`
- Create: `scripts/docs-regen/default-templates/decisions.prompt.md`

**Юніт-тести (`scripts/__tests__/docs-regen/`):**

- Create: `scripts/__tests__/docs-regen/hash.test.js`
- Create: `scripts/__tests__/docs-regen/marks.test.js`
- Create: `scripts/__tests__/docs-regen/discover.test.js`
- Create: `scripts/__tests__/docs-regen/manifest.test.js`
- Create: `scripts/__tests__/docs-regen/triggers.test.js`
- Create: `scripts/__tests__/docs-regen/llm.test.js`
- Create: `scripts/__tests__/docs-regen/lock.test.js`
- Create: `scripts/__tests__/docs-regen/cli.test.js`
- Create: `scripts/__tests__/docs-regen/fixtures/` — допоміжна папка з ADR-фікстурами для `discover.test.js` і `triggers.test.js`.

**Конфіги і інтеграція:**

- Modify: `package.json` (root) — додати `"docs:regen": "bun run scripts/docs-regen.js"` і `"test:scripts": "bun test scripts/__tests__"`. Додати залежності `globby`, `gray-matter`.
- Modify: `.gitignore` — додати `docs/ci4/.regen.log`, `.claude/hooks/.docs-regen.lock`.
- Create: `.cursor/skills/docs-regen/SKILL.md` — тонкий wrapper з описом і командою.

**Після рантайму першого регену (artifacts):**

- Created at runtime: `docs/ci4/manifest.json` — tracking-стан.
- Created at runtime: `docs/ci4/_templates/*.prompt.md` — 6 файлів, скопійовані з default-templates на першому запуску.
- Modified at runtime: `docs/ci4/01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md` — регенеровані LLM-ом.
- Modified at runtime: `docs/adr/*.md` (26 файлів) — кожен отримує блок `**Опрацьовано**` в кінці.

---

## Task 1: Setup репозиторію — конфіги, залежності, scaffolding

**Files:**

- Modify: `package.json`
- Modify: `.gitignore`
- Create directory: `scripts/docs-regen/`
- Create directory: `scripts/__tests__/docs-regen/`
- Create directory: `scripts/docs-regen/default-templates/`

**Why first:** усі наступні таски — створення файлів усередині цих директорій і використання `bun run docs:regen` / `bun run test:scripts` для запуску. Без цього TDD-цикл не запуститься.

- [ ] **Step 1: Verify git status clean**

Run: `git status`
Expected: `nothing to commit, working tree clean` (з можливим untracked файлом спеки).

- [ ] **Step 2: Створити директорії**

Run:

```bash
mkdir -p scripts/docs-regen/default-templates scripts/__tests__/docs-regen/fixtures
```

- [ ] **Step 3: Додати скрипти і залежності у `package.json`**

Modify `package.json` (root). Замінити поточний JSON-вміст так, щоб блоки `scripts` і `devDependencies` стали:

```json
{
  "name": "mlmail",
  "version": "0.1.0",
  "private": true,
  "workspaces": ["app"],
  "type": "module",
  "scripts": {
    "lint": "bun run lint-js && bun run lint-text && bun run lint-style && bun run lint-ga && bun run lint-image && oxfmt .",
    "lint-ga": "n-cursor lint-ga",
    "lint-image": "npx @nitra/minify-image --src=. --write",
    "lint-js": "bunx oxlint --fix && bunx eslint --fix . && bunx jscpd . && bunx knip --no-config-hints",
    "lint-style": "npx stylelint '**/*.{css,scss,vue}' --fix",
    "lint-text": "n-cursor lint-text",
    "docs:regen": "bun run scripts/docs-regen.js",
    "test:scripts": "bun test scripts/__tests__"
  },
  "devDependencies": {
    "@nitra/cspell-dict": "^2.1.0",
    "@nitra/cursor": "^1.11.17",
    "@nitra/eslint-config": "^3.9.2",
    "@nitra/stylelint-config": "^1.4.0",
    "globby": "^14.0.2",
    "gray-matter": "^4.0.3"
  },
  "stylelint": {
    "extends": "@nitra/stylelint-config"
  },
  "engines": {
    "bun": ">=1.3",
    "node": ">=24"
  }
}
```

- [ ] **Step 4: Установити залежності**

Run: `bun install`
Expected: `globby` і `gray-matter` зʼявляються у `bun.lock`; жодних peer-warnings, які блокують установку.

- [ ] **Step 5: Додати ignore-патерни у `.gitignore`**

Modify `.gitignore`. Додати наприкінці файлу (зберігаючи трейлінговий `\n`):

```gitignore

# docs:regen
docs/ci4/.regen.log
.claude/hooks/.docs-regen.lock
```

- [ ] **Step 6: Перевірити, що bun резолвить нові залежності**

Run: `bun -e "import { globby } from 'globby'; import matter from 'gray-matter'; console.log(typeof globby, typeof matter)"`
Expected: `function function`.

- [ ] **Step 7: Перевірити, що `test:scripts` працює без жодного тесту**

Run: `bun run test:scripts`
Expected: `0 pass, 0 fail` або повідомлення «no tests found», exit 0.

- [ ] **Step 8: Run `git status && git diff` і зупинка**

Run: `git status && git diff`
Очікувано: модифіковані `package.json`, `.gitignore`, `bun.lock`, нові порожні директорії (як untracked). Зупинись, дай юзеру переглянути.

---

## Task 2: `hash.js` — sha256-обгортка (TDD)

**Files:**

- Create: `scripts/docs-regen/hash.js`
- Create: `scripts/__tests__/docs-regen/hash.test.js`

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/hash.test.js`:

```js
import { describe, it, expect } from 'bun:test'
import { sha256 } from '../../docs-regen/hash.js'

describe('sha256', () => {
  it('returns sha256: prefixed hex for non-empty string', () => {
    const result = sha256('hello')
    expect(result).toBe('sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824')
  })

  it('handles empty string', () => {
    const result = sha256('')
    expect(result).toBe('sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855')
  })

  it('handles unicode (українська)', () => {
    const result = sha256('файловий стор токенів')
    expect(result).toMatch(/^sha256:[0-9a-f]{64}$/)
  })

  it('produces deterministic output for same input', () => {
    expect(sha256('test')).toBe(sha256('test'))
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/hash.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/hash.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/hash.js`:

```js
import { createHash } from 'node:crypto'

export function sha256(content) {
  return 'sha256:' + createHash('sha256').update(content).digest('hex')
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/hash.test.js`
Expected: `4 pass, 0 fail`.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 3: `marks.js` — формат і детекція sentinel-блоку «Опрацьовано» (TDD)

**Files:**

- Create: `scripts/docs-regen/marks.js`
- Create: `scripts/__tests__/docs-regen/marks.test.js`

**Контракт:**

- `hasMark(rawContent: string): boolean` — `true`, якщо в кінці файлу є sentinel-блок (останній `---` на окремому рядку, після якого йде рядок, що починається з `**Опрацьовано**`).
- `stripMark(rawContent: string): string` — повертає вміст без sentinel-блоку (якщо є). Якщо немає — повертає як є. Видаляє все від останнього sentinel `---` (з порожніми рядками довкола) до кінця файлу.
- `formatMark(date: string, projections: string[]): string` — повертає рядок мітки (без `---` префіксу). `projections` — масив імен (`['01-context', ...]`); якщо порожній — формат «жодної».
- `applyMark(rawContent: string, date: string, projections: string[]): string` — `stripMark` + новий блок.

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/marks.test.js`:

```js
import { describe, it, expect } from 'bun:test'
import { hasMark, stripMark, formatMark, applyMark } from '../../docs-regen/marks.js'

const BODY_PLAIN = `# Foo

Some content.

## Rationale

More content.
`

const BODY_WITH_RULE_IN_BODY = `# Foo

Some content.

---

More content after a horizontal rule that is NOT a mark.
`

const BODY_WITH_MARK = `# Foo

Some content.

---

**Опрацьовано** 2026-05-17. Проекції:
- [01-context](../ci4/01-context.md)
- [03-components](../ci4/03-components.md)
`

const BODY_WITH_EMPTY_MARK = `# Foo

Some content.

---

**Опрацьовано** 2026-05-17. Проекції: жодної.
`

describe('hasMark', () => {
  it('returns false for body without any sentinel block', () => {
    expect(hasMark(BODY_PLAIN)).toBe(false)
  })

  it('returns false for body with horizontal rule but no Опрацьовано-paragraph after', () => {
    expect(hasMark(BODY_WITH_RULE_IN_BODY)).toBe(false)
  })

  it('returns true for body with mark of projections', () => {
    expect(hasMark(BODY_WITH_MARK)).toBe(true)
  })

  it('returns true for body with "жодної" mark', () => {
    expect(hasMark(BODY_WITH_EMPTY_MARK)).toBe(true)
  })
})

describe('stripMark', () => {
  it('returns body unchanged when no mark present', () => {
    expect(stripMark(BODY_PLAIN)).toBe(BODY_PLAIN)
  })

  it('preserves horizontal rule inside body, removes only trailing mark', () => {
    const input = BODY_WITH_RULE_IN_BODY + '\n---\n\n**Опрацьовано** 2026-05-17. Проекції: жодної.\n'
    const result = stripMark(input)
    expect(result).toBe(BODY_WITH_RULE_IN_BODY)
  })

  it('strips multi-line mark', () => {
    expect(stripMark(BODY_WITH_MARK)).toBe(`# Foo

Some content.
`)
  })
})

describe('formatMark', () => {
  it('formats empty projections as "жодної"', () => {
    expect(formatMark('2026-05-18', [])).toBe('**Опрацьовано** 2026-05-18. Проекції: жодної.')
  })

  it('formats projections as bullet list with relative links', () => {
    const result = formatMark('2026-05-18', ['01-context', '03-components'])
    expect(result).toBe(
      '**Опрацьовано** 2026-05-18. Проекції:\n' +
      '- [01-context](../ci4/01-context.md)\n' +
      '- [03-components](../ci4/03-components.md)'
    )
  })
})

describe('applyMark', () => {
  it('appends mark when none present', () => {
    const result = applyMark(BODY_PLAIN, '2026-05-18', ['01-context'])
    expect(result.endsWith(
      '\n---\n\n**Опрацьовано** 2026-05-18. Проекції:\n- [01-context](../ci4/01-context.md)\n'
    )).toBe(true)
    expect(result.startsWith(BODY_PLAIN.trimEnd())).toBe(true)
  })

  it('replaces existing mark', () => {
    const result = applyMark(BODY_WITH_MARK, '2026-05-18', [])
    expect(result.endsWith('Проекції: жодної.\n')).toBe(true)
    expect(result).not.toContain('2026-05-17')
    expect(result).not.toContain('03-components')
  })

  it('preserves horizontal rule inside body when adding mark', () => {
    const result = applyMark(BODY_WITH_RULE_IN_BODY, '2026-05-18', ['04-code'])
    expect(result).toContain('More content after a horizontal rule that is NOT a mark.')
    expect(result.endsWith('- [04-code](../ci4/04-code.md)\n')).toBe(true)
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/marks.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/marks.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/marks.js`:

```js
const MARK_HEADER = '**Опрацьовано**'

/**
 * Detect sentinel mark block at the end of an ADR body.
 * A mark is: last `---` (horizontal rule on its own line),
 * followed (after optional blank line) by a paragraph that starts with `**Опрацьовано**`.
 */
export function hasMark(rawContent) {
  return findMarkStart(rawContent) !== -1
}

export function stripMark(rawContent) {
  const idx = findMarkStart(rawContent)
  if (idx === -1) return rawContent
  // idx points at the '\n---\n' separator. Trim trailing newline before separator too.
  return rawContent.slice(0, idx).replace(/\n+$/, '\n')
}

export function formatMark(date, projections) {
  if (projections.length === 0) {
    return `${MARK_HEADER} ${date}. Проекції: жодної.`
  }
  const lines = projections.map((p) => `- [${p}](../ci4/${p}.md)`)
  return `${MARK_HEADER} ${date}. Проекції:\n${lines.join('\n')}`
}

export function applyMark(rawContent, date, projections) {
  const stripped = stripMark(rawContent)
  const body = stripped.endsWith('\n') ? stripped : stripped + '\n'
  return body + '\n---\n\n' + formatMark(date, projections) + '\n'
}

function findMarkStart(rawContent) {
  // Scan from end. Find last line that is exactly `---`,
  // such that the next non-empty line starts with `**Опрацьовано**`.
  const lines = rawContent.split('\n')
  for (let i = lines.length - 1; i >= 0; i--) {
    if (lines[i].trim() !== '---') continue
    // Check next non-empty line
    let j = i + 1
    while (j < lines.length && lines[j].trim() === '') j++
    if (j < lines.length && lines[j].startsWith(MARK_HEADER)) {
      // Return index in rawContent at the start of `---` line (including preceding `\n`)
      // Count chars to start of line i
      let offset = 0
      for (let k = 0; k < i; k++) offset += lines[k].length + 1 // +1 for \n
      // Step back over preceding blank lines so stripped body has clean trailing
      while (offset > 0 && rawContent[offset - 1] === '\n') offset--
      return offset
    }
  }
  return -1
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/marks.test.js`
Expected: всі тести pass.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 4: `discover.js` — дискавері clean ADR (TDD з реальними fixtures)

**Files:**

- Create: `scripts/docs-regen/discover.js`
- Create: `scripts/__tests__/docs-regen/discover.test.js`
- Create: фікстури під `scripts/__tests__/docs-regen/fixtures/discover/`:
  - `docs/adr/clean.md` — clean ADR без frontmatter
  - `docs/adr/clean-status.md` — clean ADR з простим `**Status:**` блоком
  - `docs/adr/20260517-draft.md` — draft з `session:` frontmatter
  - `docs/adr/_inbox/old-draft.md` — старий draft в `_inbox/`
  - `docs/adr/with-mark.md` — clean ADR із sentinel-блоком наприкінці

**Контракт:** `discoverCleanAdrs(rootDir: string): Promise<Array<{ slug, path, body, rawContent, hasMark }>>`. Кидає `Error('ADR slug collision: …')` якщо є два файли з однаковим slug.

- [ ] **Step 1: Створити фікстури**

Create `scripts/__tests__/docs-regen/fixtures/discover/docs/adr/clean.md`:

```markdown
# Clean ADR без frontmatter

## Контекст

Просто текст.

## Рішення

Прийняли.
```

Create `scripts/__tests__/docs-regen/fixtures/discover/docs/adr/clean-status.md`:

```markdown
# Clean ADR зі статус-блоком

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

Текст.
```

Create `scripts/__tests__/docs-regen/fixtures/discover/docs/adr/20260517-draft.md`:

```markdown
---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-17T06:24:50+03:00
transcript: /tmp/fake.jsonl
---

Текст драфта.
```

Create `scripts/__tests__/docs-regen/fixtures/discover/docs/adr/_inbox/old-draft.md`:

```markdown
---
session: old-draft
---

Старий драфт.
```

Create `scripts/__tests__/docs-regen/fixtures/discover/docs/adr/with-mark.md`:

```markdown
# ADR з міткою

## Контекст

Текст.

---

**Опрацьовано** 2026-05-17. Проекції: жодної.
```

Run для перевірки: `ls scripts/__tests__/docs-regen/fixtures/discover/docs/adr/`
Expected: 4 файли + папка `_inbox`.

- [ ] **Step 2: Написати failing-тест**

Create `scripts/__tests__/docs-regen/discover.test.js`:

```js
import { describe, it, expect } from 'bun:test'
import { join } from 'node:path'
import { discoverCleanAdrs } from '../../docs-regen/discover.js'

const FIXTURE_ROOT = join(import.meta.dir, 'fixtures', 'discover')

describe('discoverCleanAdrs', () => {
  it('returns only clean ADRs, ignoring drafts and _inbox', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const slugs = adrs.map((a) => a.slug).sort()
    expect(slugs).toEqual(['clean', 'clean-status', 'with-mark'])
  })

  it('detects hasMark correctly per ADR', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const bySlug = Object.fromEntries(adrs.map((a) => [a.slug, a]))
    expect(bySlug['clean'].hasMark).toBe(false)
    expect(bySlug['clean-status'].hasMark).toBe(false)
    expect(bySlug['with-mark'].hasMark).toBe(true)
  })

  it('returns path relative to rootDir', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const paths = adrs.map((a) => a.path).sort()
    expect(paths).toEqual([
      'docs/adr/clean-status.md',
      'docs/adr/clean.md',
      'docs/adr/with-mark.md',
    ])
  })

  it('returns body and rawContent for each ADR', async () => {
    const adrs = await discoverCleanAdrs(FIXTURE_ROOT)
    const clean = adrs.find((a) => a.slug === 'clean')
    expect(clean.rawContent).toContain('Clean ADR без frontmatter')
    expect(clean.body).toContain('Clean ADR без frontmatter')
  })

  it('throws on slug collision', async () => {
    // Створимо другий ADR з тим самим іменем у підкаталозі.
    // Це робиться в підпапці docs/adr/sub/clean.md.
    // Для тесту: створити окремий FIXTURE_ROOT з колізією.
    const collisionRoot = join(import.meta.dir, 'fixtures', 'collision')
    // Очікуємо, що цей шлях існує (треба буде створити поруч з discover-фікстурами).
    await expect(discoverCleanAdrs(collisionRoot)).rejects.toThrow(/slug collision/)
  })
})
```

- [ ] **Step 3: Створити колізійні фікстури**

Create `scripts/__tests__/docs-regen/fixtures/collision/docs/adr/clean.md`:

```markdown
# Перший clean ADR.
```

Create `scripts/__tests__/docs-regen/fixtures/collision/docs/adr/sub/clean.md`:

```markdown
# Другий clean ADR з тим самим slug.
```

- [ ] **Step 4: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/discover.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/discover.js'`.

- [ ] **Step 5: Імплементація**

Create `scripts/docs-regen/discover.js`:

```js
import { globby } from 'globby'
import matter from 'gray-matter'
import { readFile } from 'node:fs/promises'
import { join, basename } from 'node:path'
import { hasMark } from './marks.js'

export async function discoverCleanAdrs(rootDir) {
  const paths = await globby(['docs/adr/**/*.md'], {
    cwd: rootDir,
    ignore: ['docs/adr/_inbox/**'],
  })

  const adrs = []
  const slugs = new Map() // slug -> path (for collision detection)

  for (const relPath of paths.sort()) {
    const absPath = join(rootDir, relPath)
    const rawContent = await readFile(absPath, 'utf8')
    const parsed = matter(rawContent)
    if (parsed.data && parsed.data.session) continue // draft

    const slug = basename(relPath, '.md')
    if (slugs.has(slug)) {
      throw new Error(
        `ADR slug collision: ${slug} (paths: ${slugs.get(slug)}, ${relPath})`
      )
    }
    slugs.set(slug, relPath)

    adrs.push({
      slug,
      path: relPath,
      body: parsed.content,
      rawContent,
      hasMark: hasMark(rawContent),
    })
  }
  return adrs
}
```

- [ ] **Step 6: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/discover.test.js`
Expected: всі тести pass.

- [ ] **Step 7: Run `git status && git diff` і зупинка**

---

## Task 5: `manifest.js` — load, save, default, deterministic JSON (TDD)

**Files:**

- Create: `scripts/docs-regen/manifest.js`
- Create: `scripts/__tests__/docs-regen/manifest.test.js`

**Контракт:**

- `defaultManifest(): object` — порожній маніфест зі схемою v1.
- `loadManifest(rootDir): Promise<object>` — читає `docs/ci4/manifest.json`; якщо відсутній — повертає `defaultManifest()`. На битий JSON кидає.
- `saveManifest(rootDir, manifest): Promise<void>` — атомарний запис через `.tmp` + rename, з deterministic key ordering.

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/manifest.test.js`:

```js
import { describe, it, expect, beforeEach, afterEach } from 'bun:test'
import { join } from 'node:path'
import { mkdir, rm, writeFile, readFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { defaultManifest, loadManifest, saveManifest } from '../../docs-regen/manifest.js'

let TMP

beforeEach(async () => {
  TMP = join(tmpdir(), `manifest-test-${Date.now()}-${Math.random().toString(36).slice(2)}`)
  await mkdir(join(TMP, 'docs', 'ci4'), { recursive: true })
})

afterEach(async () => {
  await rm(TMP, { recursive: true, force: true })
})

describe('defaultManifest', () => {
  it('has version 1 and empty adrs/projections/templates/rules', () => {
    const m = defaultManifest()
    expect(m.version).toBe(1)
    expect(m.adrs).toEqual({})
    expect(m.projections).toEqual({})
    expect(m.templates).toEqual({})
    expect(m.rules).toEqual({})
  })
})

describe('loadManifest', () => {
  it('returns defaultManifest when file is missing', async () => {
    const m = await loadManifest(TMP)
    expect(m.version).toBe(1)
    expect(m.adrs).toEqual({})
  })

  it('parses existing manifest', async () => {
    const stored = {
      version: 1,
      generated_at: '2026-05-18T16:00:00Z',
      adrs: { 'foo-adr': { path: 'docs/adr/foo-adr.md', processed_at: '2026-05-18', projections: ['01-context'] } },
      projections: {},
      templates: {},
      rules: {},
    }
    await writeFile(join(TMP, 'docs', 'ci4', 'manifest.json'), JSON.stringify(stored), 'utf8')
    const m = await loadManifest(TMP)
    expect(m.adrs['foo-adr'].processed_at).toBe('2026-05-18')
  })

  it('throws on invalid JSON', async () => {
    await writeFile(join(TMP, 'docs', 'ci4', 'manifest.json'), '{ not json', 'utf8')
    await expect(loadManifest(TMP)).rejects.toThrow()
  })
})

describe('saveManifest', () => {
  it('writes manifest with sorted keys and trailing newline', async () => {
    const m = {
      version: 1,
      tool: { name: 'docs-regen', version: '0.1.0' },
      adrs: {
        'z-adr': { path: 'docs/adr/z-adr.md', projections: [] },
        'a-adr': { path: 'docs/adr/a-adr.md', projections: ['01-context'] },
      },
      projections: {},
      templates: {},
      rules: {},
    }
    await saveManifest(TMP, m)
    const text = await readFile(join(TMP, 'docs', 'ci4', 'manifest.json'), 'utf8')
    expect(text.endsWith('\n')).toBe(true)
    // Перший ADR-ключ — лексикографічно перший
    const adrSection = text.match(/"adrs":\s*\{[\s\S]*?\}/)[0]
    expect(adrSection.indexOf('"a-adr"')).toBeLessThan(adrSection.indexOf('"z-adr"'))
  })

  it('round-trip: save then load gives equal structure', async () => {
    const m = {
      version: 1,
      generated_at: '2026-05-18T16:00:00Z',
      tool: { name: 'docs-regen', version: '0.1.0' },
      adrs: { foo: { path: 'docs/adr/foo.md', processed_at: '2026-05-18', projections: ['01-context'] } },
      projections: {
        '01-context': { path: 'docs/ci4/01-context.md', output_hash: 'sha256:abc', used_adrs: ['foo'] },
      },
      templates: { '_global.prompt.md': { hash: 'sha256:def' } },
      rules: { '.cursor/rules/n-ci4.mdc': { hash: 'sha256:ghi' } },
    }
    await saveManifest(TMP, m)
    const loaded = await loadManifest(TMP)
    expect(loaded).toEqual(m)
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/manifest.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/manifest.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/manifest.js`:

```js
import { readFile, writeFile, rename, mkdir } from 'node:fs/promises'
import { join, dirname } from 'node:path'

const MANIFEST_REL_PATH = 'docs/ci4/manifest.json'

export function defaultManifest() {
  return {
    version: 1,
    generated_at: null,
    tool: { name: 'docs-regen', version: '0.1.0', model: null },
    rules: {},
    templates: {},
    adrs: {},
    projections: {},
  }
}

export async function loadManifest(rootDir) {
  const path = join(rootDir, MANIFEST_REL_PATH)
  let text
  try {
    text = await readFile(path, 'utf8')
  } catch (e) {
    if (e.code === 'ENOENT') return defaultManifest()
    throw e
  }
  return JSON.parse(text)
}

export async function saveManifest(rootDir, manifest) {
  const path = join(rootDir, MANIFEST_REL_PATH)
  const tmpPath = path + '.tmp'
  await mkdir(dirname(path), { recursive: true })
  const sorted = sortKeysDeep(manifest)
  const json = JSON.stringify(sorted, null, 2) + '\n'
  await writeFile(tmpPath, json, 'utf8')
  await rename(tmpPath, path)
}

function sortKeysDeep(value) {
  if (Array.isArray(value)) return value.map(sortKeysDeep)
  if (value !== null && typeof value === 'object') {
    const out = {}
    for (const key of Object.keys(value).sort()) {
      out[key] = sortKeysDeep(value[key])
    }
    return out
  }
  return value
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/manifest.test.js`
Expected: всі тести pass.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 6: `triggers.js` — детект сигналів regen (TDD)

**Files:**

- Create: `scripts/docs-regen/triggers.js`
- Create: `scripts/__tests__/docs-regen/triggers.test.js`

**Контракт:** `detectTriggers({ adrs, manifest, ruleHash, templateHashes }): { unmarked, removed, rulesChanged, templatesChanged }`.

- `adrs` — масив `{ slug, hasMark, ... }`.
- `manifest` — обʼєкт із `adrs` (slug → metadata), `templates` (filename → { hash }), `rules` (path → { hash }).
- `ruleHash` — string, поточний хеш `.cursor/rules/n-ci4.mdc`.
- `templateHashes` — `{ filename → hash }`, поточні хеші шаблонів.

Виходи:

- `unmarked: string[]` — slug ADR без мітки.
- `removed: string[]` — slug у `manifest.adrs`, яких немає в `adrs`.
- `rulesChanged: boolean` — `manifest.rules['.cursor/rules/n-ci4.mdc']?.hash !== ruleHash`.
- `templatesChanged: boolean` — будь-який поточний хеш шаблону відрізняється від `manifest.templates[name]?.hash`.

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/triggers.test.js`:

```js
import { describe, it, expect } from 'bun:test'
import { detectTriggers } from '../../docs-regen/triggers.js'

describe('detectTriggers', () => {
  const baseManifest = () => ({
    version: 1,
    adrs: {
      foo: { path: 'docs/adr/foo.md', processed_at: '2026-05-18', projections: [] },
      bar: { path: 'docs/adr/bar.md', processed_at: '2026-05-18', projections: ['01-context'] },
    },
    templates: { '_global.prompt.md': { hash: 'sha256:t1' } },
    rules: { '.cursor/rules/n-ci4.mdc': { hash: 'sha256:r1' } },
    projections: {},
  })

  it('returns all empty when nothing changed', () => {
    const adrs = [
      { slug: 'foo', hasMark: true },
      { slug: 'bar', hasMark: true },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result).toEqual({
      unmarked: [],
      removed: [],
      rulesChanged: false,
      templatesChanged: false,
    })
  })

  it('detects unmarked ADRs', () => {
    const adrs = [
      { slug: 'foo', hasMark: false },
      { slug: 'bar', hasMark: true },
      { slug: 'new-adr', hasMark: false },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result.unmarked.sort()).toEqual(['foo', 'new-adr'])
    expect(result.removed).toEqual([])
  })

  it('detects removed ADRs', () => {
    const adrs = [{ slug: 'foo', hasMark: true }]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result.removed).toEqual(['bar'])
  })

  it('detects rule hash drift', () => {
    const adrs = [
      { slug: 'foo', hasMark: true },
      { slug: 'bar', hasMark: true },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r2-NEW',
      templateHashes: { '_global.prompt.md': 'sha256:t1' },
    })
    expect(result.rulesChanged).toBe(true)
  })

  it('detects template hash drift', () => {
    const adrs = [
      { slug: 'foo', hasMark: true },
      { slug: 'bar', hasMark: true },
    ]
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: { '_global.prompt.md': 'sha256:t2-NEW' },
    })
    expect(result.templatesChanged).toBe(true)
  })

  it('detects added template (not in manifest)', () => {
    const adrs = []
    const result = detectTriggers({
      adrs,
      manifest: baseManifest(),
      ruleHash: 'sha256:r1',
      templateHashes: {
        '_global.prompt.md': 'sha256:t1',
        '01-context.prompt.md': 'sha256:newone',
      },
    })
    expect(result.templatesChanged).toBe(true)
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/triggers.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/triggers.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/triggers.js`:

```js
const RULE_FILE = '.cursor/rules/n-ci4.mdc'

export function detectTriggers({ adrs, manifest, ruleHash, templateHashes }) {
  const unmarked = adrs.filter((a) => !a.hasMark).map((a) => a.slug)

  const currentSlugs = new Set(adrs.map((a) => a.slug))
  const removed = Object.keys(manifest.adrs || {}).filter((s) => !currentSlugs.has(s))

  const rulesChanged = (manifest.rules?.[RULE_FILE]?.hash ?? null) !== ruleHash

  let templatesChanged = false
  const manifestTpl = manifest.templates || {}
  for (const [name, hash] of Object.entries(templateHashes)) {
    if ((manifestTpl[name]?.hash ?? null) !== hash) {
      templatesChanged = true
      break
    }
  }
  if (!templatesChanged) {
    // Перевірка зворотного напрямку — шаблон видалили
    for (const name of Object.keys(manifestTpl)) {
      if (!(name in templateHashes)) {
        templatesChanged = true
        break
      }
    }
  }

  return { unmarked, removed, rulesChanged, templatesChanged }
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/triggers.test.js`
Expected: всі тести pass.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 7: `llm.js` — парсер LLM-відповіді (TDD) + Bun.spawn-обгортка

**Files:**

- Create: `scripts/docs-regen/llm.js`
- Create: `scripts/__tests__/docs-regen/llm.test.js`

**Контракт:**

- `parseLlmResponse(raw: string): { content: string, used_adrs: string[] }` — парсить рядок (можливо обгорнутий у markdown fence) у валідний обʼєкт. Кидає `Error` з описом, якщо неможливо.
- `callLlm(prompt: string, opts?: { model?: string }): Promise<string>` — спавнить `claude -p --model <m>`; fallback `cursor-agent -p --mode ask --output-format text --model <m>`. Передає `prompt` через stdin, повертає stdout. Кидає `Error('No LLM CLI available')` якщо ні один не знайдений.

Юніт-тест покриває **тільки** `parseLlmResponse`. `callLlm` — інтеграційно у smoke-тесті (Task 13).

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/llm.test.js`:

```js
import { describe, it, expect } from 'bun:test'
import { parseLlmResponse } from '../../docs-regen/llm.js'

describe('parseLlmResponse', () => {
  it('parses raw JSON', () => {
    const raw = '{"content":"# Hello","used_adrs":["foo","bar"]}'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('# Hello')
    expect(result.used_adrs).toEqual(['foo', 'bar'])
  })

  it('parses JSON wrapped in markdown fence ```json', () => {
    const raw = '```json\n{"content":"# Hi","used_adrs":[]}\n```'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('# Hi')
    expect(result.used_adrs).toEqual([])
  })

  it('parses JSON wrapped in bare ``` fence', () => {
    const raw = '```\n{"content":"x","used_adrs":["a"]}\n```'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('x')
  })

  it('strips surrounding prose and finds JSON object', () => {
    const raw = 'Here is the JSON:\n\n{"content":"x","used_adrs":[]}\n\nThanks.'
    const result = parseLlmResponse(raw)
    expect(result.content).toBe('x')
  })

  it('throws on broken JSON', () => {
    expect(() => parseLlmResponse('{ not json')).toThrow(/parse/i)
  })

  it('throws when content field missing', () => {
    expect(() => parseLlmResponse('{"used_adrs":[]}')).toThrow(/content/)
  })

  it('throws when used_adrs missing', () => {
    expect(() => parseLlmResponse('{"content":"x"}')).toThrow(/used_adrs/)
  })

  it('throws when used_adrs is not an array', () => {
    expect(() => parseLlmResponse('{"content":"x","used_adrs":"foo"}')).toThrow(/used_adrs/)
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/llm.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/llm.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/llm.js`:

```js
import { Readable } from 'node:stream'

export function parseLlmResponse(raw) {
  let text = raw.trim()
  // Strip markdown fences: ```json ... ``` or ``` ... ```
  const fenceMatch = text.match(/^```(?:json)?\s*\n([\s\S]*?)\n```\s*$/m)
  if (fenceMatch) {
    text = fenceMatch[1].trim()
  }
  // If still has surrounding prose, find first { ... } block
  if (!text.startsWith('{')) {
    const objMatch = text.match(/\{[\s\S]*\}/)
    if (objMatch) text = objMatch[0]
  }

  let parsed
  try {
    parsed = JSON.parse(text)
  } catch (e) {
    throw new Error(`Failed to parse LLM response as JSON: ${e.message}`)
  }

  if (typeof parsed.content !== 'string') {
    throw new Error('LLM response missing required field "content" (string)')
  }
  if (!Array.isArray(parsed.used_adrs)) {
    throw new Error('LLM response missing required field "used_adrs" (array)')
  }
  for (const item of parsed.used_adrs) {
    if (typeof item !== 'string') {
      throw new Error('LLM response field "used_adrs" must contain only strings')
    }
  }
  return { content: parsed.content, used_adrs: parsed.used_adrs }
}

const CLI_CANDIDATES = [
  {
    name: 'claude',
    args: (model) => ['-p', '--model', model],
    defaultModel: 'sonnet',
  },
  {
    name: 'cursor-agent',
    args: (model) => ['-p', '--mode', 'ask', '--output-format', 'text', '--model', model],
    defaultModel: 'claude-4.6-sonnet-medium',
  },
]

export async function callLlm(prompt, opts = {}) {
  const errors = []
  for (const candidate of CLI_CANDIDATES) {
    const cli = candidate.name
    const model = opts.model ?? candidate.defaultModel
    const args = candidate.args(model)
    try {
      const proc = Bun.spawn([cli, ...args], {
        stdin: Readable.from([prompt]),
        stdout: 'pipe',
        stderr: 'pipe',
      })
      const stdout = await new Response(proc.stdout).text()
      const exit = await proc.exited
      if (exit !== 0) {
        const stderr = await new Response(proc.stderr).text()
        errors.push(`${cli} exited ${exit}: ${stderr.trim()}`)
        continue
      }
      return stdout
    } catch (e) {
      if (e.code === 'ENOENT') {
        errors.push(`${cli} not found in PATH`)
        continue
      }
      throw e
    }
  }
  throw new Error('No LLM CLI available:\n' + errors.join('\n'))
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/llm.test.js`
Expected: всі тести pass.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 8: `lock.js` — атомарний lock через `fs.open(path, 'wx')` (TDD)

**Files:**

- Create: `scripts/docs-regen/lock.js`
- Create: `scripts/__tests__/docs-regen/lock.test.js`

**Контракт:**

- `acquireLock(path: string): Promise<{ acquired: boolean, release?: () => Promise<void> }>` — пробує створити lock-файл; якщо існує — `acquired: false`. Якщо успішно — повертає `release`, що видаляє файл.

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/lock.test.js`:

```js
import { describe, it, expect, beforeEach, afterEach } from 'bun:test'
import { join } from 'node:path'
import { mkdir, rm, access } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { acquireLock } from '../../docs-regen/lock.js'

let LOCK_DIR

beforeEach(async () => {
  LOCK_DIR = join(tmpdir(), `lock-test-${Date.now()}-${Math.random().toString(36).slice(2)}`)
  await mkdir(LOCK_DIR, { recursive: true })
})

afterEach(async () => {
  await rm(LOCK_DIR, { recursive: true, force: true })
})

describe('acquireLock', () => {
  it('creates lock file and returns release fn', async () => {
    const path = join(LOCK_DIR, 'a.lock')
    const lock = await acquireLock(path)
    expect(lock.acquired).toBe(true)
    expect(typeof lock.release).toBe('function')
    // file exists
    await expect(access(path)).resolves.toBeUndefined()
    await lock.release()
    // file gone
    await expect(access(path)).rejects.toThrow()
  })

  it('returns acquired:false when lock exists', async () => {
    const path = join(LOCK_DIR, 'b.lock')
    const first = await acquireLock(path)
    expect(first.acquired).toBe(true)
    const second = await acquireLock(path)
    expect(second.acquired).toBe(false)
    await first.release()
  })

  it('writes pid into lock file', async () => {
    const path = join(LOCK_DIR, 'c.lock')
    const lock = await acquireLock(path)
    const text = await Bun.file(path).text()
    expect(text).toBe(String(process.pid))
    await lock.release()
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/lock.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/lock.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/lock.js`:

```js
import { open, unlink, mkdir } from 'node:fs/promises'
import { dirname } from 'node:path'

export async function acquireLock(path) {
  await mkdir(dirname(path), { recursive: true })
  let handle
  try {
    handle = await open(path, 'wx')
  } catch (e) {
    if (e.code === 'EEXIST') return { acquired: false }
    throw e
  }
  await handle.write(String(process.pid))
  await handle.close()
  return {
    acquired: true,
    release: async () => {
      await unlink(path).catch((e) => {
        if (e.code !== 'ENOENT') throw e
      })
    },
  }
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/lock.test.js`
Expected: всі тести pass.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 9: `cli.js` — парсер аргументів через `node:util.parseArgs` (TDD)

**Files:**

- Create: `scripts/docs-regen/cli.js`
- Create: `scripts/__tests__/docs-regen/cli.test.js`

**Контракт:** `parseCliArgs(argv: string[]): { projection: string|undefined, all: boolean, dry: boolean, noMark: boolean, check: boolean }`. Кидає на невідомі флаги.

- [ ] **Step 1: Написати failing-тест**

Create `scripts/__tests__/docs-regen/cli.test.js`:

```js
import { describe, it, expect } from 'bun:test'
import { parseCliArgs } from '../../docs-regen/cli.js'

describe('parseCliArgs', () => {
  it('defaults: all false, projection undefined', () => {
    const result = parseCliArgs([])
    expect(result.all).toBe(false)
    expect(result.dry).toBe(false)
    expect(result.noMark).toBe(false)
    expect(result.check).toBe(false)
    expect(result.projection).toBeUndefined()
  })

  it('--projection 01-context', () => {
    const result = parseCliArgs(['--projection', '01-context'])
    expect(result.projection).toBe('01-context')
  })

  it('--all sets all=true', () => {
    expect(parseCliArgs(['--all']).all).toBe(true)
  })

  it('--dry sets dry=true', () => {
    expect(parseCliArgs(['--dry']).dry).toBe(true)
  })

  it('--no-mark sets noMark=true', () => {
    expect(parseCliArgs(['--no-mark']).noMark).toBe(true)
  })

  it('--check sets check=true', () => {
    expect(parseCliArgs(['--check']).check).toBe(true)
  })

  it('throws on unknown flag', () => {
    expect(() => parseCliArgs(['--bogus'])).toThrow()
  })
})
```

- [ ] **Step 2: Запустити тест, переконатися, що падає**

Run: `bun test scripts/__tests__/docs-regen/cli.test.js`
Expected: FAIL з `Cannot find module '../../docs-regen/cli.js'`.

- [ ] **Step 3: Імплементація**

Create `scripts/docs-regen/cli.js`:

```js
import { parseArgs } from 'node:util'

export function parseCliArgs(argv) {
  const { values } = parseArgs({
    args: argv,
    options: {
      projection: { type: 'string' },
      all: { type: 'boolean', default: false },
      dry: { type: 'boolean', default: false },
      'no-mark': { type: 'boolean', default: false },
      check: { type: 'boolean', default: false },
    },
    strict: true,
    allowPositionals: false,
  })
  return {
    projection: values.projection,
    all: values.all,
    dry: values.dry,
    noMark: values['no-mark'],
    check: values.check,
  }
}
```

- [ ] **Step 4: Запустити тест, переконатися, що проходить**

Run: `bun test scripts/__tests__/docs-regen/cli.test.js`
Expected: всі тести pass.

- [ ] **Step 5: Run `git status && git diff` і зупинка**

---

## Task 10: `templates.js` — bootstrap і завантаження шаблонів

**Files:**

- Create: `scripts/docs-regen/templates.js`
- (тести інтегруються через smoke; окремий unit-test не пишемо, бо логіка — copy-or-read)

**Контракт:**

- `TEMPLATE_NAMES: string[]` — масив імен файлів шаблонів.
- `bootstrapTemplates(rootDir, defaultDir): Promise<{ created: string[] }>` — якщо у `docs/ci4/_templates/` відсутній якийсь файл, копіює з `defaultDir`. Повертає список створених імен.
- `loadTemplates(rootDir): Promise<Record<string, string>>` — читає `docs/ci4/_templates/*.prompt.md`, повертає `{ name: content }`.
- `templateHashes(rootDir): Promise<Record<string, string>>` — для кожного шаблону повертає sha256.

- [ ] **Step 1: Імплементація**

Create `scripts/docs-regen/templates.js`:

```js
import { mkdir, readFile, writeFile, access } from 'node:fs/promises'
import { join } from 'node:path'
import { sha256 } from './hash.js'

export const TEMPLATE_NAMES = [
  '_global.prompt.md',
  '01-context.prompt.md',
  '02-containers.prompt.md',
  '03-components.prompt.md',
  '04-code.prompt.md',
  'decisions.prompt.md',
]

const TEMPLATE_DIR_REL = 'docs/ci4/_templates'

export async function bootstrapTemplates(rootDir, defaultDir) {
  const targetDir = join(rootDir, TEMPLATE_DIR_REL)
  await mkdir(targetDir, { recursive: true })
  const created = []
  for (const name of TEMPLATE_NAMES) {
    const target = join(targetDir, name)
    try {
      await access(target)
    } catch (e) {
      if (e.code !== 'ENOENT') throw e
      const source = join(defaultDir, name)
      const text = await readFile(source, 'utf8')
      await writeFile(target, text, 'utf8')
      created.push(name)
    }
  }
  return { created }
}

export async function loadTemplates(rootDir) {
  const dir = join(rootDir, TEMPLATE_DIR_REL)
  const out = {}
  for (const name of TEMPLATE_NAMES) {
    out[name] = await readFile(join(dir, name), 'utf8')
  }
  return out
}

export async function templateHashes(rootDir) {
  const all = await loadTemplates(rootDir)
  const out = {}
  for (const [name, content] of Object.entries(all)) {
    out[name] = sha256(content)
  }
  return out
}
```

- [ ] **Step 2: Run `git status && git diff` і зупинка**

---

## Task 11: Default templates (`scripts/docs-regen/default-templates/*.prompt.md`)

**Files:**

- Create: `scripts/docs-regen/default-templates/_global.prompt.md`
- Create: `scripts/docs-regen/default-templates/01-context.prompt.md`
- Create: `scripts/docs-regen/default-templates/02-containers.prompt.md`
- Create: `scripts/docs-regen/default-templates/03-components.prompt.md`
- Create: `scripts/docs-regen/default-templates/04-code.prompt.md`
- Create: `scripts/docs-regen/default-templates/decisions.prompt.md`

Це **дані**, не код. Без тестів.

- [ ] **Step 1: `_global.prompt.md`**

Create `scripts/docs-regen/default-templates/_global.prompt.md`:

````markdown
# Глобальні правила для всіх C4-проекцій MLMaiL

Ці інваріанти беруться з `.cursor/rules/n-ci4.mdc`. LLM має їх дотримуватись у кожному `content`-полі відповіді.

## Чистий Markdown

Жодних HTML-тегів (`<div>`, `<span>`, `<br>`, `<!-- -->` тощо), жодних CSS-класів, навігаційних обгорток або стилів. Тільки нативний Markdown: заголовки, списки, таблиці, code-fences, blockquote.

## Контекстна незалежність розділів

Кожен розділ і абзац у регенерованому файлі MLMaiL повинен мати сенс **без сусіднього тексту**. Заборонені формулювання:

- «як було згадано вище», «вище», «нижче»;
- «цей сервіс», «той самий компонент», «згаданий контейнер»;
- «попередній метод», «див. розділ X».

Замість цього — повторювати назву сутності щоразу:

- «контейнер `tauri-backend` MLMaiL» замість «він»;
- «функція `summarize_email()` MLMaiL» замість «ця функція»;
- «зовнішня система `Gmail API`» замість «цей сервіс».

## Трасування для недетермінованих компонентів

Якщо компонент MLMaiL виконує LLM-виклик, евристику, ML-інференс — обов'язкова секція з посиланням на трасування. Поки інфраструктури трасування немає — пиши `TBD: tracing-storage`.

## Зв'язок із тестами

Кожен компонент MLMaiL у проекціях `03-components.md` і `04-code.md` — з посиланням на тести (`app/src-tauri/tests/<name>.rs`, `app/src/__tests__/<name>.test.js`). Якщо тестів ще немає — `TBD: tests`.

## Мова

- Українська для бізнес-мови, описів аудиторії, призначень.
- Англійська для технічних ідентифікаторів (імена змінних, функцій, файлів, JSON keys).
- Українська назва компонента + англійський ідентифікатор у дужках: «компонент `auth-store` MLMaiL».

## Поточний стан vs цільова архітектура

Регенерована документація MLMaiL описує **цільову** архітектуру. Кожна проекція має секцію `Поточний стан` з підрозділами `Реалізовано` і `Planned`. Якщо ADR описує рішення зі статусом `planned` — компонент іде в `Planned`. Якщо implementing-код вже існує згідно з ADR — `Реалізовано`.

## Формат відповіді

Поверни рівно один JSON-обʼєкт. Без markdown-fence, без преамбули, без коментарів:

```
{
  "content": "<повний markdown файлу проекції>",
  "used_adrs": ["<slug-1>", "<slug-2>", ...]
}
```

- `content` — повний markdown файлу проекції. Не зливай з попередньою версією — згенеруй з нуля на основі ADR і поточного стану як reference.
- `used_adrs` — масив slug-ів ADR, які реально цитовані або описані в `content`. Slug береться з заголовка `### ADR: <slug>` у вхідних даних. Один ADR може зʼявитись у кількох проекціях.
````

- [ ] **Step 2: `01-context.prompt.md`**

Create `scripts/docs-regen/default-templates/01-context.prompt.md`:

```markdown
# Інструкції LLM для генерації `docs/ci4/01-context.md`

## Аудиторія

Менеджер MLMaiL і інженер MLMaiL. Перший знайомиться з продуктом, другий — освіжує контекст перед роботою з кодом.

## Призначення файлу

`docs/ci4/01-context.md` — System Context (C4 рівень 1) застосунку MLMaiL. Відповідає на питання: «Що робить MLMaiL? Хто ним користується? З якими зовнішніми системами він спілкується?».

## Обов'язкові секції

1. **Призначення MLMaiL** — 1-2 абзаци бізнес-мовою. Що робить застосунок, для кого, яку проблему вирішує. Без технічних деталей.

2. **Користувачі MLMaiL** — хто (один користувач — власник Gmail-акаунту), які цілі (зменшити час на email, чути саммері, швидко вирішувати листи).

3. **Зовнішні системи MLMaiL.** Для кожної — підзаголовок третього рівня:
   - Назва зовнішньої системи (наприклад «Google Identity»).
   - Що вона надає MLMaiL.
   - Які межі довіри і scopes/permissions.
   - Які операції MLMaiL виконує проти неї.

   Обов'язкові зовнішні системи (якщо релевантні ADR є):
   - Google Identity (OAuth 2.0).
   - Gmail API.
   - LLM-провайдер (для саммері).
   - TTS-провайдер (для озвучення саммері).

4. **Use-cases MLMaiL** — features рівня системи:
   - Авторизація через Google.
   - Отримання листів з Gmail.
   - AI-саммері листа.
   - Озвучення саммері.
   - Дії над листом: видалити / видалити з фільтром / зберегти у `home/` нотатки / зберегти у `work/` нотатки.
   - Чернетка відповіді з пропозицією.

5. **Cross-cutting concerns** — приватність даних користувача MLMaiL, місце зберігання токенів, межі довіри, мова інтерфейсу (українська).

6. **Поточний стан MLMaiL.** Підрозділи `Реалізовано` (що вже є за ADR-ами зі статусом accepted і existing code) і `Planned` (що тільки описано в ADR як майбутнє).

## Інваріанти

- Жодних HTML-тегів.
- Кожен заголовок самодостатній: «Зовнішня система `Gmail API` MLMaiL», не «Gmail API» окремо.
- Use-cases описуються як «Користувач MLMaiL <дія>», не «MLMaiL <дія>».
- Якщо ADR суперечить попередньому файлу `docs/ci4/01-context.md` — пріоритет за ADR-ом.

## Формат виводу

Дотримуйся загального формату з `_global.prompt.md`: JSON `{ "content": "...", "used_adrs": [...] }`.
```

- [ ] **Step 3: `02-containers.prompt.md`**

Create `scripts/docs-regen/default-templates/02-containers.prompt.md`:

```markdown
# Інструкції LLM для генерації `docs/ci4/02-containers.md`

## Аудиторія

Інженер MLMaiL (першочергово). Менеджер MLMaiL — оглядово, щоб зрозуміти deployable units.

## Призначення файлу

`docs/ci4/02-containers.md` — Containers (C4 рівень 2) застосунку MLMaiL. Відповідає на питання: «З яких виконуваних одиниць складається MLMaiL? Що кожна володіє? Як вони спілкуються?».

## Обов'язкові секції

1. **Список контейнерів MLMaiL.** Для кожного контейнера — підзаголовок другого рівня з назвою (наприклад «Контейнер `vue-frontend` MLMaiL»):
   - **Технологія** — повна назва стека (наприклад «Vue 3.5 + Quasar 2 + Vite 8»).
   - **Відповідальність** — 1-2 речення.
   - **Дані** — якими даними володіє контейнер MLMaiL (data-model тут). Наприклад: локальне сховище нотаток MLMaiL володіє `.md`-файлами у `home/` і `work/`.
   - **Інтерфейси** — з якими іншими контейнерами MLMaiL і зовнішніми системами спілкується, у якому напрямку.
   - **Розгортання** — macOS app bundle, Android APK, чи окремий процес.

2. **Спільна конфігурація і секрети MLMaiL** — env vars, файли `.env`, OAuth client IDs, межі довіри між контейнерами.

3. **Поточний стан MLMaiL.** Підрозділи `Реалізовано` і `Planned`.

## Інваріанти

- Кожен контейнер MLMaiL називається повним іменем (`vue-frontend`, `tauri-backend`, `notes-store`). Жодних «фронт»/«бек» без прикметника.
- Напрямки залежностей вказувати явно: «контейнер `vue-frontend` MLMaiL викликає Tauri-команди контейнера `tauri-backend`», не «фронт говорить з беком».
- Data-model — там, де контейнер володіє даними (notes у `notes-store`, OAuth-токени у `tauri-backend`).

## Формат виводу

Дотримуйся `_global.prompt.md`.
```

- [ ] **Step 4: `03-components.prompt.md`**

Create `scripts/docs-regen/default-templates/03-components.prompt.md`:

```markdown
# Інструкції LLM для генерації `docs/ci4/03-components.md`

## Аудиторія

Інженер MLMaiL перед роботою над фічею.

## Призначення файлу

`docs/ci4/03-components.md` — Components (C4 рівень 3) застосунку MLMaiL. Відповідає на питання: «Які логічні компоненти живуть всередині кожного контейнера MLMaiL? Що кожен робить? Як перевірити, що працює?».

## Обов'язкові секції

Для кожного контейнера MLMaiL — підзаголовок другого рівня. Усередині — підзаголовки третього рівня для кожного компонента. Структура одного компонента:

1. **Назва компонента MLMaiL** (заголовок третього рівня): «Компонент `gmail-client` MLMaiL».
2. **Відповідальність** — 1-2 речення, самодостатньо.
3. **Залежності** — bullet-list:
   - входи: які компоненти або контейнери MLMaiL викликають цей компонент;
   - виходи: які компоненти, контейнери або зовнішні системи цей компонент викликає.
4. **Тести** — посилання у форматі `app/src-tauri/tests/<name>.rs` або `app/src/__tests__/<name>.test.js`. Якщо тестів немає — `TBD: tests`.
5. **Трасування** — для недетермінованих компонентів MLMaiL (LLM-summary, TTS, евристики) — `TBD: tracing-storage` або реальне посилання на дашборд, якщо існує.
6. **Релевантні ADR** — bullet-list зі slug-ами ADR (без `.md`), які описують це рішення.

Орієнтовний перелік компонентів MLMaiL (LLM сам уточнить на основі ADR):

- У контейнері `tauri-backend` MLMaiL: `oauth-store`, `gmail-client`, `summary-engine`, `tts-engine`, `notes-store`, `action-handler`, `endpoints`, `tauri-commands`.
- У контейнері `vue-frontend` MLMaiL: `auth-store`, `inbox-view`, `summary-view`, `notes-view`, `quasar-components`, `i18n`.

## Інваріанти

- Кожен компонент описаний так, щоб його опис мав сенс без сусідніх компонентів.
- Тести обов'язкові: або реальне посилання, або `TBD: tests`.
- Трасування — для будь-якого недетермінованого компонента.

## Формат виводу

Дотримуйся `_global.prompt.md`.
```

- [ ] **Step 5: `04-code.prompt.md`**

Create `scripts/docs-regen/default-templates/04-code.prompt.md`:

```markdown
# Інструкції LLM для генерації `docs/ci4/04-code.md`

## Аудиторія

Інженер MLMaiL, який пише код.

## Призначення файлу

`docs/ci4/04-code.md` — Code (C4 рівень 4) застосунку MLMaiL. Конкретні файли, функції, конфігурація і операції.

## Обов'язкові секції

1. **Tauri-команди MLMaiL** — таблиця або bullet-list. Для кожної команди:
   - Назва (наприклад `gmail_inbox_count`).
   - Сигнатура Rust (повний return type).
   - Файл (`app/src-tauri/src/...`).
   - Короткий опис відповідальності.

2. **Vue-компоненти MLMaiL** ключових екранів — таблиця або bullet-list:
   - Назва (`Login.vue`, `Inbox.vue`).
   - Props.
   - Файл (`app/src/views/...` або `app/src/components/...`).
   - Які stores використовує.

3. **Конфігурація MLMaiL**:
   - `app/src-tauri/tauri.conf.json` — ключові секції.
   - Env vars (`app/src-tauri/.env`, `.env.example`).
   - OAuth client IDs, scopes.
   - Quasar variables (`app/src/quasar-variables.sass`).

4. **Operations MLMaiL** — Build, run, deploy:
   - Локальний dev (`bun run tauri dev`, `bun run android`).
   - Збірка macOS app bundle.
   - Збірка Android APK.
   - Lint, тести (`bun run lint`, `bun test`, `cargo test`).

## Інваріанти

- Усі шляхи — абсолютні відносно репо.
- Кожен запис самодостатній.

## Формат виводу

Дотримуйся `_global.prompt.md`.
```

- [ ] **Step 6: `decisions.prompt.md`**

Create `scripts/docs-regen/default-templates/decisions.prompt.md`:

```markdown
# Інструкції LLM для генерації `docs/ci4/decisions.md`

## Аудиторія

Менеджер MLMaiL і інженер MLMaiL. Перший — щоб бачити історію рішень; другий — щоб знайти, який ADR сформував конкретний компонент.

## Призначення файлу

`docs/ci4/decisions.md` — зведення ADR-впливів на C4-модель MLMaiL.

## Обов'язкові секції

1. **Хронологічний індекс ADR MLMaiL.** Таблиця: `slug | дата | статус | summary (1 рядок)`.

2. **Вплив ADR на рівні C4 MLMaiL.** Підзаголовок другого рівня. Таблиця або bullet-list: для кожного ADR — масив назв проекцій (`01-context`, `02-containers`, `03-components`, `04-code`), на які цей ADR вплинув.

3. **Зворотний індекс.** Підзаголовок другого рівня. Для кожного рівня C4 MLMaiL — bullet-list slug-ів ADR, які його сформували.

4. **Superseded chains** — якщо є. Підзаголовок другого рівня. Формат: `A → B → C`.

## Інваріанти

- Кожен запис індексу — самостійний (slug + дата + статус + однорядковий summary).
- Не дублювати тіло ADR — лише посилатися (`[<slug>](../adr/<slug>.md)`).
- Якщо ADR немає в `used_adrs` жодної з 4 проекцій (`01..04`) — все одно включити в індекс цього файлу. `decisions.md` тримає **усі** clean ADR як індекс.

## Формат виводу

Дотримуйся `_global.prompt.md`. `used_adrs` у відповіді для `decisions` має містити **всі** slug-и, що зʼявилися в індексі (фактично — всі clean ADR).
```

- [ ] **Step 7: Run `git status && git diff` і зупинка**

---

## Task 12: `projection.js` — orchestration одного projection-build

**Files:**

- Create: `scripts/docs-regen/projection.js`

**Контракт:** `regenerateProjection({ name, adrs, currentContent, templates, model }): Promise<{ content, used_adrs, prompt_length, output_length }>`.

Без юніт-тесту: тестується інтегрально у smoke (Task 14). LLM-виклик — реальний.

- [ ] **Step 1: Імплементація**

Create `scripts/docs-regen/projection.js`:

```js
import { callLlm, parseLlmResponse } from './llm.js'

export async function regenerateProjection({
  name,
  adrs,
  currentContent,
  templates,
  model,
}) {
  const prompt = buildPrompt({ name, adrs, currentContent, templates })
  const raw = await callLlm(prompt, { model })
  const parsed = parseLlmResponse(raw)
  return {
    content: parsed.content,
    used_adrs: parsed.used_adrs,
    prompt_length: prompt.length,
    output_length: raw.length,
  }
}

function buildPrompt({ name, adrs, currentContent, templates }) {
  const globalRules = templates['_global.prompt.md']
  const projectionTemplate = templates[`${name}.prompt.md`]

  const adrSection = adrs
    .map(
      (a) =>
        `### ADR: ${a.slug}\n\n${stripExistingMark(a.body)}`
    )
    .join('\n\n')

  return [
    '# Глобальні правила оформлення',
    '',
    globalRules,
    '',
    '---',
    '',
    `# Інструкції для проекції: ${name}`,
    '',
    projectionTemplate,
    '',
    '---',
    '',
    '# Поточний вміст файлу проекції (для consistency, не дублюй сліпо)',
    '',
    '```markdown',
    currentContent || '(файл порожній — створи з нуля на основі ADR)',
    '```',
    '',
    '---',
    '',
    `# ADR MLMaiL (${adrs.length} clean, повним body)`,
    '',
    adrSection,
    '',
    '---',
    '',
    '# Інструкція до відповіді',
    '',
    'Поверни рівно один JSON-обʼєкт без markdown-fence, без преамбули:',
    '```',
    '{ "content": "<повний markdown файлу>", "used_adrs": ["<slug>", ...] }',
    '```',
    '',
    `Файл, який ти генеруєш: docs/ci4/${name}.md. Зроби його повним, самодостатнім, готовим до коміту.`,
  ].join('\n')
}

function stripExistingMark(body) {
  // Видалити блок мітки, щоб не плутати LLM. Знаходимо останній `---\n\n**Опрацьовано**...`
  const idx = body.lastIndexOf('\n---\n\n**Опрацьовано**')
  if (idx === -1) {
    const altIdx = body.lastIndexOf('\n---\n**Опрацьовано**')
    if (altIdx === -1) return body
    return body.slice(0, altIdx).trimEnd()
  }
  return body.slice(0, idx).trimEnd()
}
```

- [ ] **Step 2: Run `git status && git diff` і зупинка**

---

## Task 13: `log.js` — простий логер у stdout + `.regen.log`

**Files:**

- Create: `scripts/docs-regen/log.js`

**Контракт:** клас `Logger(rootDir)`:

- `info(msg)`, `warn(msg)`, `error(msg)` — друк у stdout/stderr.
- `flush()` — записує накопичений buffer у `docs/ci4/.regen.log` (gitignored).

- [ ] **Step 1: Імплементація**

Create `scripts/docs-regen/log.js`:

```js
import { appendFile, mkdir } from 'node:fs/promises'
import { join, dirname } from 'node:path'

export class Logger {
  constructor(rootDir) {
    this.rootDir = rootDir
    this.buffer = []
  }

  info(msg) {
    const line = `[info ] ${new Date().toISOString()} ${msg}`
    console.log(line)
    this.buffer.push(line)
  }

  warn(msg) {
    const line = `[warn ] ${new Date().toISOString()} ${msg}`
    console.warn(line)
    this.buffer.push(line)
  }

  error(msg) {
    const line = `[error] ${new Date().toISOString()} ${msg}`
    console.error(line)
    this.buffer.push(line)
  }

  async flush() {
    const path = join(this.rootDir, 'docs/ci4/.regen.log')
    await mkdir(dirname(path), { recursive: true })
    await appendFile(path, this.buffer.join('\n') + '\n', 'utf8')
    this.buffer = []
  }
}
```

- [ ] **Step 2: Run `git status && git diff` і зупинка**

---

## Task 14: `docs-regen.js` — головна точка входу і повний flow

**Files:**

- Create: `scripts/docs-regen.js`

Цей файл звʼязує все докупи. Без юніт-тесту: інтеграційний smoke у Task 15.

- [ ] **Step 1: Імплементація**

Create `scripts/docs-regen.js`:

```js
#!/usr/bin/env bun
import { readFile, writeFile, stat } from 'node:fs/promises'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { parseCliArgs } from './docs-regen/cli.js'
import { discoverCleanAdrs } from './docs-regen/discover.js'
import { loadManifest, saveManifest, defaultManifest } from './docs-regen/manifest.js'
import { detectTriggers } from './docs-regen/triggers.js'
import { applyMark } from './docs-regen/marks.js'
import { acquireLock } from './docs-regen/lock.js'
import { Logger } from './docs-regen/log.js'
import { regenerateProjection } from './docs-regen/projection.js'
import { sha256 } from './docs-regen/hash.js'
import {
  TEMPLATE_NAMES,
  bootstrapTemplates,
  loadTemplates,
  templateHashes,
} from './docs-regen/templates.js'

const ROOT_DIR = process.cwd()
const RULE_FILE = '.cursor/rules/n-ci4.mdc'
const PROJECTIONS = ['01-context', '02-containers', '03-components', '04-code', 'decisions']
const LOCK_PATH = '.claude/hooks/.docs-regen.lock'
const TOOL_VERSION = '0.1.0'

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

async function run(args, logger) {
  // 1. Sanity: репо у merge/rebase?
  if (await isInMergeOrRebase()) {
    logger.warn('Repository is in merge/rebase state, aborting')
    return 0
  }

  // 2. Default templates dir (від where сам скрипт лежить)
  const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url))
  const DEFAULT_TEMPLATE_DIR = join(SCRIPT_DIR, 'docs-regen', 'default-templates')

  // 3. Bootstrap templates
  const { created } = await bootstrapTemplates(ROOT_DIR, DEFAULT_TEMPLATE_DIR)
  if (created.length > 0) {
    logger.info(`Templates bootstrapped: ${created.join(', ')}`)
  }

  // 4. Discover clean ADRs
  const adrs = await discoverCleanAdrs(ROOT_DIR)
  logger.info(`Clean ADRs found: ${adrs.length}`)

  // 5. Load manifest
  const manifest = await loadManifest(ROOT_DIR)

  // 6. Compute current hashes
  const tplHashes = await templateHashes(ROOT_DIR)
  const ruleHash = await fileHash(ROOT_DIR, RULE_FILE)

  // 7. Detect triggers
  const triggers = detectTriggers({
    adrs,
    manifest,
    ruleHash,
    templateHashes: tplHashes,
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

  // 8. --check mode
  if (args.check) {
    if (!needRegen) {
      logger.info('docs:regen --check: in sync')
      return 0
    } else {
      logger.error('docs:regen --check: drift detected, run `bun run docs:regen`')
      return 1
    }
  }

  if (!needRegen) {
    logger.info('OK, nothing to regenerate')
    return 0
  }

  // 9. Filter projections (--projection)
  const projectionsToRun = args.projection
    ? [args.projection].filter((p) => PROJECTIONS.includes(p))
    : PROJECTIONS
  if (args.projection && projectionsToRun.length === 0) {
    logger.error(`Unknown projection: ${args.projection}`)
    return 2
  }

  // 10. Pre-run sanity report
  logger.info(
    `Will regenerate ${projectionsToRun.length} projection(s): ${projectionsToRun.join(', ')}`
  )
  if (args.dry) {
    logger.info('--dry: stopping before LLM calls')
    return 0
  }

  // 11. Grace period
  await sleep(3000)

  // 12. Load templates
  const templates = await loadTemplates(ROOT_DIR)

  // 13. Generate each projection
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
      model: process.env.DOCS_REGEN_MODEL,
    })
    await writeFile(currentPath, result.content, 'utf8')
    projectionResults[name] = {
      path: `docs/ci4/${name}.md`,
      output_hash: sha256(result.content),
      generated_at: new Date().toISOString(),
      used_adrs: result.used_adrs,
      prompt_length: result.prompt_length,
      output_length: result.output_length,
    }
    logger.info(`  → wrote, ${result.used_adrs.length} ADRs used`)
  }

  // 14. Aggregate used_adrs by slug
  const adrToProjections = new Map()
  for (const [name, r] of Object.entries(projectionResults)) {
    for (const slug of r.used_adrs) {
      if (!adrToProjections.has(slug)) adrToProjections.set(slug, new Set())
      adrToProjections.get(slug).add(name)
    }
  }

  // 15. Update marks in ADRs
  if (!args.noMark) {
    const today = isoDate()
    for (const adr of adrs) {
      const projectionsUsed = [...(adrToProjections.get(adr.slug) ?? [])].sort()
      const updated = applyMark(adr.rawContent, today, projectionsUsed)
      if (updated !== adr.rawContent) {
        await writeFile(join(ROOT_DIR, adr.path), updated, 'utf8')
      }
    }
    logger.info(`Marks updated: ${adrs.length} ADRs`)
  } else {
    logger.info('Marks skipped (--no-mark)')
  }

  // 16. Update manifest
  const today = isoDate()
  const newManifest = {
    version: 1,
    generated_at: new Date().toISOString(),
    tool: {
      name: 'docs-regen',
      version: TOOL_VERSION,
      model: process.env.DOCS_REGEN_MODEL || 'sonnet',
    },
    rules: { [RULE_FILE]: { hash: ruleHash } },
    templates: Object.fromEntries(
      Object.entries(tplHashes).map(([n, h]) => [n, { hash: h }])
    ),
    adrs: Object.fromEntries(
      adrs.map((a) => [
        a.slug,
        {
          path: a.path,
          processed_at: today,
          projections: [...(adrToProjections.get(a.slug) ?? [])].sort(),
        },
      ])
    ),
    projections: Object.fromEntries(
      // зберігаємо ВСІ projections — для тих, що не регенерувались, копіюємо з попереднього manifest
      PROJECTIONS.map((name) => [
        name,
        projectionResults[name] ?? manifest.projections?.[name] ?? null,
      ])
    ),
  }
  // Викинути null значення з projections (якщо проекція ніколи не регенерувалась)
  for (const [name, val] of Object.entries(newManifest.projections)) {
    if (val === null) delete newManifest.projections[name]
  }

  await saveManifest(ROOT_DIR, newManifest)
  logger.info('Manifest updated')

  return 0
}

async function isInMergeOrRebase() {
  for (const file of ['.git/MERGE_HEAD', '.git/rebase-merge', '.git/rebase-apply']) {
    try {
      await stat(join(ROOT_DIR, file))
      return true
    } catch (e) {
      if (e.code !== 'ENOENT') throw e
    }
  }
  return false
}

async function fileHash(rootDir, relPath) {
  const content = await readFile(join(rootDir, relPath), 'utf8').catch(() => '')
  return sha256(content)
}

function isoDate() {
  return new Date().toISOString().slice(0, 10)
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

const exitCode = await main()
process.exit(exitCode)
```

- [ ] **Step 2: Перевірити, що скрипт парситься і не падає на запуску `--dry`**

Run: `bun run docs:regen --dry`
Expected behavior:
- скрипт читає `docs/adr/`, знаходить 26 clean ADR;
- bootstrap-ить 6 шаблонів у `docs/ci4/_templates/` (якщо їх ще нема);
- виводить «Will regenerate 5 projection(s): ..., --dry: stopping before LLM calls»;
- exit 0;
- НЕ викликає LLM;
- НЕ пише нічого в `docs/ci4/01..04.md`, `decisions.md`, manifest.json.

Після запуску — перевірити: `ls docs/ci4/_templates/` має 6 файлів.

- [ ] **Step 3: Run `git status && git diff` і зупинка**

Очікувано untracked: `docs/ci4/_templates/*.prompt.md` (6 файлів). Жодних модифікацій у `docs/ci4/01..04.md`, `decisions.md`, або `docs/adr/`.

---

## Task 15: Smoke-тест першого повного запуску

**Files:**

- Modify (LLM-driven): `docs/ci4/01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`
- Modify: усі 26 файлів у `docs/adr/*.md` (додавання мітки)
- Create: `docs/ci4/manifest.json`

**Передумова:** `claude` CLI у PATH, авторизований.

- [ ] **Step 1: Перевірити доступність CLI**

Run: `which claude && claude --version`
Expected: шлях до бінарю + версія. Якщо нема — `which cursor-agent && cursor-agent --version` як fallback. Якщо обох немає — інструмент не запрацює, smoke провалиться.

- [ ] **Step 2: Запустити повний цикл**

Run: `bun run docs:regen`
Expected:
- лог про знайдені 26 ADR;
- лог `Will regenerate 5 projection(s): 01-context, 02-containers, 03-components, 04-code, decisions`;
- 3-секундна пауза;
- 5 послідовних LLM-викликів (кожен з логом `→ wrote, N ADRs used`);
- лог `Marks updated: 26 ADRs`;
- лог `Manifest updated`;
- exit 0.

Час: орієнтовно 2-5 хвилин (5 LLM-запитів × ~30-60s кожен).

- [ ] **Step 3: Перевірити вихідні файли**

Run:

```bash
ls -la docs/ci4/
cat docs/ci4/manifest.json | head -30
tail -10 docs/adr/ADR-0006-google-oauth.md
```

Expected:
- `docs/ci4/manifest.json` створено, містить 26 ADR з `processed_at`, 5 проекцій з `output_hash`/`used_adrs`, хеші шаблонів і `n-ci4.mdc`.
- ADR `ADR-0006-google-oauth.md` має блок `**Опрацьовано** 2026-05-18. Проекції: ...` в кінці.
- 5 файлів у `docs/ci4/` оновлені (timestamp змінився).

- [ ] **Step 4: Перевірити lint регенерованих файлів**

Run: `bunx markdownlint-cli2 'docs/ci4/01-context.md' 'docs/ci4/02-containers.md' 'docs/ci4/03-components.md' 'docs/ci4/04-code.md' 'docs/ci4/decisions.md'`
Expected: 0 issues.

Якщо є issues:
- знайти, які саме правила порушені;
- вирішити, чи додати їх у `_global.prompt.md` як заборону для LLM, чи це баг шаблону;
- зафіксити, перезапустити `bun run docs:regen --all`.

- [ ] **Step 5: Перевірити idempotence**

Run: `bun run docs:regen` (повторно одразу)
Expected: лог `OK, nothing to regenerate`, exit 0. Жодних викликів LLM, жодних змін у файлах.

- [ ] **Step 6: Перевірити `--check`**

Видалити мітку з одного ADR руками:

Modify `docs/adr/ADR-0006-google-oauth.md` — видалити блок `**Опрацьовано** ...` (і `---` перед ним) у кінці файлу.

Run: `bun run docs:regen --check`
Expected: лог про drift, exit 1.

Відновити мітку: `bun run docs:regen` → exit 0.

- [ ] **Step 7: Перевірити `--projection`**

Run: `bun run docs:regen --all --projection 01-context`
Expected: лог про регенерацію тільки `01-context.md`, exit 0. Інші 4 файли не торкнуто.

- [ ] **Step 8: Run `git status && git diff` і зупинка**

Очікувано:
- модифіковані `docs/ci4/01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`;
- модифіковані 26 файлів у `docs/adr/`;
- новий `docs/ci4/manifest.json`;
- нові 6 файлів у `docs/ci4/_templates/`.

**Зупинись. Дай юзеру перевірити diff кожного файлу. Це найважливіша точка review всього проєкту.**

---

## Task 16: Slash-команда `.cursor/skills/docs-regen/SKILL.md`

**Files:**

- Create: `.cursor/skills/docs-regen/SKILL.md`
- Modify (через `npx @nitra/cursor`): `AGENTS.md`, `CLAUDE.md`

- [ ] **Step 1: Створити SKILL.md**

Create `.cursor/skills/docs-regen/SKILL.md`:

```markdown
---
name: docs-regen
description: Регенерувати C4-документацію MLMaiL у docs/ci4/ з clean ADR у docs/adr/ через LLM
---

# /docs-regen — регенерація C4-документації MLMaiL

Регенерує 5 файлів у `docs/ci4/` (`01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`) з clean ADR у `docs/adr/`. Для кожного clean ADR (без `session:` у frontmatter) — додає sentinel-блок `**Опрацьовано**` з посиланнями на проекції.

## Коли запускати

- Після того, як `/n-adr-normalize` перетворив накопичені drafts на clean ADR.
- Після ручного додавання / редагування ADR у `docs/adr/`.
- Коли треба синхронізувати C4-документацію з ADR перед PR.

## Запуск

`bun run docs:regen`

Опції:

- `bun run docs:regen --projection 01-context` — лише одна проекція.
- `bun run docs:regen --all` — форсити regen усіх, ігноруючи мітки.
- `bun run docs:regen --dry` — план без LLM-викликів.
- `bun run docs:regen --check` — CI-режим, fail на drift.

## Перевірка після запуску

`git status && git diff docs/ci4/ docs/adr/` — review-вікно. Розробник вирішує commit / rollback.

## Деталі

- Spec: `docs/superpowers/specs/2026-05-18-docs-regen-design.md`.
- Implementation: `scripts/docs-regen.js`.
- Шаблони промптів: `docs/ci4/_templates/*.prompt.md`.
- Tracking: `docs/ci4/manifest.json`.
```

- [ ] **Step 2: Запустити sync `npx @nitra/cursor`**

Run: `npx @nitra/cursor`
Expected: оновлюються `AGENTS.md` і `CLAUDE.md` — додається запис про новий skill.

Якщо команда питає підтвердження або щось ламається — діагностувати, не пропускати.

- [ ] **Step 3: Перевірити, що skill зареєстровано**

Run: `grep -n 'docs-regen' AGENTS.md CLAUDE.md`
Expected: знайдено в обох файлах посилання на `.cursor/skills/docs-regen/SKILL.md` і команду `/docs-regen`.

- [ ] **Step 4: Run `git status && git diff` і зупинка**

---

## Task 17: Усі lint-перевірки і знятий verification checklist

**Files:** жодних змін коду (лише валідація).

- [ ] **Step 1: `lint-js`**

Run: `bun run lint-js`
Expected: 0 errors.

Якщо є — фіксити безпосередньо в нових файлах (`scripts/docs-regen/**`, `scripts/__tests__/**`, `scripts/docs-regen.js`). НЕ глушити правила.

- [ ] **Step 2: `lint-text`**

Run: `bun run lint-text`
Expected: 0 errors.

- [ ] **Step 3: `lint-style`**

Run: `bun run lint-style`
Expected: 0 errors (touched CSS не змінювали, має пройти).

- [ ] **Step 4: Всі скрипт-тести**

Run: `bun run test:scripts`
Expected: усі юніт-тести pass (Tasks 2-9: hash, marks, discover, manifest, triggers, llm, lock, cli).

- [ ] **Step 5: Існуючі тести `app/`**

Run: `cd app && bun run test`
Expected: усі pass — наша зміна не зачіпала `app/`.

- [ ] **Step 6: Verification checklist із spec**

Перевіряй кожен пункт. Якщо не виконано — повертайся до відповідного Task.

- [ ] `bun run docs:regen` на чистому репо створює `docs/ci4/01..04.md`, `decisions.md`, `_templates/*.prompt.md`, `manifest.json` без помилок.
- [ ] Усі 26 clean ADR мають sentinel-блок «Опрацьовано» в кінці.
- [ ] `manifest.json` містить 26 ADR (з `processed_at` і `projections`), 5 проекцій (з `output_hash` і `used_adrs`), хеші шаблонів і правила `n-ci4.mdc`. Поля `hash` у `adrs` немає.
- [ ] `bun run docs:regen` без змін у репо — exit 0 з повідомленням `nothing to regenerate`.
- [ ] Видалення мітки з одного ADR + `bun run docs:regen --check` → exit 1.
- [ ] `bun run docs:regen --projection 01-context` оновлює тільки `01-context.md`.
- [ ] `bunx markdownlint-cli2 docs/ci4/**/*.md` проходить.
- [ ] `bun run lint-text` проходить.
- [ ] `bun run lint-js` проходить (новий `scripts/docs-regen.js`).
- [ ] Жодного HTML-тегу в регенерованих файлах і ADR-мітках.
- [ ] `.cursor/skills/docs-regen/SKILL.md` створено, `npx @nitra/cursor` синхронізує `AGENTS.md` і `CLAUDE.md`.

- [ ] **Step 7: Run `git status && git diff` фінальний**

Зупинись з повним diff. Користувач переглядає, ухвалює рішення про commit чи дрібні правки.

---

## Підсумок

Після Task 17 у репо:

- `scripts/docs-regen.js` + 11 модулів у `scripts/docs-regen/`.
- 6 шаблонів у `scripts/docs-regen/default-templates/`.
- 8 файлів юніт-тестів у `scripts/__tests__/docs-regen/` (+ фікстури).
- 6 живих шаблонів у `docs/ci4/_templates/` (скопійовані з default на першому запуску).
- 5 регенерованих файлів у `docs/ci4/`.
- 26 ADR з міткою «Опрацьовано».
- `docs/ci4/manifest.json` з повним tracking-станом.
- `.cursor/skills/docs-regen/SKILL.md` + оновлені `AGENTS.md` і `CLAUDE.md`.
- `package.json` зі скриптами `docs:regen` і `test:scripts`.
- `.gitignore` з ігноруванням `.regen.log` і lock-файлу.

Phase 2 (`/docs-propose` workflow з author-flow) — поза цим планом, окрема спека і окремий план потім.
