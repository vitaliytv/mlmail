---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-21T23:44:00+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового середовища з jsdom+vitest на happy-dom+bun:test

## Context and Problem Statement
Правило `n-vue` у `.cursor/rules/n-vue.mdc` забороняє `jsdom` і `vitest` у проєкті. `app/package.json` містив обидва пакети, а `vite.config.js` — блок `test` з конфігурацією Vitest. Тест `Login.vitest.js` був написаний під Vitest API.

## Considered Options
* Міграція на `@happy-dom/global-registrator` + `bun:test`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Міграція на `@happy-dom/global-registrator` + `bun:test`", because це єдиний шлях, що задовольняє правило `n-vue` і зберігає компонентні тести без окремого test-runner'а поряд з Bun.

### Consequences
* Good, because `npx @nitra/cursor check` проходить 12/12 правил; `bun test` запускає 45+ тестів без зовнішніх залежностей (jsdom, vitest).
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Змінені файли: `app/package.json` (видалено `jsdom`, `vitest`; додано `@happy-dom/global-registrator`), `app/vite.config.js` (видалено блок `test`), `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js`. Новий файл: `app/test/happy-dom.preload.js`. Скрипт тестів: `bun test --preload ./test/happy-dom.preload.js src`.

---

## ADR Bun-плагін для компіляції Vue SFC у тестах

## Context and Problem Statement
`bun test` не вміє компілювати `.vue` SFC-файли: під час першого запуску після міграції `Login.test.js` падав з `ReferenceError: ref is not defined`, а потім з помилкою `Object.assign requires that input parameter not be null or undefined` при спробі завантажити `quasar.server.prod.js`.

## Considered Options
* Додати Bun.plugin на `@vue/compiler-sfc` у `test/happy-dom.preload.js`
* Пропустити компонентні тести (залишити лише unit-тести)
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Додати Bun.plugin на `@vue/compiler-sfc` у `test/happy-dom.preload.js`", because користувач явно обрав цей підхід, коли йому запропонували вибір.

### Consequences
* Good, because усі 45 тестів (включно з компонентними для `Login.vue` та `App.vue`) проходять через `bun test`; `.vue` SFCs компілюються без Vite/Vitest.
* Bad, because `quasar` потребує примусової підміни на browser-збірку (`quasar/dist/quasar.client.js`) через Bun mock — `quasar` резолвить `node`-умову пакету до серверної збірки, яка падає без `window`.

## More Information
`app/test/happy-dom.preload.js`: реєструє `GlobalRegistrator.register()` до будь-якого імпорту Vue (щоб `runtime-dom` захопив `document`), потім `Bun.plugin` парсить `.vue` через `@vue/compiler-sfc`, потім `mock.module('quasar', ...)` замінює `quasar` на `quasar/dist/quasar.client.js`. `app/src/test-utils/quasar.js` реєструє всі Quasar-компоненти глобально (немає `@quasar/vite-plugin` під `bun test`). Версія `@vue/compiler-sfc`: `3.5.34`.

---

## ADR Встановлення cargo-llvm-cov для покриття Rust

## Context and Problem Statement
`cargo test` не рахує покриття. В `src-tauri/src` міститься 71 тест у 11 файлах, тому потрібен інструмент для вимірювання покриття Rust-коду.

## Considered Options
* Встановити `cargo-llvm-cov` (LLVM source-based coverage)
* `cargo-tarpaulin`
* Інші варіанти в transcript не обговорювалися.

## Decision Outcome
Chosen option: "Встановити `cargo-llvm-cov`", because це стандартний і точний інструмент на основі LLVM source-based coverage; користувач обрав варіант «встановити й прогнати».

### Consequences
* Good, because transcript фіксує очікувану користь: звіт покриття по рядках для всіх `src-tauri/src/**/*.rs` доступний через `bun run test:rust:coverage`.
* Bad, because transcript не містить підтверджених негативних наслідків.

## More Information
Команди встановлення: `rustup component add llvm-tools-preview`, `cargo install cargo-llvm-cov`. Версія `cargo-llvm-cov`: `0.8.7`. Новий скрипт у `app/package.json`: `"test:rust:coverage": "cargo llvm-cov --manifest-path src-tauri/Cargo.toml"`. Директорія `app/src-tauri/target/llvm-cov-target` потрапляє під наявний `.gitignore`-патерн `target/`.
