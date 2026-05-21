---
session: ecf97f5f-d56c-48f6-8917-b4e114439bc2
captured: 2026-05-21T10:19:56+03:00
transcript: /Users/vitalii/.claude/projects/-Users-vitalii-www-vitaliytv-mlmail/ecf97f5f-d56c-48f6-8917-b4e114439bc2.jsonl
---

## ADR Виділення залежностей скриптів у workspace `scripts`

## Context and Problem Statement

Пакети `globby` і `gray-matter` були у кореневому `devDependencies` (`package.json`), хоча використовуються виключно в `scripts/docs-regen/discover.js`. Правило `n-bun` вимагає, щоб залежності жили там, де їх використовують, а не в корені монорепо.

## Considered Options

- Зберегти `globby` і `gray-matter` у кореневому `devDependencies`
- Виділити `scripts/` у окремий Bun-workspace із власним `package.json`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Виділити `scripts/` у окремий Bun-workspace", because перевірка `npx @nitra/cursor check` фіксувала порушення правила `bun` — залежності мають жити поряд із кодом, який їх використовує.

### Consequences

- Good, because `bun install` після змін видалив 2 зайві пакети з кореня та встановив їх у потрібний workspace (`Removed: 2`).
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- Створено `scripts/package.json` з `"name": "mlmail-scripts"`, залежностями `globby ^14.0.2`, `gray-matter ^4.0.3`.
- У корені `package.json`: `"workspaces": ["app", "scripts"]`; `globby` і `gray-matter` видалені з `devDependencies`.
- Створено `scripts/CHANGELOG.md` (вимога правила `n-changelog`).

---

## ADR Перехід із `gitleaks` на `trufflehog` для перевірки лінтером безпеки

## Context and Problem Statement

Скрипт `lint-security` у `package.json` викликав `gitleaks detect --no-banner`. Правило `n-security` вимагає використання `trufflehog` з виключеннями через `.trufflehog-exclude` та CI-workflow `lint-security.yml`.

## Considered Options

- Залишити `gitleaks`
- Замінити на `trufflehog` з файлом виключень `.trufflehog-exclude`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "`trufflehog` з `.trufflehog-exclude`", because `npx @nitra/cursor check` вимагав відповідності канонічному шаблону правила `n-security`, яке явно прописує `trufflehog`.

### Consequences

- Good, because `npx @nitra/cursor check` досяг `12/12 правил без зауважень` після змін.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- `lint-security` у `package.json`: `trufflehog filesystem . --no-update --exclude-paths .trufflehog-exclude --results=verified,unknown --fail`
- Створено `.trufflehog-exclude` з виключеннями: `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures`.
- Створено `.github/workflows/lint-security.yml` для CI-запуску на `push`/`pull_request` до `dev` та `main`.

---

## ADR Використання `@nitra/check-env` замість прямого `process.env` у скриптах

## Context and Problem Statement

`scripts/docs-regen.js` звертався до `process.env.DOCS_REGEN_MODEL` напряму. Правило `n-js-run` вимагає використовувати `env` з `@nitra/check-env` для доступу до змінних середовища у backend-скриптах.

## Considered Options

- Залишити `process.env` із `node:process`
- Імпортувати `env` з `@nitra/check-env`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Імпортувати `env` з `@nitra/check-env`", because `npx @nitra/cursor check` повернув `❌ [scripts] docs-regen.js:125 — process.env.DOCS_REGEN_MODEL: заміни на env з '@nitra/check-env'`.

### Consequences

- Good, because transcript фіксує очікувану користь: після заміни `npx @nitra/cursor check` перестав виявляти порушення правила `js-run`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- `scripts/docs-regen.js` рядок 2: додано `import { env } from '@nitra/check-env'` поруч з наявними імпортами з `node:fs/promises`.
- Усі входження `process.env.DOCS_REGEN_MODEL` замінено на `env.DOCS_REGEN_MODEL`.
