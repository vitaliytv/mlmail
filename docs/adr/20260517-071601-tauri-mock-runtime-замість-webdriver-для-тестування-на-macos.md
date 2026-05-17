---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-17T07:16:01+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

## ADR Tauri Mock Runtime замість WebDriver для тестування на macOS

**Контекст:** Проєкт MLMaiL — Tauri v2 десктоп-застосунок для macOS; потрібне функціональне покриття команд (auth, gmail), але macOS не підтримує WebKit WebDriver, а Google OAuth через браузер неможливо стабільно автоматизувати в CI.

**Рішення/Процедура/Факт:** Обрано офіційний стек Tauri для macOS: `tauri::test::mock_builder` + `mock_context` + `noop_assets`. Додано `tauri = { features = ["test"] }` у `[dev-dependencies]`. Написано 19 інтеграційних тестів у `src-tauri/tests/`: `auth_commands.rs`, `auth_logout.rs`, `auth_finalize_login.rs`, `acquire_access_token.rs`, `gmail_commands.rs`. Покриті: `auth_is_authenticated`, `auth_current_email`, `auth_logout`, `finalize_login` (success + 3 error-кейси), `acquire_access_token` (fast path, refresh, rotation, `invalid_grant`), `gmail_inbox_count` (200, 401), `gmail_random_message` (success, empty).

**Обґрунтування:** Tauri офіційно заявляє: "On desktop, only Windows and Linux are supported due to macOS not having a WKWebView driver tool available." Appium/XCUITest — community-хак без підтримки від Tauri team. Mock Runtime — єдиний офіційний шлях для macOS, дозволяє тестувати `State<>` + `AppHandle` без реального webview і без залежності від Google OAuth.

**Розглянуті альтернативи:** WebDriver (Linux-only, потребує Docker на Mac), Appium + XCUITest (community, не покриває keyring macOS-нативно в тесті), Playwright проти Vite dev-server (не тестує Tauri runtime), Vitest-мок `@tauri-apps/api/core` (покриває лише фронт, не Rust-логіку).

**Зачіпає:** `src-tauri/Cargo.toml` (dev-dep), `src-tauri/tests/*.rs` (новий каталог), `src-tauri/src/auth/mod.rs`, `src-tauri/src/gmail/mod.rs`, `src-tauri/src/lib.rs`.

---

## ADR Storage і Endpoints як Tauri managed state (DI для тестованості)

**Контекст:** `auth_logout`, `acquire_access_token` і gmail-команди мали зашиті залежності: keyring-storage через `#[cfg(target_os)]` і константи Google URLs, що унеможливлювало підміну у тестах без змін у runtime-коді.

**Рішення/Процедура/Факт:** `Box<dyn RefreshTokenStorage>` → `Arc<dyn RefreshTokenStorage>` під аліасом `SharedStorage`; тип реєструється через `.manage()` у `lib.rs::run()`. Додано новий файл `src/endpoints.rs` зі структурою `Endpoints { google_token, gmail_label_inbox, gmail_messages_list }` і `impl Default` з реальними Google URLs; реєструється через `.manage(Endpoints::default())`. `acquire_access_token` і `finalize_login` тепер приймають URL-рядок і `&dyn RefreshTokenStorage` явно — не беруть їх зсередини. У тестах підставляється `InMemoryStorage` (вже існував) і `mockito`-сервер замість Google.

**Обґрунтування:** Без DI неможливо протестувати жоден шлях, що зачіпає keyring або зовнішній HTTP, без реального Keychain і мережі. Tauri managed state — стандартний механізм ін'єкції для команд; перехід до `Arc` обов'язковий, бо `State<>` за Tauri вимагає `Send + Sync + 'static`.

**Розглянуті альтернативи:** `#[cfg(test)]` підміна (погано масштабується, засмічує prod-код), env-var у runtime з `OnceLock` (обговорювалося раніше, але managed state виразніший і типово-безпечний).

**Зачіпає:** `src-tauri/src/auth/storage/mod.rs` (SharedStorage), `src-tauri/src/endpoints.rs` (новий), `src-tauri/src/auth/mod.rs` (finalize_login, acquire_access_token), `src-tauri/src/gmail/mod.rs`, `src-tauri/src/lib.rs`.

---

## ADR Розділення тест-раннерів: bun:test для pure-JS, Vitest для Vue SFC

**Контекст:** `bun test` фейлив на існуючих Vitest-тестах через відсутність Vite plugin pipeline (unplugin-auto-import не виконується, `vi.fn()` / `vi.mock()` відсутні в bun:test API). Хотілося щоб `bun test` як команда працював.

**Рішення/Процедура/Факт:** Тести розбито на дві групи: `bun test src/services src/i18n` — pure-JS тести через `bun:test` API (`mock`, `mock.module`), `vitest run` — тести з Vue SFC (`.vitest.js`). У `auth-store.js` додано явний `import { ref, readonly } from 'vue'` замість auto-import. `auth-store.test.js` і `auth-errors.test.js` переписані на `bun:test` (`mock()`, `mock.module()`). `Login.test.js` перейменовано в `Login.vitest.js` і залишається під Vitest. Скрипти в `package.json`: `test` → `bun test src/...`, `test:ui` → `vitest run`, `test:all` → обидва + `cargo test`.

**Обґрунтування:** Повна міграція на `bun:test` ризикована через відсутність стабільного bun-plugin-vue і несумісність з Quasar. Гібридний підхід дозволяє запускати `bun test` для бізнес-логіки (швидко, без Vite transform) і зберігає Vitest там де потрібен Vue compiler.

**Розглянуті альтернативи:** Повна міграція на `bun:test` (ризик з `.vue` компіляцією, ~3-4 год, без гарантії), `bunx vitest` (нуль змін, але `bun test` як команда не працює).

**Зачіпає:** `src/services/auth-store.js`, `src/services/auth-store.test.js`, `src/i18n/auth-errors.test.js`, `src/views/Login.vitest.js` (перейменовано), `package.json` (scripts), `vite.config.js` (Vitest include).
