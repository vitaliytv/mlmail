---
session: a8164e48-3940-47fa-99f6-2db9a40bb757
captured: 2026-05-22T20:29:06+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/a8164e48-3940-47fa-99f6-2db9a40bb757.jsonl
---

## ADR Міграція тестового середовища Vue з jsdom/vitest на happy-dom/bun:test

## Context and Problem Statement

Правило `n-vue.mdc` забороняє використання `jsdom` і `vitest` у воркспейсі `app`. При запуску `npx @nitra/cursor check` правило `vue` повертало `❌`, бо `app/package.json` містив ці залежності, а `app/vite.config.js` — блок конфігурації `test` для Vitest.

## Considered Options

- Замінити jsdom на `@happy-dom/global-registrator` і vitest на `bun:test`
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Замінити jsdom на `@happy-dom/global-registrator` і vitest на `bun:test`", because правило `n-vue.mdc` явно забороняє jsdom і vitest, і `bun:test` вже є стандартним ранером у проєкті.

### Consequences

- Good, because `npx @nitra/cursor check` перейшов із 11/12 на 12/12 правил без зауважень.
- Bad, because `bun test` не компілює `.vue` SFC нативно — `Login.vitest.js` не можна просто перейменувати, потрібен окремий Bun-плагін компілятора (задокументовано в наступному ADR).

## More Information

- Видалено залежності: `jsdom`, `vitest` з `app/package.json`
- Додано: `@happy-dom/global-registrator`, `@vue/compiler-sfc`, `@types/bun` у `app/package.json`
- Прибрано блок `test` з `app/vite.config.js`
- `app/src/views/Login.vitest.js` → `app/src/views/Login.test.js` (API `vitest` → `bun:test`)
- Скрипт `test`: `bun test --preload ./test/happy-dom.preload.js src`

---

## ADR Bun-плагін компіляції Vue SFC для bun:test

## Context and Problem Statement

Після міграції на `bun:test` виявилось, що Bun завантажує `.vue`-файли як рядки (`typeof module.default === 'string'`), а не як компоненти Vue. Без SFC-компілятора всі тести компонентів падали з `ReferenceError: ref is not defined` та помилками монтування.

## Considered Options

- Налаштувати `Bun.plugin` із `@vue/compiler-sfc` у preload-файлі — компілювати `.vue` на льоту при завантаженні
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Налаштувати `Bun.plugin` із `@vue/compiler-sfc` у preload-файлі", because це єдиний офіційний механізм розширення Bun для обробки нестандартних типів файлів без зовнішніх інструментів.

### Consequences

- Good, because `bun test` компілює `.vue` у компоненти Vue, тести App.vue і Login.vue проходять; 45 → 47 pass / 0 fail.
- Bad, because Quasar завантажується у серверній збірці (`quasar.server.prod.js`) за замовчуванням коли Bun резолвить пакет через умову `node`. Щоб дістати browser-збірку (`quasar.client.js`), довелося замокати модуль через `mock.module('quasar', ...)` у preload.

## More Information

- `app/test/happy-dom.preload.js` — реєструє happy-dom, Bun-плагін SFC, авто-імпорти Vue/Vue Router як глобали, форсує browser-збірку Quasar через `mock.module`
- `app/src/test-utils/quasar.js` — хелпер `mountWithQuasar`/`mountQuasar`, реєструє всі Quasar-компоненти глобально (немає `@quasar/vite-plugin` під `bun test`)
- `happy-dom` реєструється до імпорту Vue runtime-dom — runtime-dom захоплює `document` у момент завантаження

---

## ADR Мутаційне тестування: cargo-mutants (Rust) + StrykerJS (JS + Vue script)

## Context and Problem Statement

Кількісні метрики покриття (`bun test --coverage`, `cargo llvm-cov`) вимірюють лише чи код _виконувався_, але не чи тести справді _перевіряють_ поведінку. Виникла потреба в якісній метриці.

## Considered Options

- Mutation testing: cargo-mutants для Rust, StrykerJS для JS/Vue
- Assertion-density аналіз (підрахунок `expect()` на рядок коду)
- Аналіз гілок вручну
- Інші варіанти в transcript не обговорювалися в деталях.

## Decision Outcome

Chosen option: "Mutation testing: cargo-mutants для Rust, StrykerJS для JS/Vue", because мутаційне тестування — єдина метрика, яка відповідає на питання «чи тест впаде при поломці коду?».

### Consequences

- Good, because `cargo-mutants` виявив 31 незловлену мутацію зі 80 загальних по `src-tauri` — mutation score 46% перед виправленням; Stryker покриває `auth-store.js`, `auth-errors.js` і `<script setup>` у `.vue` SFC.
- Bad, because StrykerJS не має нативного раннера для `bun test`, тому використовується command-runner з `inPlace: true`; прогін повільний — кілька хвилин проти секунд для звичайного тесту.

## More Information

- `app/stryker.config.mjs`: `testRunner: 'command'`, `commandRunner: { command: 'bun test --preload ./test/happy-dom.preload.js src' }`, `inPlace: true`, `mutate: ['src/**/*.{js,vue}', '!src/main.js', '!src/**/*.test.js', '!src/test-utils/**']`
- `app/package.json` скрипти: `test:mutation` (`bunx stryker run`), `test:rust:mutation` (`cargo mutants --manifest-path src-tauri/Cargo.toml`)
- StrykerJS вміє мутувати `<script>` блоки у `.vue` SFC (актуальна документація https://stryker-mutator.io/docs/stryker-js/guides/vuejs/)
- `.gitignore`: `mutants.out/`, `app/reports/`

---

## ADR Єдина команда coverage: один скрипт, одна таблиця, без таймстампів

## Context and Problem Statement

Після додавання і кількісного (lcov, llvm-cov) і мутаційного (Stryker, cargo-mutants) інструментів з'явилось чотири окремих команди. Також виникла проблема таймстампів у `COVERAGE.md` — кожен запуск генерував `git diff` навіть без змін у покритті.

## Considered Options

- Один скрипт `scripts/coverage.js`, що запускає всі чотири прогони послідовно та пише єдиний `COVERAGE.md`
- Два окремих скрипти: `coverage` (швидко) і `mutation` (повільно), кожен оновлює свою секцію
- Нативні команди без агрегатора

## Decision Outcome

Chosen option: "Один скрипт `scripts/coverage.js`", because користувач явно вказав «поєднуй все в 1 скрипт» і не хоче запускати команди окремо.

### Consequences

- Good, because `bun run coverage` у корені дає повну картину якості тестів однією командою; `COVERAGE.md` без таймстампів → `git diff` рухається тільки при реальній зміні покриття.
- Bad, because один прогін тривалий (мутаційне тестування займає хвилини) — неможливо швидко оновити тільки кількісне покриття без прогону мутацій.

## More Information

- `COVERAGE.md` — єдина таблиця: `| Область | Рядки | Функції | Вбито мутацій | Score |`
- `package.json` (корінь) скрипт: `"coverage": "bun scripts/coverage.js"`
- `scripts/coverage.js` парсить lcov (`LF`/`LH`/`FNF`/`FNH`), JSON із `cargo llvm-cov --json --summary-only`, JSON із `cargo mutants --json`, stdout із Stryker (`Mutation score`)
