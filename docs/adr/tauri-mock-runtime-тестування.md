# Tauri Mock Runtime як стратегія тестування на macOS

**Status:** Accepted
**Date:** 2026-05-16

## Контекст

Проєкт MLMaiL — Tauri v2 + Vue 3 на macOS. Необхідно покрити integration-тестами Rust-команди (`auth_*`, `gmail_*`), але `tauri-driver` WebDriver не підтримує macOS.

## Рішення/Процедура/Факт

Обрано Tauri Mock Runtime (`tauri::test::mock_builder`) як офіційний підхід для тестування команд без реального webview:

- До `[dev-dependencies]` додано `tauri = { features = ["test"] }` і `tokio = { features = ["macros", "rt"] }`.
- Введено `SharedStorage = Arc<dyn RefreshTokenStorage>` у `auth/storage/mod.rs` — storage керується через `manage()`/`State<>` замість створення всередині команд.
- Створено `src/endpoints.rs` — `Endpoints { google_token, gmail_label_inbox, gmail_messages_list }` з `Default` (реальні Google URLs), що керується через `State<Endpoints>`. Дозволяє підмінити URL у тестах на mockito-адресу.
- Команди `auth_start_login`, `auth_logout`, `acquire_access_token`, `gmail_inbox_count`, `gmail_random_message` рефакторовано — приймають `State<SharedStorage>` і `State<Endpoints>`.
- Написано 5 тест-файлів у `src-tauri/tests/`: `auth_commands.rs`, `auth_logout.rs`, `auth_finalize_login.rs`, `acquire_access_token.rs`, `gmail_commands.rs`. Використовують `mock_builder()` + `mock_context()` + `mockito`.

## Обґрунтування

Mock Runtime покриває всі `#[tauri::command]` зі справжнім `State<>` DI без webview, працює нативно на macOS і тестує Keychain (apple-native). Appium mac2 потребує AX-селекторів і складнішої інфраструктури. Docker + Linux не тестує macOS keychain.

## Розглянуті альтернативи

- WebDriverIO + `tauri-driver` на Linux/Docker — офіційно підтримується, але не тестує macOS Keychain.
- Appium + XCUITest — community-рішення через Accessibility API, без офіційної підтримки Tauri.
- Виключно Vitest + mock `invoke` — вже є, але не покриває Rust-рівень.

## Зачіпає

`src-tauri/src/endpoints.rs` (новий), `src-tauri/src/auth/mod.rs`, `src-tauri/src/auth/storage/mod.rs`, `src-tauri/src/gmail/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/tests/` (5 нових файлів).

---

**Опрацьовано** 2026-05-19. Проекції:
- [01-context](../ci4/01-context.md)
- [02-containers](../ci4/02-containers.md)
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
