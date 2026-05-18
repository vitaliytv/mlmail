# Читання OAuth Client IDs у runtime через dotenvy

**Status:** Accepted
**Date:** 2026-05-13

## Context and Problem Statement

MLMaiL потребує трьох Google OAuth Client IDs (Desktop, Android, Web), що зберігаються у `app/src-tauri/.env`. Початкова реалізація `config.rs` використовувала макрос `option_env!()`, який читає значення env-змінних під час компіляції. Якщо env не були виставлені у shell під час `cargo build`, у бінарнику залишався буквальний fallback-рядок `REPLACE_ME_DESKTOP_CLIENT_ID` — навіть якщо `.env` вже містив реальні значення. Результат: `401 invalid_client` від Google під час першого тестового логіну на macOS.

## Considered Options

- `option_env!()` compile-time з обов'язковим `source .env` перед кожним `cargo build`
- `std::env::var()` + `dotenvy` runtime-читання з `OnceLock<String>` кешуванням
- Передача Client IDs через `tauri::Context` або `tauri.conf.json`

## Decision Outcome

Chosen option: "`std::env::var()` + `dotenvy` runtime", because зміни у `.env` підхоплюються наступним запуском процесу без перекомпіляції, що відповідає призначенню `.env` як гарячої конфігурації для локальної розробки.

### Consequences

- Good, because після зміни `.env` достатньо перезапустити застосунок — `cargo clean` не потрібен.
- Good, because `OnceLock<String>` кешує значення після першого виклику без performance overhead при кожному зверненні до config.
- Bad, because помилки відсутнього env var виявляються в runtime, а не в compile time.
- Neutral, because `dotenvy::dotenv().ok()` викликається першим у `lib.rs::run()` до ініціалізації Tauri — порядок виклику задокументований у коді.

## More Information

Змінені файли:

- `app/src-tauri/src/auth/config.rs` — `option_env!()` замінено на три функції `desktop_client_id()`, `android_client_id()`, `android_web_client_id()` через `std::env::var()` + `OnceLock<String>`; константи прибрані.
- `app/src-tauri/src/auth/mod.rs` — споживачі переведені з констант на виклики функцій.
- `app/src-tauri/src/lib.rs` — `dotenvy::dotenv().ok()` до `tauri::Builder`.
- `app/src-tauri/Cargo.toml` — додано `dotenvy = "0.15.7"`.

Відкинуто "Передача через `tauri.conf.json`": відсутнє шифрування, дублювання конфіг-файлу без переваг.

Додаткової інформації в transcript не зафіксовано.
