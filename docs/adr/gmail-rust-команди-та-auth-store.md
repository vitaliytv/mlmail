# Gmail API-виклики через Rust Tauri-команди та auth-store

**Status:** Accepted
**Date:** 2026-05-15

## Context and Problem Statement

MLMaiL потребує звернень до Gmail REST API (inbox count, випадковий лист). Виникло два архітектурних питання: де виконувати HTTP-запити — у Rust-бекенді Tauri чи напряму у Vue через `fetch`; та де зберігати Gmail-дані у Vue-застосунку — в `auth-store.js` чи в окремому `mailbox-store.js`.

## Considered Options

- Усі Gmail API-виклики у Rust як Tauri-команди; Vue викликає лише `invoke('gmail_inbox_count')` та `invoke('gmail_random_message')` — жодного `fetch` до googleapis у JS-коді
- Прямий `fetch` з Vue до Gmail API
- Gmail-дані (`inboxCount`, `currentMessage` тощо) в `auth-store.js`
- Окремий `mailbox-store.js`

## Decision Outcome

Chosen option: "Gmail API у Rust Tauri-команди + Gmail-дані в auth-store", because access token не виходить за межі Rust і не потрапляє у JS-контекст WebView; логіка рефрешу `acquire_access_token` вже є в `auth/mod.rs` і перевикористовується напряму; Gmail-дані семантично прив'язані до auth-сесії — фетчаться після `login()`/`initialize()` і чистяться після `logout()`; окремий `mailbox-store.js` — передчасна абстракція (YAGNI) без жодних додаткових споживачів.

### Consequences

- Good, because access token залишається в Rust — ризик витоку у WebView усунуто.
- Good, because CORS-проблеми у Tauri WebView відсутні.
- Good, because `acquire_access_token` перевикористовується між командами без дублювання коду.
- Good, because тестування через mockito (Rust) простіше, ніж mocking fetch у JS.
- Good, because одна залежність (auth-store) простіша, ніж дві скоординовані.
- Bad, because `auth-store.js` зростає за обсягом, поєднуючи auth- і mail-стан.
- Neutral, because `mailbox-store.js` був створений хуком-автоматизацією і видалений після прийняття рішення — transcript підтверджує факт.

## More Information

- `app/src-tauri/src/gmail/mod.rs` — реалізація обох Tauri-команд
- `app/src-tauri/src/gmail/error.rs` — типи помилок
- `app/src-tauri/src/auth/mod.rs` — helper `acquire_access_token`
- `app/src/services/auth-store.js` — поля `inboxCount`, `inboxErrorKind`, `currentMessage`, `messageErrorKind`, `isMessageLoading`; методи `refreshInboxCount()`, `loadRandomMessage()`
- `app/src/services/auth-store.test.js`
- `app/src/views/Login.vue`
- Деталі endpoint inbox count: ADR `gmail-inbox-count-стартовий-екран.md`
- Деталі вибірки random message: ADR `gmail-random-message-команда.md`
