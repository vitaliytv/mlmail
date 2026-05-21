# C4 рівень 3 — Components для MLMaiL

Component diagram MLMaiL розкриває внутрішню структуру кожного контейнера застосунку MLMaiL: Frontend (Vue 3 + Vite) і Backend (Rust + Tauri 2). Кожен компонент MLMaiL описаний самодостатньо — без посилань на попередні або наступні розділи.

## Поточний стан

### Реалізовано

- Frontend Bootstrap MLMaiL (`app/src/main.js`): монтує Vue-застосунок, реєструє Quasar plugin.
- App Shell MLMaiL (`app/src/App.vue`): кореневий layout-компонент із `<q-layout>`.
- Auth Component MLMaiL (`app/src/views/Login.vue`): Login-екран із Quasar-компонентами.
- Frontend UI Kit MLMaiL: Quasar 2 з macOS material-look, `iconSet: 'material-symbols-outlined'`.
- Auth Store MLMaiL (`app/src/services/auth-store.js`): реактивний UI-стан без токенів у JS.
- Auth Errors i18n MLMaiL (`app/src/i18n/auth-errors.js`): відображення Rust kind → українська.
- Backend Entry main MLMaiL (`app/src-tauri/src/main.rs`): точка входу бінарника.
- Backend Entry lib MLMaiL (`app/src-tauri/src/lib.rs`): Tauri Builder, managed state, реєстрація команд.
- Auth Module MLMaiL (`app/src-tauri/src/auth/`): OAuth flow, `FileStorage` для токенів, 5 Tauri-команд.
- Endpoints MLMaiL (`app/src-tauri/src/endpoints.rs`): DI для URL-адрес зовнішніх API.
- Gmail Module MLMaiL (`app/src-tauri/src/gmail/`): `gmail_inbox_count`, `gmail_random_message`.
- Plugin Opener MLMaiL: `tauri-plugin-opener` для відкриття системного браузера.
- Capabilities default MLMaiL (`app/src-tauri/capabilities/default.json`): Tauri IPC-дозволи.

### Planned

- Gmail Client MLMaiL (Vue-side): обгортка Gmail REST API з `Authorization: Bearer`.
- Inbox List Component MLMaiL: список вхідних листів.
- Mail Reader Component MLMaiL: перегляд одного листа + оркестрація summary/TTS.
- Summary Service MLMaiL: LLM-клієнт для коротких саммері.
- Speech Service MLMaiL: TTS-клієнт для відтворення саммері.
- Action Bar Component MLMaiL: чотири дії над листом (delete, delete+filter, save home/work).
- Reply Drafter Component MLMaiL: AI-чернетка відповіді.
- Notes Bridge MLMaiL: IPC-обгортка Tauri-команд для `.md`-заміток.
- Notes Commands MLMaiL (Backend): Tauri-команди збереження та переліку заміток.

---

## Компоненти контейнера MLMaiL Frontend

### Компонент Frontend Bootstrap MLMaiL

**Відповідальність.** Frontend Bootstrap MLMaiL — точка входу контейнера MLMaiL Frontend, файл `app/src/main.js`. Монтує кореневий Vue-застосунок у DOM-елемент `#app` і реєструє Quasar plugin з `iconSet: 'material-symbols-outlined'` та `config.dark: 'auto'`.

**Залежності:**

- Входи: операційна система запускає Tauri WebView, який завантажує `app/index.html` і виконує `main.js`.
- Виходи: App Shell MLMaiL (`App.vue`), Frontend UI Kit MLMaiL (Quasar plugin).

**Тести:** TBD: tests.

**Релевантні ADR:** quasar-2-ui-фреймворк-mlmail.

---

### Компонент App Shell MLMaiL

**Відповідальність.** App Shell MLMaiL — кореневий Vue-компонент MLMaiL, файл `app/src/App.vue`. Обгортає все у `<q-layout view="hHh lpR fFf"><q-page-container>` і надає Layout-контекст для Quasar-компонентів. У цільовій реалізації MLMaiL стає тонкою обгорткою `<router-view>` і `<layout>`.

**Залежності:**

- Входи: Frontend Bootstrap MLMaiL монтує App Shell MLMaiL.
- Виходи: Auth Component MLMaiL, Inbox List Component MLMaiL, Mail Reader Component MLMaiL (через router).

**Тести:** TBD: tests.

**Релевантні ADR:** quasar-2-ui-фреймворк-mlmail.

---

### Компонент Auth Component MLMaiL

**Відповідальність.** Auth Component MLMaiL — Vue-компонент Login-екрану MLMaiL, файл `app/src/views/Login.vue`. Рендерить два стани UI:

- не авторизовано: `q-btn` «Увійти через Google» з вбудованим loading-станом;
- авторизовано: `q-chip` з кількістю листів INBOX (або `q-skeleton` під час завантаження), картка випадкового листа (From/Subject/Date/body через `<pre>`), `q-btn` «Показати інший», `q-btn` «Вийти»; `q-banner` при помилці Gmail.

Auth Component MLMaiL не торкається токенів і не знає про OAuth — викликає лише методи Auth Store MLMaiL (`login`, `logout`, `initialize`, `loadRandomMessage`). `q-page` використовує клас `column items-center` (не `flex-center`), щоб уникнути прихованого контенту нижче viewport.

**Залежності:**

- Входи: App Shell MLMaiL через `<router-view>`.
- Виходи: Auth Store MLMaiL (методи `login`, `logout`, `initialize`, `refreshInboxCount`, `loadRandomMessage`), Auth Errors i18n MLMaiL (`errorMessage(kind)`), Frontend UI Kit MLMaiL (Quasar).

**Тести:** `app/src/views/Login.vitest.js` (Vitest + `@vue/test-utils` + `mountWithQuasar`, 5 тестів; mock-об'єкти з `vi.fn()` оголошуються через `vi.hoisted(() => ({...}))` для коректного hoisting разом із `vi.mock`).

**Релевантні ADR:** ADR-0006-google-oauth, quasar-2-ui-фреймворк-mlmail, q-page-flex-center-overflow, gmail-inbox-count-стартовий-екран, gmail-random-message-команда, vi-hoisted-vitest-mock.

---

### Компонент Frontend UI Kit MLMaiL

**Відповідальність.** Frontend UI Kit MLMaiL — Quasar 2 як UI-фреймворк MLMaiL. Підключений через `@quasar/vite-plugin` у `app/vite.config.js` із кастомними sass-vars у `app/src/quasar-variables.sass`: `$primary: #0a84ff` (macOS Accent Blue), system-ui font stack із SF Pro, button radius 6 px, generic radius 8 px. Dark mode `config.dark: 'auto'` наслідує OS `prefers-color-scheme`. Іконки — Material Symbols Outlined (`iconSet: 'material-symbols-outlined'`); у template-атрибутах обов'язковий prefix `sym_o_` (наприклад, `sym_o_mail`, `sym_o_refresh`, `sym_o_logout`).

**Залежності:**

- Входи: Frontend Bootstrap MLMaiL реєструє Quasar plugin у `main.js`.
- Виходи: усі Vue-компоненти MLMaiL використовують Quasar UI primitives (`q-page`, `q-btn`, `q-card`, `q-chip`, `q-banner`, `q-skeleton`, `q-layout`, `q-page-container`).

**Тести:** `app/src/test-utils/quasar.js` — helper `mountWithQuasar`, що обгортає компоненти у `q-layout > q-page-container` і реєструє Quasar з `dark: false` для детермінованого тестування.

**Релевантні ADR:** quasar-2-ui-фреймворк-mlmail.

---

### Компонент Auth Store MLMaiL

**Відповідальність.** Auth Store MLMaiL — реактивний singleton-composable MLMaiL, файл `app/src/services/auth-store.js`. Тримає виключно UI-стан у Vue `ref`-ах: `email`, `isAuthenticated`, `isLoading`, `errorKind`, `inboxCount`, `inboxErrorKind`, `currentMessage`, `messageErrorKind`, `isMessageLoading`. Жодних токенів у JS-пам'яті: доступ до access token виключно через Tauri-команду `auth_get_access_token`. При отриманні `ReauthRequired` від контейнера MLMaiL Backend — автоматично переводить UI у стан «не залогінений».

Публічна поверхня Auth Store MLMaiL: `initialize`, `login`, `logout`, `getAccessToken`, `refreshInboxCount`, `loadRandomMessage` та readonly-`ref`-и стану.

**Залежності:**

- Входи: Auth Component MLMaiL; (у майбутньому) Inbox List Component MLMaiL, Mail Reader Component MLMaiL.
- Виходи: контейнер MLMaiL Backend через `@tauri-apps/api/core::invoke` (команди `auth_start_login`, `auth_get_access_token`, `auth_is_authenticated`, `auth_current_email`, `auth_logout`, `gmail_inbox_count`, `gmail_random_message`).

**Тести:** `app/src/services/auth-store.test.js` (bun:test, 9 тестів).

**Релевантні ADR:** ADR-0006-google-oauth, gmail-rust-команди-та-auth-store, gmail-inbox-count-стартовий-екран, gmail-random-message-команда, bun-vitest-dual-runner, міграція-з-vitest-на-bun-test-runner.

---

### Компонент Auth Errors i18n MLMaiL

**Відповідальність.** Auth Errors i18n MLMaiL — словник MLMaiL, файл `app/src/i18n/auth-errors.js`. Функція `errorMessage(kind)` мапить Rust `AuthError`/`GmailError` kind у українську строку для UI. Десять ключів: `Cancelled`, `Network`, `OAuth`, `Storage`, `ReauthRequired`, `Platform`, `Http`, `Parse`, `Empty`, `Unknown`. Для невідомого kind повертає «Невідома помилка.». Ключі `Http`, `Parse`, `Empty` покривають помилки Gmail-шару MLMaiL.

**Залежності:**

- Входи: Auth Component MLMaiL викликає `errorMessage(kind)` для відображення у `q-banner`.
- Виходи: немає зовнішніх залежностей; лише статичний словник.

**Тести:** `app/src/i18n/auth-errors.test.js` (bun:test, 8 тестів).

**Релевантні ADR:** ADR-0006-google-oauth, gmail-inbox-count-стартовий-екран, gmail-random-message-команда, міграція-з-vitest-на-bun-test-runner.

---

### Компонент Gmail Client MLMaiL (planned)

**Відповідальність.** Gmail Client MLMaiL — Vue-side клієнт Gmail REST API MLMaiL. Інкапсулює HTTPS-виклики до Gmail, додає заголовок `Authorization: Bearer <token>` через Auth Store MLMaiL та виконує retry при `401`.

Цільова поверхня Gmail Client MLMaiL:

- `listInbox(maxResults)` — список листів INBOX;
- `getMessage(id)` — повне тіло листа;
- `trashMessage(id)` — видалення у кошик;
- `createFilter(criteria, action)` — Gmail-фільтр для дії delete + filter;
- `createDraft(message)` — чернетка відповіді.

**Залежності:**

- Входи: Inbox List Component MLMaiL, Mail Reader Component MLMaiL, Action Bar Component MLMaiL, Reply Drafter Component MLMaiL.
- Виходи: зовнішня система Gmail API (`https://gmail.googleapis.com/gmail/v1`).

**Тести:** TBD: tests.

**Релевантні ADR:** ADR-0006-google-oauth, gmail-rust-команди-та-auth-store.

---

### Компонент Inbox List Component MLMaiL (planned)

**Відповідальність.** Inbox List Component MLMaiL — Vue-компонент списку вхідних листів MLMaiL. Отримує список листів через Gmail Client MLMaiL, рендерить рядок на кожен лист, обробляє вибір листа і переходить на маршрут Mail Reader Component MLMaiL.

**Залежності:**

- Входи: App Shell MLMaiL через `<router-view>`.
- Виходи: Gmail Client MLMaiL (`listInbox`), Mail Reader Component MLMaiL (через router).

**Тести:** TBD: tests.

**Релевантні ADR:** gmail-rust-команди-та-auth-store, c4-документація-mlmail-ініціалізація.

---

### Компонент Mail Reader Component MLMaiL (planned)

**Відповідальність.** Mail Reader Component MLMaiL — Vue-компонент перегляду одного листа Gmail у MLMaiL. Оркеструє: (1) отримання тіла листа через Gmail Client MLMaiL; (2) запит саммері у Summary Service MLMaiL; (3) передачу саммері у Speech Service MLMaiL для відтворення аудіо; (4) відображення Action Bar Component MLMaiL; (5) при виборі `reply` — відображення Reply Drafter Component MLMaiL.

**Залежності:**

- Входи: App Shell MLMaiL через `<router-view>`; Inbox List Component MLMaiL передає `messageId`.
- Виходи: Gmail Client MLMaiL, Summary Service MLMaiL, Speech Service MLMaiL, Action Bar Component MLMaiL, Reply Drafter Component MLMaiL.

**Тести:** TBD: tests.

**Трасування:** TBD: tracing-storage (LLM-виклик через Summary Service MLMaiL).

**Релевантні ADR:** gmail-rust-команди-та-auth-store, c4-документація-mlmail-ініціалізація.

---

### Компонент Summary Service MLMaiL (planned)

**Відповідальність.** Summary Service MLMaiL — клієнт LLM-провайдера у контейнері MLMaiL Frontend. Приймає тіло листа і метадані, повертає коротке саммері. Поверхня: `summarize(messageBody): Promise<string>`. Конкретний LLM-провайдер MLMaiL визначається окремим ADR.

**Залежності:**

- Входи: Mail Reader Component MLMaiL.
- Виходи: зовнішня система LLM-провайдер (TBD).

**Тести:** TBD: tests.

**Трасування:** TBD: tracing-storage.

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

### Компонент Speech Service MLMaiL (planned)

**Відповідальність.** Speech Service MLMaiL — клієнт TTS-провайдера у контейнері MLMaiL Frontend. Приймає текст саммері і відтворює аудіо у Tauri WebView. Базовий кандидат — браузерний `SpeechSynthesis` API (без мережі і без ключів). Доступність на Android System WebView — завдання майбутньої реалізації MLMaiL.

**Залежності:**

- Входи: Mail Reader Component MLMaiL.
- Виходи: Web API `SpeechSynthesis` або зовнішня TTS-служба (TBD).

**Тести:** TBD: tests.

**Трасування:** TBD: tracing-storage.

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

### Компонент Action Bar Component MLMaiL (planned)

**Відповідальність.** Action Bar Component MLMaiL — Vue-компонент з чотирма діями над поточним листом MLMaiL:

- `delete` — `trashMessage(id)` через Gmail Client MLMaiL;
- `delete + filter` — `trashMessage(id)` плюс `createFilter(…)` для відправника/теми;
- `save → home` — запис `.md` у `notes/home/` через Notes Bridge MLMaiL;
- `save → work` — запис `.md` у `notes/work/` через Notes Bridge MLMaiL.

Після будь-якої дії — перехід до Reply Drafter Component MLMaiL.

**Залежності:**

- Входи: Mail Reader Component MLMaiL.
- Виходи: Gmail Client MLMaiL (`trashMessage`, `createFilter`), Notes Bridge MLMaiL (`saveNote`), Reply Drafter Component MLMaiL (через router).

**Тести:** TBD: tests.

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

### Компонент Reply Drafter Component MLMaiL (planned)

**Відповідальність.** Reply Drafter Component MLMaiL — Vue-компонент MLMaiL, що показує AI-чернетку відповіді на лист і дає користувачу відредагувати її перед збереженням у Gmail. Пряме відправлення без перегляду навмисно не передбачено — користувач MLMaiL завжди контролює фінальний крок. Збереження чернетки — через Gmail Client MLMaiL `createDraft(message)`.

**Залежності:**

- Входи: Action Bar Component MLMaiL (через router після будь-якої дії).
- Виходи: Gmail Client MLMaiL (`createDraft`).

**Тести:** TBD: tests.

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

### Компонент Notes Bridge MLMaiL (planned)

**Відповідальність.** Notes Bridge MLMaiL — тонка IPC-обгортка MLMaiL над Tauri-командами контейнера MLMaiL Backend для роботи з `.md`-замітками у контейнері Локальне сховище MLMaiL.

Поверхня Notes Bridge MLMaiL:

- `saveNote(kind: 'work' | 'home', message): Promise<void>`;
- `listNotes(kind: 'work' | 'home'): Promise<Note[]>`.

Реалізується через `invoke('save_note', …)` і `invoke('list_notes', …)`.

**Залежності:**

- Входи: Action Bar Component MLMaiL.
- Виходи: контейнер MLMaiL Backend (Notes Commands MLMaiL) через `@tauri-apps/api/core::invoke`.

**Тести:** TBD: tests (тест на круг `saveNote → listNotes` проти тимчасової теки `app_data_dir()`).

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

## Компоненти контейнера MLMaiL Backend

### Компонент Backend Entry main MLMaiL

**Відповідальність.** Backend Entry main MLMaiL — файл `app/src-tauri/src/main.rs`. На не-debug збірках MLMaiL вимикає консольне вікно Windows атрибутом `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` і викликає `mlmail_lib::run()`. Уся логіка контейнера MLMaiL Backend живе у crate-бібліотеці `mlmail_lib`.

**Залежності:**

- Входи: операційна система запускає бінарник MLMaiL Backend.
- Виходи: Backend Entry lib MLMaiL (функція `mlmail_lib::run()`).

**Тести:** TBD: tests.

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

### Компонент Backend Entry lib MLMaiL

**Відповідальність.** Backend Entry lib MLMaiL — функція `run()` у файлі `app/src-tauri/src/lib.rs`. Будує `tauri::Builder` для MLMaiL, реєструє Tauri managed state (`SharedStorage`, `Endpoints`), плагіни і всі Tauri-команди. Атрибут `#[cfg_attr(mobile, tauri::mobile_entry_point)]` робить функцію `run()` точкою входу для Android-збірки MLMaiL. Першою дією викликає `dotenvy::dotenv().ok()` для завантаження OAuth Client IDs з `app/src-tauri/.env` і `app/src-tauri/.env.secret`.

**Залежності:**

- Входи: Backend Entry main MLMaiL.
- Виходи: Auth Module MLMaiL, Gmail Module MLMaiL, Plugin Opener MLMaiL, Endpoints MLMaiL, Notes Commands MLMaiL (planned).

**Тести:** TBD: tests.

**Релевантні ADR:** ADR-0006-google-oauth, tauri-storage-endpoints-managed-state, oauth-client-ids-runtime-dotenvy.

---

### Компонент Auth Module MLMaiL

**Відповідальність.** Auth Module MLMaiL — Rust-підсистема MLMaiL у `app/src-tauri/src/auth/`, що реалізує всю Google OAuth-механіку. Експортує п'ять Tauri-команд: `auth_start_login`, `auth_get_access_token`, `auth_is_authenticated`, `auth_current_email`, `auth_logout`. Внутрішній helper `acquire_access_token` повторно використовується Gmail Module MLMaiL та integration тестами.

Підкомпоненти Auth Module MLMaiL:

| Файл | Роль |
| --- | --- || `mod.rs` | П'ять Tauri-команд; helper `acquire_access_token`; `on_startup` для відновлення сесії при холодному старті |
| `state.rs` | `AuthState` (in-memory access token + email + expiry); `is_access_token_fresh()` з 30 с буфером |
| `pkce.rs` | PKCE pair: verifier 43 chars URL-safe, challenge = base64url(SHA-256(verifier)) |
| `token_exchange.rs` | HTTPS-обмін до `oauth2.googleapis.com/token`; 400 + `invalid_grant` → `AuthError::ReauthRequired`; `Option<&str>` для `client_secret` (Desktop — `Some(secret)`, Android — `None`) |
| `id_token.rs` | Витяг `email` з JWT payload без верифікації (токен отриманий напряму від Google по HTTPS) |
| `error.rs` | `AuthError` enum з serde-серіалізацією `{kind, message}` |
| `config.rs` | OAuth Client IDs через `std::env::var()` + `OnceLock<String>` (runtime, `dotenvy`); `.env` — публічні Client IDs, `.env.secret` — Desktop `client_secret` |
| `storage/mod.rs` | Trait `RefreshTokenStorage`; `SharedStorage = Arc<dyn RefreshTokenStorage>`; `platform_storage()` factory (macOS → `FileStorage`, Android → `EncryptedSharedPreferences`) |
| `storage/file.rs` | `FileStorage`: JSON `{app_data_dir}/session.json` з правами `0600`; атомарний запис — temp-файл з `mode(0o600)` + POSIX `rename` (усуває TOCTOU-вразливість) |
| `storage/android.rs` | Kotlin-bridge для Android `EncryptedSharedPreferences` через Tauri mobile plugin |
| `flow/macos.rs` | Loopback HTTP server на `127.0.0.1:RANDOM_PORT` + `tauri-plugin-opener` для системного браузера + 5-хв таймаут + CSRF перевірка |
| `flow/android.rs` | Tauri 2 mobile plugin → Kotlin `signInAndAuthorize` (Credential Manager + AuthorizationClient) + token exchange |

**Залежності:**

- Входи: Auth Store MLMaiL (контейнер MLMaiL Frontend) через Tauri IPC; Gmail Module MLMaiL (через `acquire_access_token`); Backend Entry lib MLMaiL реєструє команди і managed state.
- Виходи: зовнішня система Google Identity Services (`oauth2.googleapis.com/token`); `FileStorage` записує `{app_data_dir}/session.json`; `flow/macos.rs` відкриває системний браузер через Plugin Opener MLMaiL.

**Тести:**

- `app/src-tauri/tests/auth_commands.rs` (Tauri mock runtime)
- `app/src-tauri/tests/auth_logout.rs` (Tauri mock runtime)
- `app/src-tauri/tests/auth_finalize_login.rs` (Tauri mock runtime)
- `app/src-tauri/tests/acquire_access_token.rs` (Tauri mock runtime)
- `app/src-tauri/src/auth/storage/file.rs` (7 unit-тестів FileStorage)
- 32 unit-тести у `app/src-tauri/src/auth/` (PKCE, ID token, state, token_exchange з mockito, InMemoryStorage, parser callback)

**Релевантні ADR:** ADR-0006-google-oauth, oauth-client-ids-runtime-dotenvy, tauri-mock-runtime-тестування, tauri-storage-endpoints-managed-state, файловий-стор-токенів-замість-keychain.

---

### Компонент Endpoints MLMaiL

**Відповідальність.** Endpoints MLMaiL — файл `app/src-tauri/src/endpoints.rs`. Структура `Endpoints { google_token, gmail_label_inbox, gmail_messages_list }` з `impl Default` (реальні Google URLs). Реєструється у Tauri managed state через `.manage(Endpoints::default())`. Дозволяє підмінити URL у тестах на mockito-адресу без зміни prod-коду Auth Module MLMaiL і Gmail Module MLMaiL.

**Залежності:**

- Входи: Backend Entry lib MLMaiL реєструє `Endpoints::default()` у managed state.
- Виходи: Auth Module MLMaiL і Gmail Module MLMaiL отримують `State<Endpoints>` у Tauri-командах.

**Тести:** `app/src-tauri/tests/auth_commands.rs`, `app/src-tauri/tests/gmail_commands.rs` (підміна URL на mockito у тестах).

**Релевантні ADR:** tauri-storage-endpoints-managed-state, tauri-mock-runtime-тестування.

---

### Компонент Gmail Module MLMaiL

**Відповідальність.** Gmail Module MLMaiL — Rust-підсистема MLMaiL у `app/src-tauri/src/gmail/`, що інкапсулює HTTPS-виклики до Gmail REST API. Експортує дві Tauri-команди:

- `gmail_inbox_count() -> Result<u64, GmailError>` — повертає `messagesTotal` мітки `INBOX` через `GET users/me/labels/INBOX`. На 401 від Gmail → `GmailError::ReauthRequired` (Auth Store MLMaiL переводить UI у стан «не залогінений»).
- `gmail_random_message() -> Result<GmailMessage, GmailError>` — вибирає випадковий id серед перших 100 листів INBOX, отримує повне тіло (`format=full`), парсить headers (From/Subject/Date) і витягує `text/plain` (пріоритет) або очищений `text/html`; тіло обрізається до 10 000 символів. На порожній INBOX → `GmailError::Empty`.

Підкомпоненти Gmail Module MLMaiL:

| Файл | Роль |
| --- | --- || `mod.rs` | Tauri-команди `gmail_inbox_count` / `gmail_random_message`; HTTP helpers з URL як параметром (для mockito у тестах) |
| `message.rs` | `GmailMessage` DTO (`id, from, subject, date, body`); `extract_header` (case-insensitive); `extract_plain_text` (рекурсивний обхід `payload.parts`, пріоритет `text/plain`, fallback `text/html` зі стрипом тегів) |
| `error.rs` | `GmailError { Network, Http{status,body}, Parse, ReauthRequired, Platform, Empty }` + конверсії з `reqwest::Error` і `AuthError` |

**Залежності:**

- Входи: Auth Store MLMaiL (контейнер MLMaiL Frontend) через `invoke('gmail_inbox_count')`, `invoke('gmail_random_message')`; Backend Entry lib MLMaiL реєструє команди.
- Виходи: Auth Module MLMaiL (через `acquire_access_token`); зовнішня система Gmail API (`https://gmail.googleapis.com/gmail/v1`).

**Тести:**

- `app/src-tauri/tests/gmail_commands.rs` (Tauri mock runtime + mockito)
- 30 unit-тестів у `app/src-tauri/src/gmail/` (`parse_messages_total`, `fetch_inbox_count_at`, `list_inbox_ids_at`, `get_message_at`, `extract_header`, `extract_plain_text` для plain/html/multipart/nested/empty, `GmailError` serde)

**Релевантні ADR:** gmail-inbox-count-стартовий-екран, gmail-random-message-команда, gmail-rust-команди-та-auth-store, tauri-mock-runtime-тестування, tauri-storage-endpoints-managed-state.

---

### Компонент Notes Commands MLMaiL (planned)

**Відповідальність.** Notes Commands MLMaiL — набір Tauri-команд MLMaiL для роботи з `.md`-замітками у контейнері Локальне сховище MLMaiL:

- `save_note(kind: NoteKind, message: GmailMessage) -> Result<NotePath, AppError>`;
- `list_notes(kind: NoteKind) -> Result<Vec<NoteSummary>, AppError>`;
- (можливо) `delete_note(path: NotePath) -> Result<(), AppError>`.

Команди Notes Commands MLMaiL використовують `app_data_dir()` як корінь теки `notes/`, нормалізують ім'я файлу за шаблоном `YYYYMMDD-HHMMSS-<gmail-message-id>.md` і записують Markdown-замітку MLMaiL з frontmatter. Точну схему фіксуватиме окремий ADR.

**Залежності:**

- Входи: Notes Bridge MLMaiL (контейнер MLMaiL Frontend) через `invoke('save_note', …)` і `invoke('list_notes', …)`; Backend Entry lib MLMaiL реєструє команди.
- Виходи: контейнер Локальне сховище MLMaiL (файлова система `app_data_dir()/notes/`).

**Тести:** TBD: tests (тест на круг `save_note → list_notes` проти тимчасової теки).

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

### Компонент Plugin Opener MLMaiL

**Відповідальність.** Plugin Opener MLMaiL — `tauri-plugin-opener`, ініціалізований у Backend Entry lib MLMaiL. Надає API для відкриття URL і файлів у системному застосунку. Auth Module MLMaiL використовує Plugin Opener MLMaiL для відкриття системного браузера під час OAuth authorization redirect у `flow/macos.rs`.

**Залежності:**

- Входи: Backend Entry lib MLMaiL реєструє `.plugin(tauri_plugin_opener::init())`.
- Виходи: зовнішня система — системний браузер macOS (OAuth authorization URL).

**Тести:** TBD: tests.

**Релевантні ADR:** ADR-0006-google-oauth.

---

### Компонент Capabilities default MLMaiL

**Відповідальність.** Capabilities default MLMaiL — файл `app/src-tauri/capabilities/default.json`. Описує дозволи Tauri для головного вікна MLMaiL. Поточний вміст:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default"]
}
```

Будь-який новий Tauri-API MLMaiL (наприклад, `fs:allow-write-text-file` для Notes Commands MLMaiL) має бути доданий до `permissions` у цьому файлі — без цього IPC-виклики MLMaiL відхиляються Tauri runtime.

**Залежності:**

- Входи: Tauri Build System читає файл під час компіляції MLMaiL Backend.
- Виходи: Tauri runtime — дозволяє або відхиляє IPC-виклики для вікна `main`.

**Тести:** TBD: tests.

**Релевантні ADR:** c4-документація-mlmail-ініціалізація.

---

## Тести рівня Components MLMaiL

### Реалізовано

| Файл | Runner | Тестів | Покриває |
| --- | --- | --- | --- || `app/src/i18n/auth-errors.test.js` | bun:test | 8 | Auth Errors i18n MLMaiL |
| `app/src/services/auth-store.test.js` | bun:test | 9 | Auth Store MLMaiL |
| `app/src/views/Login.vitest.js` | Vitest | 5 | Auth Component MLMaiL |
| `app/src-tauri/src/auth/storage/file.rs` | cargo test | 7 | FileStorage (Auth Module MLMaiL) |
| `app/src-tauri/src/auth/` (unit) | cargo test | 32 | Auth Module MLMaiL (PKCE, token_exchange, state, id_token) |
| `app/src-tauri/src/gmail/` (unit) | cargo test | 30 | Gmail Module MLMaiL (parse, extract, error) |
| `app/src-tauri/tests/auth_commands.rs` | cargo test | — | Auth Module MLMaiL (mock runtime) |
| `app/src-tauri/tests/auth_logout.rs` | cargo test | — | Auth Module MLMaiL (`auth_logout`) |
| `app/src-tauri/tests/auth_finalize_login.rs` | cargo test | — | Auth Module MLMaiL (finalize_login) |
| `app/src-tauri/tests/acquire_access_token.rs` | cargo test | — | Auth Module MLMaiL (`acquire_access_token`) |
| `app/src-tauri/tests/gmail_commands.rs` | cargo test | — | Gmail Module MLMaiL (mock runtime) |

### Planned

- Vitest + `@vue/test-utils` для Inbox List Component MLMaiL, Mail Reader Component MLMaiL, Action Bar Component MLMaiL, Reply Drafter Component MLMaiL.
- `cargo test` для Notes Commands MLMaiL + integration тест `save_note → list_notes` проти тимчасової теки.
- Summary Service MLMaiL та Speech Service MLMaiL — після вибору провайдерів (окремі ADR).
