# Міграція з Vitest на Bun Test Runner

**Status:** Accepted
**Date:** 2026-05-17

## Context and Problem Statement

У проєкті `app/` паралельно існували залежність Vitest у `devDependencies` і скрипт `bun test`, що створювало дублювання. Обидва наявні тест-файли (`auth-errors.test.js`, `auth-store.test.js`) вже імпортували з `bun:test` та використовували `mock.module()` — Bun-специфічний API, несумісний із Vitest. Скрипт `test` у `app/package.json` також викликав `bun test`. Vitest залишався у `devDependencies` без жодного реального тесту під його API.

## Considered Options

- Bun Test Runner як єдиний test runner
- Гібридна схема: Vitest тільки для DOM-сюїти, Bun для юніт-тестів
- `jsdom`-preload замість `happy-dom`

## Decision Outcome

Chosen option: "Bun Test Runner як єдиний test runner", because фактичний стан коду вже спирався на `bun:test`/`mock.module()`, тобто повернення до Vitest технічно неможливе без рефакторингу, а Bun Test Runner не потребує окремого конфіга.

### Consequences

- Good, because усуває дублювання залежностей — `vitest` і `jsdom` підлягають видаленню з `devDependencies`.
- Good, because `happy-dom` швидший за `jsdom` і достатній для Vue-компонентного тестування.
- Bad, because transcript не містить підтвердження негативних наслідків.
- Neutral, because для DOM-тестів Vue-компонентів потрібна мінімальна конфігурація через `bunfig.toml` preload.

## More Information

- `app/package.json`: скрипт `"test": "bun test src/services src/i18n"`, `vitest` і `jsdom` підлягають видаленню з `devDependencies`.
- `app/src/i18n/auth-errors.test.js`, `app/src/services/auth-store.test.js`: імпорт виключно з `bun:test` (`describe`, `it`, `expect`, `mock`, `beforeEach`).
- `.cursor/rules/n-vue.mdc` оновлено (версія 1.8 → 1.9): секція «Тестування» рекомендує `bun:test` замість Vitest.
- Гібридна схема відхилена як надмірно складна конфігурація.
- `jsdom`-preload — запасний варіант при нестачі API у `happy-dom`, але пріоритет за `happy-dom`.

## Update 2026-05-17

Деталі налаштування `happy-dom` для DOM-тестів з `@vue/test-utils`:

1. `app/package.json` devDependencies: `"happy-dom": "^16.0.0"`.
2. `app/bunfig.toml`:

```toml
[test]
preload = ["./setup-happy-dom.ts"]
```

3. `app/setup-happy-dom.ts`:

```ts
import { GlobalRegistrator } from '@happy-dom/global-registrator'
GlobalRegistrator.register()
```

Після реєстрації `document`, `window`, `HTMLElement` доступні у тестах; `@vue/test-utils` підхоплює автоматично.

---

**Опрацьовано** 2026-05-20. Проекції:
- [02-containers](../ci4/02-containers.md)
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
