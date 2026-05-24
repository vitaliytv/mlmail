---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T20:35:53+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового середовища з jsdom+vitest на happy-dom+bun:test

## Context and Problem Statement

Правило `vue` з `.cursor/rules/n-vue.mdc` забороняє використання `jsdom` та `vitest` у пакеті `app`. Команда `npx @nitra/cursor check` повертала `❌` на цьому правилі, і задача `/n-fix` вимагала усунення порушення.

## Considered Options

- Перейти на `@happy-dom/global-registrator` + `bun:test`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Перейти на `@happy-dom/global-registrator` + `bun:test`", because правило `vue` явно забороняє `jsdom` і `vitest`, і `bun test` є нативним раннером для Bun-монорепо.

### Consequences

- Good, because `npx @nitra/cursor check` повернув `12/12 ✅` після міграції.
- Bad, because `bun test` не компілює `.vue` SFC з коробки — потребував додаткового `Bun.plugin` на `@vue/compiler-sfc`, що було окремою задачею.

## More Information

- `app/package.json`: прибрано `jsdom`, `vitest`; додано `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`.
- `app/vite.config.js`: прибрано блок `test` (конфіг Vitest).
- `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js` (`mock` / `mock.module` з `bun:test`).
- Скрипт `test`: `bun test --preload ./test/happy-dom.preload.js src`.

---

## ADR Bun-плагін для компіляції Vue SFC у тестовому preload

## Context and Problem Statement

Після міграції на `bun:test` з'ясувалося, що Bun не має вбудованого компілятора Vue SFC — `.vue`-файли завантажуються як сирі рядки, що спричиняє падіння компонентних тестів (11 fail у `Login.test.js`). Автор запитав у transcript, як обробити мігровані тести.

## Considered Options

- Додати `Bun.plugin` на `@vue/compiler-sfc` у preload-файл
- Пропустити тести компонента (залишити тільки unit-тести)
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Додати `Bun.plugin` на `@vue/compiler-sfc` у preload-файл", because користувач явно обрав "Set up Vue SFC loader" у діалозі вибору.

### Consequences

- Good, because після додання плагіна всі 45 тестів пройшли (0 fail), зокрема компонентні тести `Login.vue`.
- Bad, because `bun:test` не підтримує `@quasar/vite-plugin` для авто-імпорту компонентів — потрібно реєструвати всі Quasar-компоненти глобально у `app/src/test-utils/quasar.js`.

## More Information

- `app/test/happy-dom.preload.js`: `GlobalRegistrator.register()` → `Bun.plugin` для `.vue` SFC → авто-імпорти `vue`/`vue-router` як глобали → `mock.module('quasar', () => quasar.client.js)`.
- `app/src/test-utils/quasar.js`: реєструє `Object.values(Quasar)` через `app.component(...)`.
- Причина порядку: `happy-dom` обов'язково реєструється до першого імпорту `vue/runtime-dom`, бо runtime-dom захоплює `document` при завантаженні.

---

## ADR Формат COVERAGE.md — Markdown без таймстампа

## Context and Problem Statement

Потрібно об'єднати метрики покриття (JS + Rust) у єдиний файл у корені репозиторію, щоб зміни були видні у `git diff`.

## Considered Options

- Markdown з підсумковою таблицею без таймстампа
- JSON (машиночитабельний)
- Markdown із таймстампом
- Інші варіанти не були детально обговорені, але перші три сформульовано у `AskUserQuestion`.

## Decision Outcome

Chosen option: "Markdown з підсумковою таблицею без таймстампа", because користувач вибрав Markdown + «Лише підсумки» у діалозі; таймстамп явно виключено, щоб `git diff` рухався тільки при реальній зміні покриття.

### Consequences

- Good, because `COVERAGE.md` у `git diff` відображає тільки зміни відсотків/чисел, без шуму від дати запуску.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

- Файл: `COVERAGE.md` у корені монорепо.
- Колонки: `Область | Рядки | Функції`.
- Формат числових комірок: `99.74% (390/391)` (абсолютні значення збережено для видимості приросту коду, не тільки відсотків).

---

## ADR Єдиний скрипт coverage.js для покриття та мутаційних метрик

## Context and Problem Statement

Паралельна сесія реалізувала мутаційне тестування як окрему субкоманду з двома окремими секціями у `COVERAGE.md` та окремою командою `mutation` у кореневому `package.json`. Користувач попросив об'єднати все в одну команду й одну таблицю.

## Considered Options

- Один скрипт `scripts/coverage.js` з одним прогоном (покриття + мутації) → одна таблиця
- Два окремі скрипти з двома командами
- Один файл з двома секціями, кожну оновлює своя команда

## Decision Outcome

Chosen option: "Один скрипт `scripts/coverage.js` з одним прогоном", because користувач явно сказав «я не буду окремо запускати» і «поєднуй все в 1 скрипт».

### Consequences

- Good, because одна команда `bun run coverage` дає повний знімок якості (покриття + mutation score) за один прогін.
- Bad, because прогін стає повільним — `cargo-mutants` і Stryker запускають увесь тестовий набір на кожну мутацію; окремий швидкий `coverage` без мутацій більше недоступний через root-команду.

## More Information

- `scripts/coverage.js`: послідовно запускає JS-покриття (lcov), Rust-покриття (`cargo llvm-cov --json`), JS-мутації (Stryker command-runner), Rust-мутації (`cargo-mutants`); парсить результати; пише `COVERAGE.md`.
- Колонки таблиці: `Область | Рядки | Функції | Вбито мутацій | Score`.
- root `package.json`: одна команда `coverage: "bun scripts/coverage.js"`.
- Granular команди `test:mutation` / `test:rust:mutation` збережено у `app/package.json` як будівельні блоки.

---

## ADR Stryker inPlace режим для Bun-монорепо

## Context and Problem Statement

Стандартний режим Stryker копіює файли у тимчасову пісочницю перед мутацією. У Bun-монорепо з hoisted `node_modules` це ламає пісочницю: скопійований код не знаходить залежностей за очікуваними шляхами.

## Considered Options

- `inPlace: true` (мутувати файли на місці, відновлювати після кожного тесту)
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "`inPlace: true`", because коментар у `app/stryker.config.mjs` явно пояснює: «`inPlace` avoids hoisted-node_modules issues in a Bun monorepo sandbox».

### Consequences

- Good, because Stryker успішно запускається у монорепо без помилок резолюції залежностей.
- Bad, because при примусовому завершенні (`SIGKILL`) Stryker не встигає відновити мутовані файли — `auth-store.js`, `auth-errors.js`, `App.vue`, `Login.vue` залишились у Stryker-інструментованому стані (маркери `stryNS_…`), і потрібне `git restore` вручну.

## More Information

- `app/stryker.config.mjs`: `inPlace: true`, `testRunner: 'command'`, `commandRunner.command: 'bun test --preload ./test/happy-dom.preload.js src'`.
- `mutate: ["src/**/*.js", "src/**/*.vue"]` (`.vue` `<script>` блоки мутуються Stryker підтримує це нативно).
- `.gitignore`: `app/reports/` (артефакти Stryker), `mutants.out/` (cargo-mutants; виправлено з неправильного `app/mutants.out/`).
