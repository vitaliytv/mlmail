---
session: e44bf0ca-9c07-4840-b7a0-defdeeff62a4
captured: 2026-05-16T16:34:07+03:00
transcript: /Users/vitaliytv/.claude/projects/-Users-vitaliytv-www-vitaliytv-mlmail/e44bf0ca-9c07-4840-b7a0-defdeeff62a4.jsonl
---

Всі 64 тести вже є і проходять — код вже набагато повніший, ніж показував транскрипт. Наша зміна (`token_url` як параметр `exchange_code`) вже закомічена в `exchange_code_at`.

---

## ADR Tauri e2e на macOS — вибір Mock Runtime замість WebDriver

**Контекст:** Під час планування e2e-тестування Tauri v2 + Vue 3 застосунку (mlmail) виникло питання як тестувати реальний UI на macOS — основній платформі розробки.

**Рішення/Процедура/Факт:** Обрано **Tauri Mock Runtime** (`tauri::test::mock_builder`) як офіційний підхід для тестування команд без реального webview. Тести розміщуються у `#[cfg(test)] mod tests` блоках всередині модулів (`src/auth/`, `src/gmail/`), а не в зовнішньому `tests/`. Для підтримки mockito-тестування HTTP-ендпоінтів Google `exchange_code` рефакторовано на `exchange_code_at(url, ...)` — URL передається параметром, дефолт `config::TOKEN_URL` залишається у виробничому коді. Dev-залежності: `tauri = { features = ["test"] }`, `tokio = { features = ["macros", "rt"] }`.

**Обґрунтування:** Tauri v2 офіційно не підтримує WebDriver на macOS (`"macOS not having a WKWebView driver tool available"`). Appium/XCUITest — community-хак без офіційної підтримки. Mock Runtime покриває всі `#[tauri::command]` функції без реального вікна, працює нативно на macOS, дозволяє тестувати keyring (apple-native), і вже дає 64 тести для auth + gmail логіки.

**Розглянуті альтернативи:**
- Linux Docker + `tauri-driver` + WebDriverIO — офіційний WebDriver, але не тестує macOS keyring і потребує Docker
- Appium + XCUITest — видиме вікно на macOS через Accessibility API, але немає DOM-селекторів і немає офіційної підтримки Tauri
- Playwright проти Vite dev-сервера — мокає `window.__TAURI_INTERNALS__`, не тестує Rust-бекенд

**Зачіпає:** `app/src-tauri/Cargo.toml` (`[dev-dependencies]`), `src/auth/token_exchange.rs` (сигнатура `exchange_code_at`), `src/auth/mod.rs`, `src/gmail/mod.rs`, `src/auth/storage/in_memory.rs`
