# Інтеграція gitleaks як обов'язкового security-лінтера

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

Правило `n-security.mdc` (версія `1.1`, частина `@nitra/cursor@1.13.13`) вимагає наявності `.gitleaks.toml` та скрипту `lint-security` у `package.json`. До впровадження ні файл, ні скрипт у проєкті не існували; `npx @nitra/cursor check` повертав 2 порушення по правилу `security`.

## Considered Options

- Додати `.gitleaks.toml` + `lint-security` відповідно до канону `n-security.mdc`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Додати `.gitleaks.toml` + `lint-security`", because `npx @nitra/cursor check` відхиляв збірку з кодом виходу `1` через відсутність цих артефактів, а правило є обов'язковим.

### Consequences

- Good, because після змін `npx @nitra/cursor check` досяг `12/12 правил без зауважень`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- `.gitleaks.toml`: `[extend] useDefault = true` + `[allowlist]` для `node_modules`, `.git`, `dist`, `build`, `*.lock`, `fixtures?/`.
- `package.json` `scripts.lint-security`: `"gitleaks detect --no-banner"`.
- `package.json` `scripts.lint`: доданий `&& bun run lint-security` до кінця ланцюга.
- Правило: `.cursor/rules/n-security.mdc` (version `1.1`).
- Та сама сесія: видалено явні `import { ... } from 'vue'` з `app/src/services/auth-store.js`, `app/src/test-utils/quasar.js`, `app/src/views/Login.vue` відповідно до `n-vue.mdc`; `npx @nitra/cursor check` підтвердив `12/12`.

---

**Опрацьовано** 2026-05-19. Проекції:
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
