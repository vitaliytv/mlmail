# Випадковий лист на стартовому екрані — design spec

**Дата:** 2026-05-15
**Статус:** Approved, готовий до плану імплементації
**Scope:** Після успішного логіну MLMaiL показує тіло одного випадкового листа з INBOX (plain text) поряд із кількістю листів. Кнопка «Показати інший» обирає новий випадковий. Без списку, без дій над листом, без HTML-рендеру.

## Мета

1. Після `initialize()` / `login()` MLMaiL автоматично тягне один випадковий лист з мітки INBOX і рендерить його як картку: `From`, `Subject`, `Date`, `Body` (plain text).
2. Кнопка «Показати інший» поряд із карткою викликає той самий шлях ще раз — на екрані новий випадковий лист.
3. Помилки шару Gmail (мережа, 401, 5xx, парсинг MIME) трактуються однаково з inbox count: 401 → переходимо на login, інше — український рядок з `auth-errors.js`.
4. Порожня скринька → не показуємо картку, лише український рядок «Скринька порожня.» (новий kind `Empty`).
5. UI українською; Rust повертає англомовні `kind`-и.

Що **поза scope** цієї ітерації:

- HTML-рендер тіла листа (картинки, форматування, посилання). Тільки plain text.
- Перехід до повноцінного перегляду листа (Mail Reader Component із C4-моделі).
- Sand­box iframe, DOMPurify, sanitization.
- Авто-таймер «новий лист кожні N сек».
- Кеш листа на диску.
- Фільтр по інших labels / search-фільтри.
- Дії: видалити, відповісти, додати в notes.
- Підтримка inline-attachments, multipart-зображень.
- Чесний рандом по всьому INBOX (поза першими 100 повідомленнями).

## Ключові архітектурні рішення

1. **Sample space — перші 100 листів `messages.list?labelIds=INBOX&maxResults=100&fields=messages/id`.** Один HTTP-виклик, мінімальний JSON (тільки id), стабільна швидкість. Альтернативи (pageToken-пагінація по всьому скринці; `q=before:/after:`) відкинуто — overkill для UX «покажи якийсь лист».
2. **Дві Gmail-команди один за одним:** `messages.list` → випадковий id → `messages.get?format=full`. Випадковість — `rand::random::<usize>() % ids.len()`.
3. **Body extraction — `text/plain` first, fallback `text/html` з регекс-стрипом.** Robust підхід: рекурсивно обходимо `payload.parts`. Якщо знайдено `text/plain` — base64url decode, повертаємо як є. Якщо є тільки `text/html` — base64url decode, strip тегів простим regex `<[^>]+>` + `html_escape::decode_html_entities`. Якщо payload без parts (`text/plain` чи `text/html` напряму у `payload.body`) — той самий шлях.
4. **HTTPS-виклик у Rust**, не у WebView. Узгоджено з ADR-0006 + рішенням про inbox count.
5. **DTO:** `pub struct GmailMessage { id, from, subject, date, body }` — усі `String`. Body truncate до 10 000 символів у Rust перед поверненням (UX-страховка).
6. **UI MLMaiL — українською.** Нові тексти у `auth-errors.js`: `Empty` → «Скринька порожня.» (інші kinds уже є).
7. **Стан у фронті** живе в тому самому `auth-store.js` (як і `inboxCount`) — щоб не плодити окремий store-композабл під одну поверхню. Перейменування на `mail-store` буде окремою ітерацією, коли список листів зʼявиться.

## Архітектура

### Rust шар

Розширюємо існуючий модуль `app/src-tauri/src/gmail/`:

```
gmail/
  mod.rs      // + Tauri-команда gmail_random_message + helpers
  error.rs    // + GmailError::Empty варіант
  message.rs  // NEW: GmailMessage DTO + extract_plain_text + extract_headers
```

- `#[derive(Serialize)] pub struct GmailMessage { id: String, from: String, subject: String, date: String, body: String }`.
- `pub fn extract_plain_text(payload: &serde_json::Value) -> String`:
  - якщо `payload.mimeType` починається з `text/plain` — base64url-decode `payload.body.data`;
  - якщо `text/html` без plain-альтернативи — decode + strip тегів регексом + decode entities;
  - якщо є `parts` — рекурсивно обходимо, перевага `text/plain` над `text/html`.
- `pub fn extract_header(headers: &[Value], name: &str) -> String` — case-insensitive пошук.
- `#[tauri::command] pub async fn gmail_random_message(app, state) -> Result<GmailMessage, GmailError>`:
  1. `acquire_access_token`.
  2. `list_inbox_ids_at(LIST_URL, &token, 100) -> Vec<String>`. Якщо `[]` → `GmailError::Empty`.
  3. `let i = rand::random::<usize>() % ids.len(); let id = &ids[i];`
  4. `get_message_at(MESSAGE_URL_BASE, &token, id) -> serde_json::Value`.
  5. Парс: from = `extract_header(headers, "From")`, subject = `extract_header(headers, "Subject")`, date = `extract_header(headers, "Date")`, body = `extract_plain_text(&payload).chars().take(10_000).collect()`.
  6. Повернути `GmailMessage`.
- Status mapping як в `fetch_inbox_count_at`: 401 → `ReauthRequired`, інше → `Http{status,body}`.
- `Empty` додаємо до `GmailError`.

### Vue шар

`auth-store.js` отримує:

- `_currentMessage = ref(null)`, `_messageErrorKind = ref(null)`, `_isMessageLoading = ref(false)`;
- метод `async loadRandomMessage()`:
  - якщо `!_isAuthenticated.value` — нічого;
  - `_isMessageLoading.value = true; _messageErrorKind.value = null;`
  - try `_currentMessage.value = await invoke('gmail_random_message')`;
  - catch — `_currentMessage.value = null; _messageErrorKind.value = kind ?? 'Unknown'`; якщо `kind === 'ReauthRequired'` — скидаємо логін як у `refreshInboxCount`;
  - finally `_isMessageLoading.value = false`;
- `initialize()` після `refreshInboxCount` дзвонить `loadRandomMessage()`;
- `login()` так само після `refreshInboxCount`;
- `logout()` ресетить нові поля;
- експорт публічного API доповнюється `currentMessage`, `messageErrorKind`, `isMessageLoading`, `loadRandomMessage`.

`Login.vue` під рядком inbox count:

```vue
<section v-if="auth.currentMessage.value" class="message">
  <header>
    <p><strong>Від:</strong> {{ auth.currentMessage.value.from }}</p>
    <p><strong>Тема:</strong> {{ auth.currentMessage.value.subject }}</p>
    <p><strong>Дата:</strong> {{ auth.currentMessage.value.date }}</p>
  </header>
  <pre class="body">{{ auth.currentMessage.value.body }}</pre>
</section>
<p v-else-if="auth.isMessageLoading.value" class="muted">Завантаження…</p>
<p v-else-if="auth.messageErrorKind.value" class="error">
  {{ errorMessage(auth.messageErrorKind.value) }}
</p>
<button
  type="button"
  :disabled="auth.isMessageLoading.value"
  @click="auth.loadRandomMessage()"
>
  Показати інший
</button>
```

### i18n

Додаємо ключ `Empty: 'Скринька порожня.'` у `app/src/i18n/auth-errors.js`.

## Потік даних (happy path)

1. `App.vue` → `Login.vue.onMounted` → `auth.initialize()`.
2. `initialize` → `auth_is_authenticated` true → `auth_current_email` → `refreshInboxCount` → `loadRandomMessage`.
3. Rust: `acquire_access_token` (можливо з refresh) → `messages.list?labelIds=INBOX&maxResults=100&fields=messages/id` → 47 id-шників → `rand::random % 47 = 12` → `messages.get?id=<id>&format=full` → парс headers + body → `GmailMessage`.
4. Vue: `_currentMessage.value` = `{from, subject, date, body}` → `Login.vue` рендерить картку + кнопку «Показати інший».
5. Кнопка → `loadRandomMessage()` повторює кроки 3-4 із новим випадковим id.

## Помилки

| Сценарій | Rust kind | UI |
| --- | --- | --- |
| Немає мережі | `Network` | Червоний рядок з повідомленням `Network` |
| 401 від Gmail на list або get | `ReauthRequired` | Назад до «Увійти через Google» |
| INBOX порожній (`list` повернув `messages: null`/`[]`) | `Empty` | «Скринька порожня.» |
| 4xx/5xx | `Http{status,body}` | «Gmail повернув помилку. Спробуйте пізніше.» |
| Невалідний JSON / brakey base64 / нема `payload` | `Parse` | «Несподівана відповідь від Gmail.» |
| Знайшли лист, але body порожнє після strip | `Parse` | (фактично порожній лист) → показати картку з порожнім тілом — НЕ помилка |

## Тести

**Rust (unit):**

- `extract_plain_text`:
  - тільки `text/plain` payload → повертає decoded data;
  - тільки `text/html` payload → strip-ed plain text;
  - `multipart/alternative` з обома — повертає `text/plain` part;
  - вкладений `multipart/related` всередині `multipart/alternative` — рекурсія знаходить `text/plain`;
  - відсутні `parts` і відсутній `body.data` → повертає `""`.
- `extract_header` — case-insensitive (`From` vs `from`), без знайденого → `""`.
- `list_inbox_ids_at` + `get_message_at` з mockito:
  - щаслива траєкторія `200` + JSON → повна `GmailMessage`;
  - `list` повертає `{"resultSizeEstimate":0}` (нема `messages`) → `Empty`;
  - `list` 401 → `ReauthRequired`;
  - `get` 401 → `ReauthRequired`;
  - `get` 503 → `Http{503}`.

**Vue (vitest):**

- `auth-store.test.js` новий suite «random message»:
  - `initialize` → `currentMessage` зʼявляється;
  - `login` → `currentMessage` зʼявляється;
  - кнопковий потік: `loadRandomMessage()` ще раз → новий обʼєкт;
  - `Empty` → `currentMessage` null, `messageErrorKind === 'Empty'`;
  - `ReauthRequired` → `isAuthenticated=false`, обнуляє все;
  - `logout()` чистить `currentMessage`, `messageErrorKind`, `isMessageLoading`;
  - `isMessageLoading` flips true → false навколо успіху і помилки.
- `Login.test.js`:
  - після `initialize` з фейковим повідомленням `{from, subject, date, body}` — DOM містить «Від: …», «Тема: …», «Дата: …», тіло;
  - на `Empty` — DOM містить «Скринька порожня.»;
  - клік по кнопці «Показати інший» викликає `gmail_random_message` повторно.

## Документація, яку оновлюємо

1. `docs/ci4/03-components.md` — Gmail Module MLMaiL: додаємо нову команду; Auth Store: розширюємо ref-list і поверхню.
2. `docs/ci4/04-code.md` — секція для `app/src-tauri/src/gmail/message.rs` і нової команди.
3. `docs/ci4/decisions.md` — рішення «Випадковий лист: sample = 100 останніх з INBOX».
4. ADR у `docs/adr/_inbox/` — коротка нотатка про вибір sample-space.

## Залежності

Додаємо до `app/src-tauri/Cargo.toml`:

- `base64` уже є.
- `regex = "1"` — для strip тегів. Альтернатива (html2text) — нова велика залежність, не варто.
- `html-escape = "0.2"` — для decode HTML entities у html→text fallback.

## Поза scope (для майбутніх ітерацій)

- Перейменувати `auth-store.js` → `mail-store.js` коли зʼявиться список листів.
- HTML-рендер через DOMPurify + `<iframe sandbox>` без `allow-scripts`.
- Mail Reader Component MLMaiL із C4-карти.
- Кеш листа в `app_data_dir()` між запусками.
- Чесний рандом по всьому INBOX (потребує курсор-пагінації).
