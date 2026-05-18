# Відображення кількості листів INBOX на стартовому екрані

**Status:** Accepted
**Date:** 2026-05-14

## Context and Problem Statement

MLMaiL має показувати на стартовому екрані (Login.vue) точну кількість листів у скриньці INBOX одразу після успішного входу, використовуючи вже наявний scope `gmail.modify`.

## Considered Options

- `GET /gmail/v1/users/me/labels/INBOX` (`users.labels.get?id=INBOX`) — повертає точне поле `messagesTotal` для INBOX
- `users.getProfile` — повертає `messagesTotal` по всій скриньці, не тільки INBOX
- `messages.list?labelIds=INBOX&maxResults=1` з `resultSizeEstimate` — приблизне значення
- Прямий `fetch` з Vue — потребує передачі access token у JS-рівень

## Decision Outcome

Chosen option: "`users.labels.get?id=INBOX`", because це канонічний однозапитний шлях з точним числом листів INBOX; HTTP-виклик у Rust узгоджується з наявним патерном команд `auth_*`, не виносить access token у JS і виключає CORS-проблеми у WebView.

### Consequences

- Good, because `messagesTotal` з `labels.get` — точне значення, не `resultSizeEstimate`.
- Good, because Rust тримає access token — JS-рівень токена не бачить.
- Good, because CORS-проблеми у Tauri WebView виключені.
- Neutral, because під час завантаження Vue показує плейсхолдер «…»; помилка — i18n-рядок з `auth-errors.js`.
- Bad, because transcript не містить підтвердження щодо поведінки при офлайн-режимі або rate limiting.

## More Information

**Новий модуль:** `app/src-tauri/src/gmail/mod.rs` — команда `gmail_inbox_count(state) -> Result<u64, GmailError>`; 401 → `GmailError::ReauthRequired`; мережева помилка → `GmailError::Network`.

**Vue:** `auth-store.js` — `ref inboxCount` + `refreshInboxCount()`, викликаються автоматично після `initialize()`/`login()`.

**Документація:** оновлено `docs/ci4/03-components.md`, `docs/ci4/04-code.md`; спека: `docs/superpowers/specs/2026-05-14-inbox-count-design.md`.

**Файли:** `app/src-tauri/src/gmail/mod.rs` (новий), `app/src-tauri/src/lib.rs`, `app/src/services/auth-store.js`, `app/src/views/Login.vue`, `app/src/i18n/auth-errors.js`.

## Update 2026-05-14

- Rust-модуль: `app/src-tauri/src/gmail/`, HTTP через спільний helper `auth::acquire_access_token`
- Tauri-команда: `gmail_inbox_count` повертає поле `messagesTotal` (тип `u64`)
- Vue Auth Store: ref `inboxCount` — скидається до null при `ReauthRequired` від Gmail
- i18n покриває коди помилок `Http` та `Parse`
