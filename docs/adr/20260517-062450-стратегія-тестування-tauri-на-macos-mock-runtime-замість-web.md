---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-17T06:24:50+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

## ADR Стратегія тестування Tauri на macOS: Mock Runtime замість WebDriver

**Контекст:** Потрібно покрити Tauri-команди інтеграційними тестами на macOS; WebDriver (`tauri-driver`) офіційно не підтримується на macOS через відсутність WKWebView driver від Apple.

**Рішення/Процедура/Факт:** Прийнято офіційну рекомендацію Tauri — `tauri::test::mock_builder` + `mock_context` + `noop_assets`. Паралельно виконано рефакторинг Rust-коду для DI: `Box<dyn RefreshTokenStorage>` → `Arc<dyn ...>` як `SharedStorage` у Tauri managed state; введено `endpoints.rs` зі структурою `Endpoints { google_token, gmail_label_inbox, gmail_messages_list }` як другий managed state. Команди `auth_*` і `gmail_*` тепер беруть `State<SharedStorage>` і `State<Endpoints>` замість hardcoded URLs і `make_storage()` всередині. Виділено `finalize_login(resp, &dyn storage, &Mutex<state>)` як публічний тестований хелпер для внутрішньої логіки `auth_start_login`. Додано 19 інтеграційних тестів у `src-tauri/tests/` і скрипти `test:rust` / `test:all` у `package.json`.

**Обґрунтування:** macOS не дає WebKit WebDriver для WKWebView — це системне обмеження Apple, не Tauri. Appium (альтернатива) не підтримує CSS/DOM-селектори і потребує Accessibility API. Mock Runtime тестує реальну бізнес-логіку команд (state machine, token refresh, storage) без UI і без Google Network, на рідній продовій платформі (зокрема реальний Keychain замінюється `InMemoryStorage` лише в тестах). DI через managed state — мінімальний рефакторинг, що не змінює публічний API команд і сумісний з `tauri::State` у виробничому коді.

**Розглянуті альтернативи:** (1) `tauri-driver` у Linux/Docker — офіційний WebDriver, але тестує Linux-збірку, не macOS-продову платформу. (2) Appium + XCUITest — реальне вікно на macOS, але лише Accessibility-селектори, без DOM, повільніше, community-рішення. (3) Playwright проти Vite dev-сервера — мок `window.__TAURI_INTERNALS__`, лише фронтенд без Rust-логіки.

**Зачіпає:** `src-tauri/src/auth/mod.rs`, `src-tauri/src/auth/storage/mod.rs`, `src-tauri/src/gmail/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/endpoints.rs` (новий), `src-tauri/tests/auth_commands.rs`, `tests/auth_logout.rs`, `tests/auth_finalize_login.rs`, `tests/acquire_access_token.rs`, `tests/gmail_commands.rs`, `app/package.json` (скрипти `test:rust`, `test:all`), `src-tauri/Cargo.toml` (`tauri = { features = ["test"] }` у dev-dependencies).
