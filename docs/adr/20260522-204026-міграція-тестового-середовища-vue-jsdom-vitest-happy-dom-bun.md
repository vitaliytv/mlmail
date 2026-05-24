---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T20:40:26+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового середовища Vue: jsdom/Vitest → Happy-DOM/bun test

## Context and Problem Statement

Правило `vue` у `.cursor/rules/` забороняє використання `jsdom` та `vitest` у воркспейсі `app`. На момент сесії `app/package.json` містив обидві залежності, а тести `Login.vitest.js` запускались через Vitest. Запуск `npx @nitra/cursor check` показував `11/12` — правило `vue` падало на `jsdom` + `vitest`. Завдання сесії — усунути ці порушення.

## Considered Options

- Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun test`
- Зберегти поточний стек із виключеннями у правилі `vue`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Замінити `jsdom` + `vitest` на `@happy-dom/global-registrator` + `bun test`", because правило `vue` явно забороняє `jsdom` і `vitest`, і usердиктував відповідність правилам як вимогу без винятків.

### Consequences

- Good, because `npx @nitra/cursor check` → 12/12 (повне проходження усіх правил).
- Bad, because `bun test` не компілює `.vue` SFC нативно — знадобився окремий `Bun.plugin` на основі `@vue/compiler-sfc` у preload-файлі `app/test/happy-dom.preload.js`.

## More Information

Змінені файли: `app/package.json` (прибрано `jsdom`, `vitest`; додано `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun`), `app/vite.config.js` (прибрано блок `test`), `app/test/happy-dom.preload.js` (новий preload: реєстрація happy-dom + SFC-компілятор + автоімпорти Vue/Vue Router + Quasar browser-збірка), `app/src/test-utils/quasar.js` (глобальна реєстрація Quasar-компонентів), `app/src/views/Login.test.js` (міграція `Login.vitest.js` на `bun:test`).

---

## ADR Додавання Vue SFC-компілятора як Bun-плагіна у тестовий preload

## Context and Problem Statement

Після міграції на `bun test` (замість Vitest) `Login.vue`-тести падали: Bun завантажував `.vue` файли як сирі рядки, бо не має вбудованого компілятора Vue SFC. Потрібно було налаштувати компіляцію `.vue` у тестовому середовищі без Vite/Vitest.

## Considered Options

- Додати `Bun.plugin` на базі `@vue/compiler-sfc` у preload-файл
- Налаштувати окремий білд-крок перед тестами (не обговорювався)

## Decision Outcome

Chosen option: "Додати `Bun.plugin` на базі `@vue/compiler-sfc` у preload-файл", because user прямо обрав «Set up Vue SFC loader» у відповідь на запитання про стратегію виправлення Login-тестів.

### Consequences

- Good, because transcript фіксує очікувану користь: `bun test` → 45 pass / 0 fail після додавання плагіна.
- Bad, because `@vue/compiler-sfc` треба встановлювати як devDep, а плагін покладається на `Bun.plugin` API, яке не є частиною стандарту.

## More Information

Файли: `app/test/happy-dom.preload.js` (Bun-плагін, що перехоплює `.vue`-імпорти, компілює `<script setup>` через `@vue/compiler-sfc`, повертає JS-модуль з `default` експортом компонента), `app/package.json` (додано `@vue/compiler-sfc`, `@types/bun`). Команда запуску: `bun test --preload ./test/happy-dom.preload.js src`.

---

## ADR Покриття коду: bun test --coverage (JS) + cargo-llvm-cov (Rust)

## Context and Problem Statement

Після міграції на `bun test` проект не мав жодної команди для вимірювання покриття коду. JS і Rust — окремі рантайми з різними інструментами. Потрібна одна команда, яка показує кількісний стан покриття в обох технологічних шарах.

## Considered Options

- `bun test --coverage` (JS) + `cargo llvm-cov` (Rust) + агрегований `COVERAGE.md`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "`bun test --coverage` (JS) + `cargo llvm-cov` (Rust) + агрегований `COVERAGE.md`", because user підтвердив, що хоче обидва інструменти й агрегований звіт в одному форматі.

### Consequences

- Good, because transcript фіксує очікувану користь: `COVERAGE.md` у корені проекту — git-trackable без таймстампів, зручно порівнювати покриття між комітами.
- Bad, because `cargo-llvm-cov` потребує `rustup component add llvm-tools-preview` та `cargo install cargo-llvm-cov` (~50с компіляції).

## More Information

Нові скрипти: `test:coverage` (`bun test --coverage --preload ./test/happy-dom.preload.js src`) та `test:rust:coverage` (`cargo llvm-cov --manifest-path src-tauri/Cargo.toml`) у `app/package.json`; `coverage` у кореневому `package.json` (викликає `scripts/coverage.js`). `scripts/coverage.js` парсить lcov (JS) та `cargo llvm-cov --json --summary-only` (Rust), рахує зважений підсумок, пише `COVERAGE.md`.

---

## ADR Агрегований звіт покриття: формат COVERAGE.md без таймстампа

## Context and Problem Statement

Перша реалізація `COVERAGE.md`, створена паралельною сесією, містила заголовок `# Coverage report — 2026-05-22 04:39:43`. Таймстамп у заголовку означав, що git-diff засмічується на кожен прогін, навіть якщо покриття не змінилось.

## Considered Options

- Markdown без таймстампа (git-diff рухається лише при зміні відсотків)
- Markdown із таймстампом (перший варіант паралельної сесії)

## Decision Outcome

Chosen option: "Markdown без таймстампа", because user прямо вибрав «порівнювати в гіті» і затвердив дизайн без таймстампа — «щоб git-diff рухався тільки коли реально змінилось покриття».

### Consequences

- Good, because transcript фіксує очікувану користь: `git diff COVERAGE.md` показує тільки реальні зміни покриття.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Формат таблиці: `| Область | Рядки | Функції |` з рядками JS (app), Rust (src-tauri), **Разом** (зважений підсумок). Файл `COVERAGE.md` у корені репозиторію, генерується `scripts/coverage.js`.

---

## ADR Мутаційне тестування: cargo-mutants (Rust) + рішення щодо StrykerJS (JS)

## Context and Problem Statement

Після налаштування кількісного покриття постало питання якісних метрик: чи справді тести щось перевіряють. User обрав mutation testing. У проекті є два технологічних шари — Rust (`src-tauri`) і JS/Vue (`app/src`).

## Considered Options

- Лише Rust (`cargo-mutants`) — JS не мутувати
- Обидва: `cargo-mutants` (Rust) + StrykerJS command-runner на `bun test` (JS/Vue)
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Обидва: `cargo-mutants` + StrykerJS з `inPlace: true` та `testRunner: command`", because user прямо обрав підхід C (усі мови, включно з `.vue`).

### Consequences

- Good, because transcript фіксує очікувану користь: охоплює всю логіку у `auth-store.js`, `auth-errors.js` (JS) та весь `src-tauri` (Rust).
- Bad, because StrykerJS із `inPlace: true` при force-kill залишає вихідні файли в інструментованому стані (виявлено в сесії: `stryMutAct_9fa48` у `auth-store.js`); Bun-рантайм викликав `Maximum call stack size exceeded` у Stryker dry-run — StrykerJS не завершив жодного прогону.

## More Information

Конфіг: `app/stryker.config.mjs` — `testRunner: command`, `commandRunner.command: bun test --preload ./test/happy-dom.preload.js src`, `inPlace: true`, `mutate: ["src/**/*.{js,vue}", "!src/**/*.test.js", "!src/test-utils/**", "!src/main.js"]`. `.gitignore` += `mutants.out/`, `app/reports/`. Скрипти: `test:mutation`, `test:rust:mutation` у `app/package.json`.

---

## ADR Стратегія оновлення COVERAGE.md: один скрипт, один прогін

## Context and Problem Statement

Первісний дизайн мав дві окремі команди — `coverage` і `mutation`. User попросив об'єднати їх в один скрипт, щоб не запускати окремо, і поєднати обидві секції (кількісну і мутаційну) в одну таблицю `COVERAGE.md`.

## Considered Options

- Один скрипт `scripts/coverage.js` — один прогін, одна таблиця
- Два окремі скрипти з двома секціями у `COVERAGE.md` (початковий варіант)

## Decision Outcome

Chosen option: "Один скрипт `scripts/coverage.js` — один прогін, одна таблиця", because user прямо сказав «я не буду окремо запускати scripts/coverage.js, тому поєднуй все в 1 скрипт» і затвердив формат таблиці `| Область | Рядки | Функції | Вбито мутацій | Score |`.

### Consequences

- Good, because transcript фіксує очікувану користь: один запуск `bun run coverage` дає повний стан проекту (coverage + mutation) в єдиному файлі.
- Bad, because один прогін тепер повільний (хвилини): coverage секунди + mutation хвилини; неможливо оновити лише coverage-секцію без перезапуску мутацій.

## More Information

Формат `COVERAGE.md`: `| Область | Рядки | Функції | Вбито мутацій | Score |` з рядками JS (app), Rust (src-tauri), **Разом**. Команда: `bun run coverage` → `bun scripts/coverage.js`.
