# Bun-плагін для компіляції Vue SFC у bun:test

**Status:** Accepted
**Date:** 2026-05-22

## Context and Problem Statement

`bun test` не має вбудованого компілятора Vue SFC: `.vue`-файли завантажуються як рядки (`typeof module.default === 'string'`). Після міграції на `bun:test` компонентні тести (`Login.test.js`) падали з `ReferenceError: ref is not defined` та помилками монтування — `mountWithQuasar(Login)` отримував рядок замість component-об'єкта.

## Considered Options

- Налаштувати `Bun.plugin` із `@vue/compiler-sfc` у preload-файлі
- Пропустити/видалити компонентні тести (залишити лише юніт-тести)
- Завантажувати `.vue` як рядки (залишити як є)

## Decision Outcome

Chosen option: "Налаштувати `Bun.plugin` із `@vue/compiler-sfc` у preload-файлі", because користувач явно обрав «Set up Vue SFC loader» через `AskUserQuestion`-виклик у transcript.

### Consequences

- Good, because `bun test` проходить 45/45 (у пізніших ітераціях 47/47), включно з усіма компонентними тестами `Login.vue` і `App.vue`.
- Good, because авто-імпорт-глобали у preload полагодили тести `auth-store`, які падали на `main` з `ref is not defined`.
- Bad, because `app/test/happy-dom.preload.js` вимагає ручного перерахування Vue-глобалів (`createApp`, `ref`, `computed` тощо) та примусового форсування browser-збірки Quasar: Quasar резолвить умову `node` у полі `exports` до серверної збірки `quasar.server.prod.js`, яка падає без `window` — обхід через `mock.module('quasar', () => require('quasar/dist/quasar.client.js'))` у preload.

## More Information

- `app/test/happy-dom.preload.js`: реєструє `GlobalRegistrator.register()` (happy-dom) до будь-якого імпорту Vue (щоб `runtime-dom` захопив `document`), реєструє `Bun.plugin` що компілює `.vue` через `@vue/compiler-sfc`, встановлює глобали `Vue` / `vue-router` (авто-імпорти), форсує browser-збірку Quasar через `mock.module('quasar', ...)`
- `app/src/test-utils/quasar.js`: хелпер `mountWithQuasar`/`mountQuasar`, реєструє всі Quasar-компоненти через `Object.entries(Quasar)` (аналог `@quasar/vite-plugin`, якого немає під `bun test`)
- `app/package.json`: додано `@vue/compiler-sfc ^3.5.34` у `devDependencies`; версія збігається з `vue 3.5.34` у `node_modules`
- Команда тестів: `bun test --preload ./test/happy-dom.preload.js src`
- `.gitignore` виправлено: `app/mutants.out/` → `mutants.out/` (глобус без префікса матчить `mutants.out/` на будь-якому рівні вкладеності, покриває і `app/mutants.out/` і `app/src-tauri/mutants.out/`)
