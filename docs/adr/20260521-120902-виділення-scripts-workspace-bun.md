# Виділення `scripts/` у Окремий Bun Workspace

**Status:** Accepted
**Date:** 2026-05-21

## Context and Problem Statement

Пакети `globby` і `gray-matter` були у кореневому `devDependencies`, хоча використовуються виключно в `scripts/docs-regen/`. Правило `n-bun.mdc` вимагає, щоб залежності жили там, де їх використовують, а не в корені монорепо. Перевірка `npx @nitra/cursor check` фіксувала `bun: ❌`.

## Considered Options

- Зберегти `globby` і `gray-matter` у кореневому `devDependencies`
- Виділити `scripts/` у окремий Bun-workspace із власним `package.json`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Виділити `scripts/` у окремий Bun-workspace", because правило `n-bun.mdc` вимагає, щоб залежності інструментів жили у відповідному workspace; `npx @nitra/cursor check` повернув `bun: ❌` через кореневе розміщення.

Водночас у тій самій сесії виправлено кілька суміжних порушень `npx @nitra/cursor check`:

- `gitleaks detect` у `lint-security` замінено на `trufflehog filesystem … --exclude-paths .trufflehog-exclude --results=verified,unknown --fail` (вимога `n-security.mdc`).
- `globby ^14.0.2` замінено на `tinyglobby ^0.2.16` (вже транзитивна залежність, кращий lint від `@e18e`); у `discover.js`: `import { globby } from 'globby'` → `import { glob } from 'tinyglobby'`, `paths.sort()` → `paths.toSorted()`.
- Навмисні дублікати шаблонів `scripts/docs-regen/default-templates/` ↔ `docs/ci4/_templates/` виключено з `jscpd` (функція `bootstrapTemplates()` копіює їх при першому запуску — дублювання свідоме).
- `docs/ci4/manifest.json` і `.claude/settings.local.json` додано до `.v8rignore` (власні формати без схем у Schema Store).

### Consequences

- Good, because `bun: ✅` після переміщення; `security: ✅` після заміни на `trufflehog`; `bun install` видалив 2 зайві пакети з кореня (`Removed: 2`).
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Створено `scripts/package.json` з `"name": "mlmail-scripts"`, залежностями `gray-matter ^4.0.3` і `tinyglobby ^0.2.16`; у кореневий `package.json` додано `"scripts"` до масиву `workspaces`. Новий скрипт `lint-security`: `trufflehog filesystem . --no-update --exclude-paths .trufflehog-exclude --results=verified,unknown --fail`. Файл `.trufflehog-exclude` виключає `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures`. CI: `.github/workflows/lint-security.yml` на `push`/`pull_request` до `dev`/`main`. До `.jscpd.json` додано `"scripts/docs-regen/default-templates/**"` і `"docs/ci4/_templates/**"`.

## Update 2026-05-21

`scripts/docs-regen.js` звертався до `process.env.DOCS_REGEN_MODEL` напряму. Правило `n-js-run` вимагає використовувати `env` з `@nitra/check-env` для доступу до змінних середовища у backend-скриптах. Після заміни (`import { env } from '@nitra/check-env'` + `env.DOCS_REGEN_MODEL` замість `process.env.DOCS_REGEN_MODEL`) `npx @nitra/cursor check` перестав виявляти порушення правила `js-run`.
