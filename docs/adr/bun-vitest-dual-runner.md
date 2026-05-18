# Dual-runner стратегія: bun:test для pure-JS, Vitest для Vue SFC

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

`bun test` фейлив на існуючих Vitest-тестах: `unplugin-auto-import` не виконується без Vite plugin pipeline, `vi.fn()` / `vi.mock()` відсутні в bun:test API, а Vue SFC потребують компіляції через `@vitejs/plugin-vue` + Quasar plugin. Потрібно було зробити `bun test` функціональним без втрати покриття Vue-компонентів.

## Considered Options

- Повна міграція на `bun:test` включно з Vue SFC через `bun-plugin-vue`
- `bunx vitest` для всіх тестів без змін коду
- Dual-runner: `bun:test` для pure-JS, Vitest для Vue SFC

## Decision Outcome

Chosen option: "Dual-runner: bun:test для pure-JS, Vitest для Vue SFC", because bun:test швидший для бізнес-логіки без Vite transform, а Vue SFC + Quasar потребують Vite pipeline, яку bun не надає стабільно.

### Consequences

- Good, because `bun test` запускається і покриває pure-JS логіку (store, i18n) без Vite overhead.
- Good, because розподіл за розширенням (`.test.js` vs `.vitest.js`) дає чітку конвенцію без per-file конфігурації.
- Bad, because `bun-plugin-vue` нестабільний — повна міграція займе ~3-4 год без гарантій результату.
- Neutral, because два окремі runner-и потребують двох команд або агрегатного скрипту `test:all`.

## More Information

Конкретні зміни:
- `auth-store.js`: додано явний `import { readonly, ref } from 'vue'` замість auto-import.
- `auth-store.test.js`, `auth-errors.test.js`: переписано під `bun:test` API (`mock()`, `mock.module()`).
- `Login.test.js` → `Login.vitest.js` (залишається під Vitest).
- `vite.config.js`: `include` звужено до `'src/**/*.vitest.{js,vue}'`.
- `package.json` скрипти: `test` (bun:test для `src/services src/i18n`), `test:ui` (vitest run), `test:rust` (cargo test), `test:all` (усі три послідовно).

Зачіпає: `app/src/services/auth-store.js`, `app/src/services/auth-store.test.js`, `app/src/i18n/auth-errors.test.js`, `app/src/views/Login.vitest.js` (перейменовано з `Login.test.js`), `app/vite.config.js`, `app/package.json`.
