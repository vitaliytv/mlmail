---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T07:43:24+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

12/12. Тепер ADR-рішення з цієї сесії:

---

## ADR Міграція тестового середовища Vue з jsdom/Vitest на happy-dom/bun:test

## Context and Problem Statement

Правило `vue` у `.cursor/rules/n-vue.mdc` явно забороняє `jsdom` і `vitest` у Vue-проєкті. Команда `npx @nitra/cursor check` повертала `❌` по правилу `vue`. Потрібно було замінити тестове середовище, зберігши існуючі тести.

## Considered Options

- Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun:test`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun:test`", because це єдиний варіант, що задовольняє правило `vue` і вкладається в заборону `vitest`.

### Consequences

- Good, because `npx @nitra/cursor check` перейшов із 11/12 на 12/12 після міграції.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінені файли: `app/package.json` (прибрано `jsdom`, `vitest`; додано `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`), `app/vite.config.js` (прибрано блок `test`), `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js`. Нові файли: `app/test/happy-dom.preload.js`.

---

## ADR Bun-плагін для компіляції Vue SFC у середовищі bun:test

## Context and Problem Statement

Після міграції на `bun:test` з'ясувалося, що Bun не вміє компілювати `.vue` SFC без додаткового плагіна: `bun test` завантажує `.vue`-файли як рядки (`typeof m.default === 'string'`). Компонентні тести `Login.vue` провалювалися (11 з 45 тестів).

## Considered Options

- Налаштувати Bun-плагін (`Bun.plugin`) на `@vue/compiler-sfc` у preload-файлі
- Пропустити/видалити компонентні тести
- Запускати компонентні тести окремим скриптом

## Decision Outcome

Chosen option: "Налаштувати Bun-плагін на `@vue/compiler-sfc` у preload-файлі", because користувач явно обрав «Set up Vue SFC loader» через `AskUserQuestion`-виклик у transcript.

### Consequences

- Good, because transcript фіксує очікувану користь: усі 47 тестів проходять після налаштування, включно з компонентними тестами `Login.vue` і `App.vue`.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файл `app/test/happy-dom.preload.js`: реєструє `@happy-dom/global-registrator`, встановлює `Bun.plugin` з `@vue/compiler-sfc`, форсує browser-збірку Quasar через `mock.module('quasar', …)`, надає глобальні авто-імпорти Vue і Vue Router. `app/src/test-utils/quasar.js` додано хелпер `mountQuasar` із глобальною реєстрацією всіх Quasar-компонентів. Команда: `bun --cwd=app run test`.

---

## ADR Об'єднаний звіт покриття JS і Rust у COVERAGE.md

## Context and Problem Statement

У проєкті є два окремих coverage-скрипти: `bun test --coverage` (JS/Vue, `app/`) і `cargo llvm-cov` (Rust, `app/src-tauri/`). Результати існували лише в консолі, без збереження для відстеження динаміки між запусками.

## Considered Options

- Єдиний скрипт `scripts/coverage.js`, що парсить обидва джерела й записує підсумки у `COVERAGE.md` у корені монорепо
- JSON або LCOV як формат файлу
- Markdown із повними таблицями файлів

## Decision Outcome

Chosen option: "Єдиний скрипт із Markdown-підсумками у `COVERAGE.md`", because користувач обрав «Markdown (COVERAGE.md)» і «Лише підсумки» через `AskUserQuestion` у transcript; пояснена мотивація — читабельність і `git diff`.

### Consequences

- Good, because transcript фіксує очікувану користь: `git diff COVERAGE.md` між двома запусками одразу показує зміну відсотків.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Файл `scripts/coverage.js`: запускає `bun run test:coverage` і `cargo llvm-cov --manifest-path app/src-tauri/Cargo.toml` через `Bun.spawnSync`, парсить рядки `All files` (JS) і `TOTAL` (Rust) регулярними виразами, записує `COVERAGE.md`. Кореневий скрипт: `"coverage": "bun scripts/coverage.js"` у `package.json`. Залежності: `cargo-llvm-cov v0.8.7` + rustup-компонент `llvm-tools-preview`.

---

`app/CHANGELOG.md` оновлено (`0.1.3`, новий розділ), `npx @nitra/cursor check` → 12/12.
