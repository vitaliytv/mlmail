---
session: b69d2c87-5272-41a6-8c90-8197ce66c4b0
captured: 2026-05-17T18:29:18+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/b69d2c87-5272-41a6-8c90-8197ce66c4b0.jsonl
---

## ADR Інтеграція `gitleaks` як обов'язкового security-лінтера

## Context and Problem Statement

Правило `n-security.mdc` (версія `1.1`, частина `@nitra/cursor@1.13.13`) вимагає наявності `.gitleaks.toml` та скрипту `lint-security` у `package.json`. До сесії ні файл, ні скрипт у проєкті не існували; `npx @nitra/cursor check` повертав 2 порушення по правилу `security`.

## Considered Options

- Додати `.gitleaks.toml` + `lint-security` відповідно до канону `security.mdc`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Додати `.gitleaks.toml` + `lint-security`", because `npx @nitra/cursor check` відхиляв збірку з кодом виходу `1` через відсутність цих артефактів.

### Consequences

- Good, because transcript фіксує очікувану користь: після змін `npx @nitra/cursor check` дав `12/12 правил без зауважень`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- Файл `.gitleaks.toml`: `[extend] useDefault = true` + `[allowlist]` для `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures?/`.
- `package.json` `scripts.lint-security`: `"gitleaks detect --no-banner"`.
- `package.json` `scripts.lint`: доданий `&& bun run lint-security` до кінця ланцюга.
- Правило: `.cursor/rules/n-security.mdc` (version `1.1`).

---

## ADR Vue auto-import: заборона явних імпортів з `'vue'`

## Context and Problem Statement

Правило `n-vue.mdc` вимагає не писати явні імпорти значень (composables, хелперів) із модуля `'vue'` — вони мають надходити через механізм auto-import. Три файли у `app/src/` містили такі імпорти, що спричиняло порушення при `npx @nitra/cursor check`.

## Considered Options

- Видалити явні `import { ... } from 'vue'` і покластися на auto-import
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Видалити явні `import { ... } from 'vue'`", because вимагає правило `n-vue.mdc` і auto-import покриває ці символи автоматично.

### Consequences

- Good, because transcript фіксує очікувану користь: після видалення `npx @nitra/cursor check` досяг `12/12`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- `app/src/services/auth-store.js`: видалено `import { ref, readonly } from 'vue'`.
- `app/src/test-utils/quasar.js`: видалено `import { h } from 'vue'`.
- `app/src/views/Login.vue`: видалено `import { onMounted } from 'vue'`.
- Правило: `.cursor/rules/n-vue.mdc`.
