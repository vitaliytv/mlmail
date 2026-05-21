---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-21T23:24:50+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового середовища з jsdom + Vitest на happy-dom + bun:test

## Context and Problem Statement
Правило `n-vue.mdc` у `.cursor/rules/` забороняє `jsdom` і `vitest` у воркспейсі `app`. `npx @nitra/cursor check` повертав `❌` саме через ці залежності в `app/package.json` і блок `test` у `vite.config.js`.

## Considered Options
* Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun:test`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun:test`", because правило `n-vue.mdc` явно забороняє `jsdom` і `vitest`, а `bun:test` вже є єдиним дозволеним test-runner-ом у монорепо згідно з правилом `n-bun.mdc`.

### Consequences
* Good, because `npx @nitra/cursor check` перейшов із 11/12 на 12/12 — порушення правила `vue` усунено.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- `app/package.json`: видалено `jsdom`, `vitest`; додано `@happy-dom/global-registrator ^20.9.0`, `@types/bun ^1.3.14`
- `app/vite.config.js`: видалено блок `test` (конфіг Vitest)
- `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js`: всі `vi.fn()` / `vi.mock` замінено на `mock()` / `mock.module` з `bun:test`
- Скрипт `test` у `app/package.json`: `bun test --preload ./test/happy-dom.preload.js src`
- Версія `app` підвищена `0.1.0` → `0.1.1`, створено `app/CHANGELOG.md` (правило `n-changelog.mdc`)

---

## ADR Vue SFC-компіляція в bun:test через Bun.plugin + @vue/compiler-sfc

## Context and Problem Statement
`bun test` не має вбудованого компілятора Vue SFC: `.vue`-файли завантажуються як рядки. Після міграції на `bun:test` компонентні тести (`Login.test.js`) падали з `ReferenceError: ref is not defined` та помилками монтування, бо `mountWithQuasar(Login)` отримував string замість component-об'єкта.

## Considered Options
* Додати `Bun.plugin` з `@vue/compiler-sfc` у preload-файл
* Пропустити компонентні тести (зберегти лише юніт-тести)
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Додати `Bun.plugin` з `@vue/compiler-sfc` у preload-файл", because користувач явно обрав «Set up Vue SFC loader» коли запитали як вирішити падіння Login-тестів.

### Consequences
* Good, because `bun test` проходить 45/45, включно зі всіма компонентними тестами `Login.test.js`.
* Good, because transcript фіксує очікувану користь: авто-імпорт-глобали у preload також полагодили юніт-тести `auth-store`, які падали на `main` з `ref is not defined`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
- `app/test/happy-dom.preload.js`: викликає `GlobalRegistrator.register()` (happy-dom), реєструє `Bun.plugin` що компілює `.vue` через `@vue/compiler-sfc`, встановлює глобали `Vue` / `vue-router` (авто-імпорти), форсує `mock.module('quasar', …)` на browser-збірку `quasar/dist/quasar.esm.prod.js`
- `app/src/test-utils/quasar.js`: реєструє всі Quasar-компоненти через `Object.entries(Quasar)` (аналог `@quasar/vite-plugin`, якого немає в `bun test`)
- `app/package.json`: додано `@vue/compiler-sfc ^3.5.34` у `devDependencies`
- Версія `@vue/compiler-sfc` збігається з `vue 3.5.34` у `node_modules`
