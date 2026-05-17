---
session: b69d2c87-5272-41a6-8c90-8197ce66c4b0
captured: 2026-05-17T19:07:21+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/b69d2c87-5272-41a6-8c90-8197ce66c4b0.jsonl
---

## ADR Інтеграція gitleaks як обов'язкового security-лінтера

## Context and Problem Statement
Пакет `@nitra/cursor@1.13.13` додав нове правило `n-security.mdc`, яке вимагало наявності `.gitleaks.toml` та скрипту `lint-security` у проєкті. `npx @nitra/cursor check` повертав три помилки: відсутній `.gitleaks.toml`, відсутній `scripts.lint-security`, не інтегровано `lint-security` в агрегований `lint`.

## Considered Options
* Створити `.gitleaks.toml` за каноном `security.mdc` і додати скрипт `lint-security`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Створити `.gitleaks.toml` і додати `lint-security`", because `npx @nitra/cursor check` вимагав саме цього відповідно до `n-security.mdc`, а ціль сесії — привести проєкт у відповідність до всіх правил.

### Consequences
* Good, because `npx @nitra/cursor check` досяг результату `12/12 правил без зауважень`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- `.gitleaks.toml` — `[extend] useDefault = true`, `[allowlist]` ігнорує `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures`
- `package.json` (корінь): `scripts.lint-security = "gitleaks detect --no-banner"`, доданий виклик `bun run lint-security` в агрегований `lint`
- Правило-джерело: `.cursor/rules/n-security.mdc`

---

## ADR Виключення `.claude/worktrees/` з `.gitignore` для уникнення хибних спрацювань jscpd

## Context and Problem Statement
`bun run lint` падав на кроці `jscpd` через виявлення дублікатів між робочими файлами та тимчасовою директорією `.claude/worktrees/hopeful-cori-9cef9b/`, яку Claude Code створює для ізольованих агентських workspace-ів. Ця директорія не потрапляла до `.gitignore`, тому `jscpd` (з `gitignore: true`) сканував її разом з основними файлами.

## Considered Options
* Додати `.claude/worktrees/` до `.gitignore`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Додати `.claude/worktrees/` до `.gitignore`", because `jscpd` поважає `.gitignore` (`gitignore: true` в конфігурації), а директорія `.claude/worktrees/` є ефемерним артефактом Claude Code і не має сканувати як частина кодової бази.

### Consequences
* Good, because після зміни `bun run lint` пройшов крок `jscpd` без помилок.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- Змінений файл: `.gitignore` — додано рядок `.claude/worktrees/`
- Відповідний інструмент: `bunx jscpd .` (викликається як частина `lint-js`)

---

## ADR Виключення `.cursor/rules/*.mdc` із markdownlint

## Context and Problem Statement
`bun run lint-text` (через `markdownlint-cli2`) перевіряв файли у `.cursor/rules/`, які синхронізуються пакетом `@nitra/cursor` і не підлягають ручному редагуванню. Виправлення порушень у цих файлах безглузде, оскільки наступний запуск `npx @nitra/cursor` перезапише їх.

## Considered Options
* Додати `.cursor/rules/**` до поля `ignores` у `.markdownlint-cli2.jsonc`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Виключити `.cursor/rules/**` з `ignores`", because ці файли керуються зовнішнім пакетом `@nitra/cursor` і правки в них перезаписуються при синхронізації.

### Consequences
* Good, because `bun run lint-text` більше не звітує про порушення в зовнішньо керованих файлах.
* Bad, because реальні помилки в `.cursor/rules/` також не будуть виявлені lint-ом — прийнятно, оскільки ці файли не редагуються вручну.

## More Information
- Змінений файл: `.markdownlint-cli2.jsonc` — додано `".cursor/rules/**"` до масиву `ignores`
- Пакет-джерело правил: `@nitra/cursor@1.13.13`

---

## ADR Додавання файлів без Schema Store до `.v8rignore`

## Context and Problem Statement
`bun run lint-text` запускає `bunx v8r` для валідації JSON/TOML файлів за JSON Schema. Файли `.cursor/hooks.json` та `.gitleaks.toml`, створені в рамках сесії, не мають відповідних схем у Schema Store — `v8r` завершував перевірку з помилкою `Could not find a schema to validate`.

## Considered Options
* Додати `.cursor/hooks.json` і `.gitleaks.toml` до `.v8rignore`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Додати до `.v8rignore`", because правило `n-text.mdc` передбачає виключати файли, для яких Schema Store не має запису, щоб `v8r` не завершувався помилкою.

### Consequences
* Good, because `bun run lint-text` проходить без помилок після виключення цих файлів.
* Bad, because зміни схеми `.cursor/hooks.json` або `.gitleaks.toml` не будуть автоматично підхоплені Schema-валідацією — Neutral, because transcript не містить підтвердження наслідку.

## More Information
- Змінений файл: `.v8rignore`
- Правило-джерело: `.cursor/rules/n-text.mdc` (секція про `v8r` та `.v8rignore`)
