# Гард `ConfigMissing` для незаповнених OAuth-credentials

**Status:** Accepted
**Date:** 2026-06-15

## Context and Problem Statement

`app/src-tauri/.env` містив порожнє значення `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID=`. Функція `is_real_client_id("")` повертала `true` (порожній рядок не починається з `REPLACE_ME`), тому login-флоу стартував із порожнім client_id, Google повертав помилку, а користувач бачив «Помилка авторизації Google.» без вказівки на справжню причину.

## Considered Options

- Додати явний variant `ConfigMissing` і гард у `run_login` перед стартом флоу
- Інші варіанти в transcript не обговорювалися.

## Decision Outcome

Chosen option: "Додати `ConfigMissing` гард", because користувач підтвердив явно: «Зробити цей гард»; без гарду порожній client_id спричиняє заплутану Google-помилку замість діагностичного повідомлення.

### Consequences

- Good, because замість «Помилка авторизації Google» користувач бачить «Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.»; `is_real_client_id` тепер відхиляє порожній і пробільний рядки.
- Bad, because transcript не містить підтверджених негативних наслідків.

## More Information

Змінені файли: `app/src-tauri/src/auth/error.rs` (новий variant `ConfigMissing(String)`), `app/src-tauri/src/auth/config.rs` (`is_real_client_id` — додано `!value.trim().is_empty()` + тест `empty_or_blank_value_is_not_considered_real`), `app/src-tauri/src/auth/mod.rs` (гард у `run_login` для `macos` та `android` — повертає `Err(AuthError::ConfigMissing(…))` якщо `!is_real_client_id`), `app/src/i18n/auth-errors.js` (новий ключ `ConfigMissing`), `app/src/i18n/auth-errors.test.js` (новий тест). Rust config-тести: 3/3 ok.

## Update 2026-06-15

Деталі реалізації (transcript dd346b91): `auth/error.rs` — варіант `ConfigMissing(String)`; `auth/config.rs` — `is_real_client_id` доповнено `!value.trim().is_empty()`, нова функція `require_configured`; `auth/mod.rs` — `run_login` (macOS і Android) викликає `require_configured` перед флоу; `app/src/i18n/auth-errors.js` — `ConfigMissing: 'Google OAuth не налаштовано: заповніть credentials у .env / .env.secret.'`; `auth-errors.test.js` — новий JS-тест. Rust config-тести: 3/3; JS: 86/86. Коміт: `0fdde42`.
