---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-17T07:43:34+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

---

З наведеного транскрипту виконано значну проектну роботу. Генерую артефакти.

## ADR Тестування Tauri e2e на macOS: вибір Mock Runtime замість WebDriver

**Контекст:** Проєкт mlmail (Tauri v2 + Vue 3 + Quasar) потребував e2e-тестування на macOS, де `tauri-driver` офіційно не підтримується — Apple не надає WKWebView WebDriver. Потрібно було обрати підхід для покриття Tauri-команд на macOS без реального UI-драйвера.

**Рішення/Процедура/Факт:** Обрано офіційний шлях `tauri::test::mock_builder` / `mock_context` / `noop_assets` — інтеграційні тести Rust-команд без реального webview. Виконано три етапи: (1) додано `tauri = { version = "2", features = ["test"] }` у `[dev-dependencies]`; (2) storage перенесено з `Box<dyn ...>` → `Arc<dyn ...>` (`SharedStorage`) як Tauri managed state, що дало можливість підставляти `InMemoryStorage` в тестах; (3) зовнішні URLs (`google_token`, `gmail_label_inbox`, `gmail_messages_list`) винесено у новий `Endpoints` managed state. Введено `finalize_login` як testable public хелпер з auth-логіки. Написано 19 нових integration-тестів у `src-tauri/tests/` (auth_commands, auth_logout, auth_finalize_login, acquire_access_token, gmail_commands). Разом: 83 Rust-тести проходять.

**Обґрунтування:** Офіційна документація Tauri v2 прямо констатує "only Windows and Linux are supported" для WebDriver. Appium + XCUITest як альтернатива надто дорогий у налаштуванні і не дає DOM-селекторів. Mock Runtime покриває 90%+ реальних багів (бізнес-логіка Rust-команд) за ~4 год роботи замість ~0.5 дня Docker+WebDriver інфраструктури + OAuth-стабу.

**Розглянуті альтернативи:** Linux WebDriver у Docker (відмовлено — складна інфраструктура, не тестує keyring/native macOS); Appium + XCUITest (відмовлено — community-хак, AX-селектори замість DOM, ~30 хв setup, але без DOM); Playwright проти Vite dev-server без Tauri (відмовлено — не e2e, не тестує Rust-команди).

**Зачіпає:** `app/src-tauri/Cargo.toml`, `app/src-tauri/src/auth/mod.rs`, `app/src-tauri/src/auth/storage/mod.rs`, `app/src-tauri/src/gmail/mod.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/src/endpoints.rs` (новий), `app/src-tauri/tests/` (5 нових файлів).

---

## ADR Dual-runner стратегія frontend-тестів: bun:test + Vitest

**Контекст:** Frontend-тести запускались лише через Vitest, але `bun test` (bun's native runner) фейлив через auto-imports (`ref`/`readonly` без `import`), несумісний API (`vi.fn()` vs `mock()`), і відсутність Vue SFC compilation pipeline. Потрібно було зробити `bun test` функціональним.

**Рішення/Процедура/Факт:** Прийнято dual-runner підхід: `*.test.js` — bun:test (pure JS логіка), `*.vitest.js` — Vitest (Vue SFC + Quasar). Кроки: (1) додано explicit `import { readonly, ref } from 'vue'` в `auth-store.js` (auto-imports прибрано); (2) `auth-store.test.js` і `auth-errors.test.js` переписано під `bun:test` API (`mock()`, `mock.module()`); (3) `Login.test.js` перейменовано в `Login.vitest.js`; (4) `vite.config.js` звужено до `include: ['src/**/*.vitest.{js,vue}']`; (5) в `package.json` додано скрипти `test` (bun:test), `test:ui` (vitest), `test:rust` (cargo), `test:all` (усі три). Результат: `bun test src/services src/i18n` — 34 тести, `bun run test:ui` — 11 тестів, `bun run test:all` — 128 тестів разом.

**Обґрунтування:** `bun test` швидший за Vitest для pure JS логіки, але не може компілювати `.vue` + Quasar plugin pipeline. Розподіл за розширенням файлів (`.test.js` vs `.vitest.js`) дає чітку конвенцію без необхідності окремих конфіг-файлів.

**Розглянуті альтернативи:** Повна міграція на `bun:test` (відмовлено — Vue SFC + Quasar потребують Vite pipeline, ризик поламати Login.test); `bunx vitest` замість `bun test` (0 змін, але не вирішує мету "bun test працює").

**Зачіпає:** `app/src/services/auth-store.js`, `app/src/services/auth-store.test.js`, `app/src/i18n/auth-errors.test.js`, `app/src/views/Login.test.js` → `Login.vitest.js`, `app/vite.config.js`, `app/package.json`.

---

## Knowledge Іконки Quasar: prefixed names для material-symbols-outlined

**Контекст:** У Quasar 2 підключено `@quasar/extras/material-symbols-outlined` і `iconSet: 'material-symbols-outlined'`, але іконки в шаблонах використовувались під старими іменами (`mail`, `refresh`, `logout`, `login`). В UI відображався літеральний текст замість гліфів.

**Рішення/Процедура/Факт:** Quasar 2 вимагає prefix-нотацію для символьних шрифтів: `sym_o_` для material-symbols-outlined. Без префікса Quasar генерує клас `.material-icons`, а шрифт `material-symbols-outlined` реєструється під іншим CSS-класом. Правильні назви: `sym_o_mail`, `sym_o_refresh`, `sym_o_logout`, `sym_o_login`.

**Обґрунтування:** Архітектурне рішення не приймалося — це відомий API Quasar 2. Нотатка фіксує ненайочевидний факт: `iconSet` у конфігурації змінює дефолт, але не перехоплює iconName у template-атрибутах без правильного префікса.

**Розглянуті альтернативи:** не обговорювались.

**Зачіпає:** `app/src/views/Login.vue`, `app/src/main.js`.
