# List-Unsubscribe (RFC 2369/8058) та sticky bottom bar у Gmail-клієнті

**Status:** Accepted
**Date:** 2026-05-24

## Context and Problem Statement

Застосунок не мав механізму відписки від розсилок: `GmailMessage` не містив поля `unsubscribe`, UI не пропонував кнопок дій під листом. Прокрутка до кнопок у довгому листі ускладнена, особливо на мобільних пристроях.

## Considered Options

- Regex-пошук посилань «unsubscribe» у тілі листа
- Парсинг стандартного заголовка `List-Unsubscribe` (RFC 2369/8058)
- Комбо: header-парсинг як основний шлях + body-fallback
- Sticky bottom bar для кнопок дій
- Top toolbar (sticky при скролі)
- Floating Action Button

## Decision Outcome

Chosen option: "Парсинг `List-Unsubscribe` заголовка + sticky bottom bar", because стандартний заголовок доступний у більшості листів-розсилок (особливо після вимог Google/Yahoo 2024), а sticky bottom bar відповідає thumb-zone мобільних пристроїв і є стандартом Apple Mail / Outlook Mobile.

### Consequences

- Good, because кнопки завжди доступні без прокрутки; one-click unsubscribe не вимагає відкриття браузера.
- Good, because бонус-команда `gmail_random_newsletter` (q=`has:list-unsubscribe`) дозволяє переглядати лише newsletters.
- Bad, because fallback для листів без `List-Unsubscribe` (body-scan) не реалізований у цій сесії — кнопка «Відписатися» буде disabled для таких листів.
- Neutral, because transcript не містить підтвердження інших наслідків.

## More Information

Реалізовано: enum `UnsubscribeAction { OneClick{url} | Url{url} | Mailto{to,subject} }` у `app/src-tauri/src/gmail/message.rs`; tauri-команди `gmail_unsubscribe` та `gmail_random_newsletter` у `app/src-tauri/src/gmail/mod.rs`; нові стани `isUnsubscribing`, `unsubscribeErrorKind`, `onlyNewsletters` у `app/src/services/auth-store.js`; sticky bottom bar через `q-page-sticky position="bottom"` + `safe-area-inset-bottom` у `app/src/views/Login.vue`. OneClick надсилає `POST` з body `List-Unsubscribe=One-Click` та `Content-Type: application/x-www-form-urlencoded`; URL/Mailto відкриваються через `tauri-plugin-opener`.
