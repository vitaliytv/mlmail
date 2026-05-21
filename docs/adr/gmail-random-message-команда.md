# Команда gmail_random_message: вибірка з 100 листів і витяг тексту

**Status:** Accepted
**Date:** 2026-05-15

## Контекст

MLMaiL показує один випадковий лист з INBOX після логіну, щоб дати користувачу відчуття «живого» поштового клієнта. Необхідно обрати простір вибірки та спосіб витягти читабельний текст з Gmail API без HTML-рендерингу у WebView.

## Рішення/Процедура/Факт

Нова Tauri-команда `gmail_random_message` виконує два HTTP-виклики: `GET messages?labelIds=INBOX&maxResults=100&fields=messages/id` → масив id, потім `GET messages/<rand_id>?format=full`. Індекс — `rand::thread_rng().gen_range(0..n)`. Витяг тіла (`gmail/message.rs`, `extract_plain_text`): рекурсивний обхід `payload.parts`, перевага `text/plain` над `text/html`; HTML-fallback — стрип тегів через `regex` + декодування entities через `html-escape`; truncate до 10 000 символів. `GmailError::Empty` для порожньої скриньки; i18n-ключ `Empty` → «Скринька порожня.» На Vue-боці `auth-store.js` отримав `currentMessage`, `messageErrorKind`, `isMessageLoading`, `loadRandomMessage()`; `Login.vue` — картка From/Тема/Дата + `<pre class="message-body">` + кнопка «Показати інший».

## Обґрунтування

Два HTTP-виклики з передбачуваним часом відповіді незалежно від розміру скриньки. Повна пагінація потребувала б 50+ запитів для скриньок з 5K+ листів. Пріоритет `text/plain` виключає DOMPurify/iframe-ізоляцію. Стан у `auth-store.js` (YAGNI, без окремого composable).

## Розглянуті альтернативи

- Рандом по всьому INBOX через pageToken-пагінацію — відхилено через непередбачуваний час відповіді.
- Тільки snippet (~200 символів) — відхилено на користь повного `text/plain`.
- Повний HTML-рендер через DOMPurify або `<iframe sandbox>` — відкладено на наступну ітерацію.
- Окремий composable для message-стану — відхилено (YAGNI).

## Зачіпає

`app/src-tauri/src/gmail/message.rs` (новий), `app/src-tauri/src/gmail/mod.rs`, `app/src-tauri/src/gmail/error.rs`, `app/src-tauri/src/lib.rs`, `app/src/services/auth-store.js`, `app/src/views/Login.vue`, `app/src/i18n/auth-errors.js`, `app/src-tauri/Cargo.toml` (deps: `regex`, `html-escape`).

## Update 2026-05-15

Деталі реалізації body extraction та обробки помилок:

- Sample space: `messages.list?labelIds=INBOX&maxResults=100&fields=messages/id`
- Вибір індексу: `rand::random::<u64>() as usize % ids.len()`
- Отримання листа: `messages.get?id=<id>&format=full`
- Body extraction: пріоритет `text/plain`; fallback — `text/html` зі стрипом тегів через `regex` + `html-escape`
- Обмеження body: 10 000 символів
- Порожній INBOX: `GmailError::Empty` → рядок «Скринька порожня.» у UI
- Plain-text підхід усуває XSS-ризики без DOMPurify/iframe; HTML-рендер тіла винесено в окрему ітерацію
- Зачіпає: `app/src-tauri/src/gmail/error.rs` (новий kind `Empty`), `app/src/i18n/auth-errors.js`

---

**Опрацьовано** 2026-05-20. Проекції:

- [01-context](../ci4/01-context.md)
- [02-containers](../ci4/02-containers.md)
- [03-components](../ci4/03-components.md)
- [04-code](../ci4/04-code.md)
- [decisions](../ci4/decisions.md)
