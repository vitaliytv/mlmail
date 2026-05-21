# Зведення ADR-впливів на C4-модель MLMaiL

`docs/ci4/decisions.md` — хронологічний індекс усіх архітектурних рішень MLMaiL і їхній вплив на рівні C4-моделі проєкту. Формальні ADR живуть у `docs/adr/`.

## Хронологічний індекс ADR MLMaiL

| Slug                                                                                       | Дата       | Статус   | Summary                                                                                                                |
| ------------------------------------------------------------------------------------------ | ---------- | -------- | ---------------------------------------------------------------------------------------------------------------------- |
| [ADR-0006-google-oauth](../adr/ADR-0006-google-oauth.md)                                   | 2026-05-11 | Accepted | Google OAuth-авторизація MLMaiL: Authorization Code + PKCE (macOS), Credential Manager (Android); токени зберігає Rust |
| [c4-документація-mlmail-ініціалізація](../adr/c4-документація-mlmail-ініціалізація.md)     | 2026-05-11 | Accepted | Ініціалізація C4-документації MLMaiL у `docs/ci4/` на основі реального коду                                            |
| [oauth-client-id-у-приватному-репо](../adr/oauth-client-id-у-приватному-репо.md)           | 2026-05-13 | Accepted | Google OAuth Client IDs MLMaiL у приватному репозиторії; Internal consent screen для `nitralabs.com`                   |
| [oauth-client-ids-runtime-dotenvy](../adr/oauth-client-ids-runtime-dotenvy.md)             | 2026-05-13 | Accepted | Читання OAuth Client IDs MLMaiL у runtime через `dotenvy` замість compile-time `option_env!()`                         |
| [gmail-inbox-count-стартовий-екран](../adr/gmail-inbox-count-стартовий-екран.md)           | 2026-05-14 | Accepted | Точна кількість листів INBOX MLMaiL через `users.labels.get`; HTTP-виклик у Rust                                       |
| [tauri-vs-capacitor-огляд](../adr/tauri-vs-capacitor-огляд.md)                             | 2026-05-14 | Accepted | Tauri обрано над Capacitor для MLMaiL: менший бінарник, Rust FFI, локальні ML-моделі                                   |
| [tauri-web-таргет-відсутність](../adr/tauri-web-таргет-відсутність.md)                     | 2026-05-14 | Accepted | Tauri MLMaiL не має web-цілі; web-розгортання — окремий Vite + шар `platform.ts`                                       |
| [gmail-random-message-команда](../adr/gmail-random-message-команда.md)                     | 2026-05-15 | Accepted | Команда `gmail_random_message` MLMaiL: вибірка з 100 листів INBOX, `text/plain` витяг                                  |
| [gmail-rust-команди-та-auth-store](../adr/gmail-rust-команди-та-auth-store.md)             | 2026-05-15 | Accepted | Gmail API-виклики MLMaiL у Rust Tauri-командах; Gmail-дані в `auth-store.js`                                           |
| [bun-run-lint-виправлення](../adr/bun-run-lint-виправлення.md)                             | 2026-05-16 | Accepted | Виправлення lint-ланцюжка `bun run lint` MLMaiL після `/n-fix`                                                         |
| [nitra-cursor-rules-compliance](../adr/nitra-cursor-rules-compliance.md)                   | 2026-05-16 | Accepted | Приведення проєкту MLMaiL до правил `@nitra/cursor` через `/n-fix`; `12/12 правил`                                     |
| [quasar-2-ui-фреймворк-mlmail](../adr/quasar-2-ui-фреймворк-mlmail.md)                     | 2026-05-16 | Accepted | Quasar 2 як UI-фреймворк MLMaiL з macOS material-look                                                                  |
| [tauri-e2e-macos-webdriver](../adr/tauri-e2e-macos-webdriver.md)                           | 2026-05-16 | Accepted | Tauri e2e MLMaiL на macOS не підтримується через WebDriver; `mock_builder` як альтернатива                             |
| [tauri-mock-runtime-тестування](../adr/tauri-mock-runtime-тестування.md)                   | 2026-05-16 | Accepted | Tauri Mock Runtime (`mock_builder`) як стратегія інтеграційного тестування MLMaiL на macOS                             |
| [vi-hoisted-vitest-mock](../adr/vi-hoisted-vitest-mock.md)                                 | 2026-05-16 | Accepted | `vi.hoisted()` обов'язковий для mock-об'єктів з `vi.fn()` у Vitest-тестах MLMaiL                                       |
| [adr-нормалізація-ручний-запуск](../adr/adr-нормалізація-ручний-запуск.md)                 | 2026-05-17 | Accepted | Ручний запуск ADR-нормалізації MLMaiL через env-var override (`THRESHOLD=0`, `MIN_INTERVAL=0`)                         |
| [bun-vitest-dual-runner](../adr/bun-vitest-dual-runner.md)                                 | 2026-05-17 | Accepted | Dual-runner MLMaiL: `bun:test` для pure-JS, Vitest для Vue SFC; витіснений повною міграцією на Bun                     |
| [gitleaks-security-лінтер](../adr/gitleaks-security-лінтер.md)                             | 2026-05-17 | Accepted | `gitleaks` як обов'язковий security-лінтер MLMaiL; `.gitleaks.toml` + `lint-security`                                  |
| [міграція-з-vitest-на-bun-test-runner](../adr/міграція-з-vitest-на-bun-test-runner.md)     | 2026-05-17 | Accepted | Повна міграція MLMaiL з Vitest на Bun Test Runner як єдиний test runner                                                |
| [nitra-cursor-видалення-з-підпакету](../adr/nitra-cursor-видалення-з-підпакету.md)         | 2026-05-17 | Accepted | Видалення `@nitra/cursor` з `app/package.json` MLMaiL; залишається лише у кореневому workspace                         |
| [normalize-decisions-batch-sonnet](../adr/normalize-decisions-batch-sonnet.md)             | 2026-05-17 | Accepted | `ADR_NORMALIZE_BATCH=10` для `normalize-decisions.sh` MLMaiL; витіснений рішенням BATCH=5                              |
| [q-page-flex-center-overflow](../adr/q-page-flex-center-overflow.md)                       | 2026-05-17 | Accepted | `q-page flex-center` MLMaiL приховує контент нижче viewport; замінено на `column items-center`                         |
| [rust-analyzer-workspace-підтека](../adr/rust-analyzer-workspace-підтека.md)               | 2026-05-17 | Accepted | `linkedProjects` у `.vscode/settings.json` MLMaiL для `rust-analyzer` у монорепо                                       |
| [tauri-storage-endpoints-managed-state](../adr/tauri-storage-endpoints-managed-state.md)   | 2026-05-17 | Accepted | Tauri Managed State (`Arc`) MLMaiL для `Storage` та `Endpoints` як DI для тестованості                                 |
| [файловий-стор-токенів-замість-keychain](../adr/файловий-стор-токенів-замість-keychain.md) | 2026-05-17 | Accepted | `FileStorage` (`session.json`, `0600`) MLMaiL замість macOS Keychain для зберігання токенів                            |
| [adr-нормалізація-розмір-батчу-batch5](../adr/adr-нормалізація-розмір-батчу-batch5.md)     | 2026-05-18 | Accepted | `ADR_NORMALIZE_BATCH=5` як стабільний розмір батчу `normalize-decisions.sh` MLMaiL                                     |

## Вплив ADR на рівні C4 MLMaiL

Для кожного ADR MLMaiL — проекції C4-моделі, на які цей ADR вплинув.

| ADR slug                               | 01-context | 02-containers | 03-components | 04-code |
| -------------------------------------- | :--------: | :-----------: | :-----------: | :-----: |
| ADR-0006-google-oauth                  |     ✓      |       ✓       |       ✓       |    ✓    |
| c4-документація-mlmail-ініціалізація   |     ✓      |       ✓       |       ✓       |    ✓    |
| oauth-client-id-у-приватному-репо      |     —      |       —       |       —       |    —    |
| oauth-client-ids-runtime-dotenvy       |     —      |       —       |       —       |    ✓    |
| gmail-inbox-count-стартовий-екран      |     —      |       —       |       ✓       |    ✓    |
| tauri-vs-capacitor-огляд               |     —      |       ✓       |       —       |    —    |
| tauri-web-таргет-відсутність           |     ✓      |       ✓       |       —       |    —    |
| gmail-random-message-команда           |     —      |       —       |       ✓       |    ✓    |
| gmail-rust-команди-та-auth-store       |     —      |       ✓       |       ✓       |    ✓    |
| bun-run-lint-виправлення               |     —      |       —       |       —       |    —    |
| nitra-cursor-rules-compliance          |     —      |       —       |       —       |    —    |
| quasar-2-ui-фреймворк-mlmail           |     —      |       —       |       ✓       |    ✓    |
| tauri-e2e-macos-webdriver              |     —      |       —       |       —       |    —    |
| tauri-mock-runtime-тестування          |     —      |       —       |       ✓       |    ✓    |
| vi-hoisted-vitest-mock                 |     —      |       —       |       —       |    —    |
| adr-нормалізація-ручний-запуск         |     —      |       —       |       —       |    —    |
| bun-vitest-dual-runner                 |     —      |       —       |       —       |    —    |
| gitleaks-security-лінтер               |     —      |       —       |       —       |    —    |
| міграція-з-vitest-на-bun-test-runner   |     —      |       —       |       —       |    —    |
| nitra-cursor-видалення-з-підпакету     |     —      |       —       |       —       |    —    |
| normalize-decisions-batch-sonnet       |     —      |       —       |       —       |    —    |
| q-page-flex-center-overflow            |     —      |       —       |       ✓       |    ✓    |
| rust-analyzer-workspace-підтека        |     —      |       —       |       —       |    —    |
| tauri-storage-endpoints-managed-state  |     —      |       —       |       ✓       |    ✓    |
| файловий-стор-токенів-замість-keychain |     —      |       ✓       |       ✓       |    ✓    |
| adr-нормалізація-розмір-батчу-batch5   |     —      |       —       |       —       |    —    |

## Зворотний індекс

Для кожного рівня C4-моделі MLMaiL — slug-и ADR, що його сформували.

### 01-context.md

- [ADR-0006-google-oauth](../adr/ADR-0006-google-oauth.md) — зовнішні системи Google Identity Services і Gmail API у System Context MLMaiL
- [c4-документація-mlmail-ініціалізація](../adr/c4-документація-mlmail-ініціалізація.md) — ініціалізація файлу рівня `01-context.md` MLMaiL
- [tauri-web-таргет-відсутність](../adr/tauri-web-таргет-відсутність.md) — MLMaiL розгортається як desktop/mobile без web-контейнера

### 02-containers.md

- [ADR-0006-google-oauth](../adr/ADR-0006-google-oauth.md) — HTTPS-стрілки до Google Identity Services перенесено з контейнера MLMaiL Frontend на контейнер MLMaiL Backend
- [c4-документація-mlmail-ініціалізація](../adr/c4-документація-mlmail-ініціалізація.md) — ініціалізація файлу рівня `02-containers.md` MLMaiL
- [gmail-rust-команди-та-auth-store](../adr/gmail-rust-команди-та-auth-store.md) — контейнер MLMaiL Backend виконує всі HTTP-виклики до Gmail API без передачі токенів у MLMaiL Frontend
- [tauri-vs-capacitor-огляд](../adr/tauri-vs-capacitor-огляд.md) — контейнер MLMaiL Backend реалізовано як Tauri + Rust, а не Capacitor
- [tauri-web-таргет-відсутність](../adr/tauri-web-таргет-відсутність.md) — відсутній web-контейнер у C4-діаграмі MLMaiL; Tauri розгортається лише на macOS і Android
- [файловий-стор-токенів-замість-keychain](../adr/файловий-стор-токенів-замість-keychain.md) — контейнер Локальне сховище MLMaiL зберігає `session.json` (`FileStorage`, `0600`)

### 03-components.md

- [ADR-0006-google-oauth](../adr/ADR-0006-google-oauth.md) — компоненти Auth Component MLMaiL, Auth Store MLMaiL, Auth Module MLMaiL (Rust), Auth Errors i18n MLMaiL
- [c4-документація-mlmail-ініціалізація](../adr/c4-документація-mlmail-ініціалізація.md) — ініціалізація файлу рівня `03-components.md` MLMaiL
- [gmail-inbox-count-стартовий-екран](../adr/gmail-inbox-count-стартовий-екран.md) — компонент Gmail Module MLMaiL (Rust) з командою `gmail_inbox_count`; Auth Store MLMaiL отримав `inboxCount`, `refreshInboxCount`
- [gmail-random-message-команда](../adr/gmail-random-message-команда.md) — компонент Gmail Module MLMaiL отримав команду `gmail_random_message`; Auth Store MLMaiL — `currentMessage`, `loadRandomMessage`
- [gmail-rust-команди-та-auth-store](../adr/gmail-rust-команди-та-auth-store.md) — структура компонента Auth Store MLMaiL з Gmail-даними замість окремого `mailbox-store.js`
- [q-page-flex-center-overflow](../adr/q-page-flex-center-overflow.md) — виправлення поведінки компонента Auth Component MLMaiL (`Login.vue`): `column items-center` замість `flex-center`
- [quasar-2-ui-фреймворк-mlmail](../adr/quasar-2-ui-фреймворк-mlmail.md) — компонент Frontend UI Kit MLMaiL (Quasar 2) з macOS material-look
- [tauri-mock-runtime-тестування](../adr/tauri-mock-runtime-тестування.md) — DI-структура Tauri-команд MLMaiL через `SharedStorage` і `Endpoints` (Managed State)
- [tauri-storage-endpoints-managed-state](../adr/tauri-storage-endpoints-managed-state.md) — компоненти SharedStorage MLMaiL і Endpoints MLMaiL як `Arc`-об'єкти у Tauri Managed State
- [файловий-стор-токенів-замість-keychain](../adr/файловий-стор-токенів-замість-keychain.md) — компонент Auth Module MLMaiL: `FileStorage` замість macOS Keychain

### 04-code.md

- [ADR-0006-google-oauth](../adr/ADR-0006-google-oauth.md) — нові файли `app/src-tauri/src/auth/`, Kotlin-файли `gen/android/.../auth/`, `auth-store.js`, `Login.vue`, `auth-errors.js`
- [c4-документація-mlmail-ініціалізація](../adr/c4-документація-mlmail-ініціалізація.md) — ініціалізація файлу рівня `04-code.md` MLMaiL
- [gmail-inbox-count-стартовий-екран](../adr/gmail-inbox-count-стартовий-екран.md) — `app/src-tauri/src/gmail/mod.rs` (новий), оновлення `lib.rs`, `auth-store.js`
- [gmail-random-message-команда](../adr/gmail-random-message-команда.md) — `app/src-tauri/src/gmail/message.rs` (новий), оновлення `lib.rs`, `auth-store.js`, `Login.vue`
- [gmail-rust-команди-та-auth-store](../adr/gmail-rust-команди-та-auth-store.md) — структура `gmail/mod.rs`, `gmail/error.rs`, оновлення `auth-store.js`
- [oauth-client-ids-runtime-dotenvy](../adr/oauth-client-ids-runtime-dotenvy.md) — `app/src-tauri/src/auth/config.rs`: runtime-функції замість compile-time констант; `dotenvy::dotenv()` у `lib.rs`
- [q-page-flex-center-overflow](../adr/q-page-flex-center-overflow.md) — виправлення класу `q-page` у `app/src/views/Login.vue`
- [quasar-2-ui-фреймворк-mlmail](../adr/quasar-2-ui-фреймворк-mlmail.md) — `quasar-variables.sass`, `main.js`, `vite.config.js`, `App.vue`, `Login.vue`, `test-utils/quasar.js`
- [tauri-mock-runtime-тестування](../adr/tauri-mock-runtime-тестування.md) — `app/src-tauri/src/endpoints.rs` (новий), `tests/*.rs` (5 нових файлів)
- [tauri-storage-endpoints-managed-state](../adr/tauri-storage-endpoints-managed-state.md) — `endpoints.rs`, `auth/storage/mod.rs`, `gmail/mod.rs`, `lib.rs`
- [файловий-стор-токенів-замість-keychain](../adr/файловий-стор-токенів-замість-keychain.md) — `app/src-tauri/src/auth/storage/file.rs` (новий), `storage/mod.rs`

## Superseded chains

Ланцюжок замін розміру батчу `normalize-decisions.sh` MLMaiL:

`normalize-decisions-batch-sonnet` (BATCH=10) → `adr-нормалізація-розмір-батчу-batch5` (BATCH=5)

ADR `normalize-decisions-batch-sonnet` зафіксував `BATCH=10` як відповідь на збій `BATCH=30` (перевищення output-токенів). Реальний прогін з `BATCH=10` завершився `Request timed out` (~1 год). ADR `adr-нормалізація-розмір-батчу-batch5` закріплює остаточне рішення: `BATCH=5` стабільно завершує прогін у межах output-токенів і часу виконання `claude` CLI.

Ланцюжок замін стратегії test runner MLMaiL:

`bun-vitest-dual-runner` (bun:test + Vitest) → `міграція-з-vitest-на-bun-test-runner` (bun:test єдиний)

ADR `bun-vitest-dual-runner` запропонував розподіл: `bun:test` для pure-JS із `*.test.js`-конвенцією, Vitest для Vue SFC із `*.vitest.js`. Реальний стан коду вже спирався виключно на `bun:test`/`mock.module()` — повернення до Vitest потребувало б рефакторингу. ADR `міграція-з-vitest-на-bun-test-runner` закріплює `bun:test` як єдиний runner і вилучає `vitest` з `devDependencies`.

## Правило синхронізації MLMaiL

Будь-яка зміна, що впливає на індекс вище — новий ADR у `docs/adr/`, новий зовнішній сервіс MLMaiL, нова ключова залежність — оновлює:

1. цей файл (`docs/ci4/decisions.md`);
2. відповідний рівень C4-моделі MLMaiL (`01-context.md`, `02-containers.md`, `03-components.md`, `04-code.md`);
3. тіло самого ADR з явним переліком C4-проекцій для оновлення (правило `.cursor/rules/n-adr.mdc`).

Усе у тому ж PR — розсинхрон між кодом, ADR і C4-моделлю MLMaiL заборонено.
