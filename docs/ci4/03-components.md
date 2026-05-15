# C4 рівень 3 — Components для MLMaiL

Component diagram MLMaiL розкриває **внутрішню структуру кожного контейнера**
з [02-containers.md](02-containers.md). На цьому рівні описуємо логічні модулі
контейнера MLMaiL Frontend (Vue 3) і контейнера MLMaiL Backend (Rust + Tauri).

Розділи нижче самодостатні: кожен компонент MLMaiL прив'язаний до файлів коду
і до зовнішніх систем з [01-context.md](01-context.md).

## Компоненти контейнера MLMaiL Frontend

```mermaid
graph TB
    Bootstrap[Frontend Bootstrap MLMaiL<br/>app/src/main.js]
    Router[Frontend Router MLMaiL<br/>vite-plugin-vue-layouts-next<br/>planned]
    AppShell[App Shell MLMaiL<br/>app/src/App.vue]

    Auth[Auth Component MLMaiL<br/>Login екран, Google OAuth flow<br/>implemented]
    Inbox[Inbox List Component MLMaiL<br/>список листів Gmail<br/>planned]
    Reader[Mail Reader Component MLMaiL<br/>відкритий лист + саммері + плеєр<br/>planned]
    Actions[Action Bar Component MLMaiL<br/>delete / filter / save / reply<br/>planned]
    Drafter[Reply Drafter Component MLMaiL<br/>чернетка відповіді<br/>planned]

    SummaryService[Summary Service MLMaiL<br/>виклик LLM-провайдера<br/>planned]
    SpeechService[Speech Service MLMaiL<br/>виклик TTS-провайдера<br/>planned]
    GmailClient[Gmail Client MLMaiL<br/>обгортка Gmail REST API<br/>planned]
    AuthStore[Auth Store MLMaiL<br/>фасад над auth_* / gmail_inbox_count<br/>implemented]
    AuthI18n[Auth Errors i18n MLMaiL<br/>kind → українська строка<br/>implemented]
    NotesBridge[Notes Bridge MLMaiL<br/>invoke Tauri-команд для .md<br/>planned]

    Bootstrap --> AppShell
    AppShell --> Router
    Router --> Auth
    Router --> Inbox
    Router --> Reader

    Reader --> Actions
    Actions --> Drafter

    Auth --> AuthStore
    Auth --> AuthI18n
    Inbox --> GmailClient
    Reader --> GmailClient
    Reader --> SummaryService
    Reader --> SpeechService
    Drafter --> GmailClient

    Actions --> NotesBridge
    Actions --> GmailClient
```

### Компонент Frontend Bootstrap MLMaiL

Frontend Bootstrap MLMaiL — точка входу контейнера MLMaiL Frontend, файл
[app/src/main.js](../../app/src/main.js). Він монтує кореневий Vue-застосунок у
DOM-елемент `#app` з [app/index.html](../../app/index.html). Завдяки
`unplugin-auto-import` глобальна функція `createApp` доступна без явного імпорту.

Зараз файл містить мінімум:

```js
import App from './App.vue'

createApp(App).mount('#app')
```

У цільовій реалізації MLMaiL Frontend Bootstrap MLMaiL також підключає
маршрутизатор (vue-router) і реєструє auto-imported layouts.

### Компонент App Shell MLMaiL

App Shell MLMaiL — кореневий Vue-компонент, файл [app/src/App.vue](../../app/src/App.vue).
У стартовому шаблоні MLMaiL це демо-сторінка з логотипами Tauri/Vue/Vite і
формою `greet`, що викликає Tauri-команду `greet` (див.
[04-code.md](04-code.md)).

У цільовій реалізації MLMaiL App Shell MLMaiL стає тонкою обгорткою
`<router-view>` + `<layout>`, делегуючи всю поведінку дочірнім компонентам:
Auth Component MLMaiL, Inbox List Component MLMaiL, Mail Reader Component MLMaiL.

### Компонент Auth Component MLMaiL (implemented)

Auth Component MLMaiL — Vue-компонент Login-екрану MLMaiL
([app/src/views/Login.vue](../../app/src/views/Login.vue)). Має дві гілки UI:

- не авторизовано → кнопка "Увійти через Google" (заблокована поки триває
  логін, текст змінюється на "Зачекайте…");
- авторизовано → "Ви увійшли як {email}", рядок "Листів у скриньці: N",
  картка випадкового листа (Від/Тема/Дата + тіло у `<pre>`) і кнопка
  "Показати інший", яка обирає інший випадковий лист; внизу кнопка "Вийти";
- помилка останньої спроби логіну — український рядок з Auth Errors i18n MLMaiL.

Auth Component MLMaiL **не торкається токенів** і не знає про OAuth — він
лише викликає методи Auth Store MLMaiL (`login`, `logout`, `initialize`).
OAuth-механіка живе у контейнері MLMaiL Backend (Auth Module MLMaiL нижче).

Залежить від: Auth Store MLMaiL, Auth Errors i18n MLMaiL.

### Компонент Auth Store MLMaiL (implemented)

Auth Store MLMaiL — реактивний singleton-composable
([app/src/services/auth-store.js](../../app/src/services/auth-store.js)). Тримає
**лише UI-стан** (`email`, `isAuthenticated`, `isLoading`, `errorKind`,
`inboxCount`, `inboxErrorKind`, `currentMessage`, `messageErrorKind`,
`isMessageLoading`) у Vue `ref`-ах і виставляє readonly-обгортки назовні.
Жодних токенів у JS-пам'яті — єдиний доступ до access token іде через
`getAccessToken()`, який під капотом викликає Tauri-команду
`auth_get_access_token` і повертає рядок. `inboxCount` оновлюється методом
`refreshInboxCount()`, а `currentMessage` — методом `loadRandomMessage()`.
Обидва автоматично викликаються після `initialize()` і `login()`; при
отриманні `ReauthRequired` від Gmail стор сам переводить юзера у стан
«не залогінений».

Поверхня Auth Store MLMaiL: `initialize`, `login`, `getAccessToken`, `logout`,
`refreshInboxCount`, `loadRandomMessage` — та readonly-`ref`-и стану.

Залежить від: контейнера MLMaiL Backend (через `@tauri-apps/api/core::invoke`,
команди `auth_*`, `gmail_inbox_count`, `gmail_random_message`).

### Компонент Auth Errors i18n MLMaiL (implemented)

Auth Errors i18n MLMaiL — словник
([app/src/i18n/auth-errors.js](../../app/src/i18n/auth-errors.js)), що мапить
англомовний `kind` з Rust `AuthError`/`GmailError` у українську строку для UI.
Десять ключів (`Cancelled`, `Network`, `OAuth`, `Storage`, `ReauthRequired`,
`Platform`, `Http`, `Parse`, `Empty`, `Unknown`) — `errorMessage(kind)`
повертає українську строку або `"Невідома помилка."` для невідомих kind.
`Http`/`Parse`/`Empty` — для помилок Gmail-шару.

### Компонент Gmail Client MLMaiL (planned)

Gmail Client MLMaiL — обгортка над Gmail REST API для MLMaiL. Інкапсулює
HTTPS-виклики, додавання заголовку `Authorization: Bearer <token>` з Auth Store
MLMaiL, retry при `401` через Auth Store MLMaiL.

Поверхня Gmail Client MLMaiL відповідає мінімальному набору сценаріїв MLMaiL:

- `listInbox(maxResults)` — отримати останні листи з вхідних;
- `getMessage(id)` — отримати тіло конкретного листа;
- `trashMessage(id)` — видалити лист (помістити в кошик);
- `createFilter(criteria, action)` — створити Gmail-фільтр для дії
  `delete + filter`;
- `createDraft(message)` — створити чернетку відповіді.

### Компонент Inbox List Component MLMaiL (planned)

Inbox List Component MLMaiL — Vue-компонент списку вхідних листів MLMaiL.
Використовує Gmail Client MLMaiL, щоб отримати список, рендерить рядок на лист,
обробляє вибір листа і переходить на маршрут Mail Reader Component MLMaiL.

### Компонент Mail Reader Component MLMaiL (planned)

Mail Reader Component MLMaiL — Vue-компонент перегляду одного листа Gmail у
MLMaiL. Відповідає за оркестрацію наступних кроків сценарію MLMaiL:

1. отримати тіло листа через Gmail Client MLMaiL;
2. запитати саммері у Summary Service MLMaiL;
3. передати саммері у Speech Service MLMaiL і відтворити аудіо;
4. показати Action Bar Component MLMaiL з чотирма діями;
5. при виборі `reply` — показати Reply Drafter Component MLMaiL.

### Компонент Summary Service MLMaiL (planned)

Summary Service MLMaiL — клієнт LLM-провайдера у контейнері MLMaiL Frontend.
Приймає тіло листа (текст + метадані) і повертає коротке саммері.

Конкретний LLM-провайдер для MLMaiL обере ADR (див. [decisions.md](decisions.md)).
До того часу Summary Service MLMaiL описаний як абстракція: одна функція
`summarize(messageBody): Promise<string>`.

### Компонент Speech Service MLMaiL (planned)

Speech Service MLMaiL — клієнт TTS-провайдера у контейнері MLMaiL Frontend.
Приймає текст саммері і відтворює аудіо в межах WebView.

Базовий кандидат для MLMaiL — браузерний `SpeechSynthesis` API (працює без
мережі і без ключів). Перевірка доступності на Android System WebView —
завдання майбутньої реалізації MLMaiL.

### Компонент Action Bar Component MLMaiL (planned)

Action Bar Component MLMaiL — Vue-компонент з чотирма діями над поточним листом
у MLMaiL:

- `delete` — Gmail Client MLMaiL `trashMessage(id)`;
- `delete + filter` — Gmail Client MLMaiL `trashMessage(id)` плюс
  `createFilter(…)` для відправника/теми;
- `save → home` — Notes Bridge MLMaiL пише `.md` у `notes/home/`;
- `save → work` — Notes Bridge MLMaiL пише `.md` у `notes/work/`;
- після будь-якої з дій — перехід до Reply Drafter Component MLMaiL для
  останнього кроку (підготовка чернетки відповіді).

### Компонент Reply Drafter Component MLMaiL (planned)

Reply Drafter Component MLMaiL — Vue-компонент, що показує запропоновану AI
чернетку відповіді на лист і дає користувачу MLMaiL відредагувати її перед
відправкою. Сама відправка — через Gmail Client MLMaiL `createDraft(message)`
(прямі відправлення без перегляду навмисно не передбачені у MLMaiL — користувач
завжди контролює фінальний крок).

### Компонент Notes Bridge MLMaiL (planned)

Notes Bridge MLMaiL — тонка обгортка над IPC у MLMaiL: викликає Tauri-команди
контейнера MLMaiL Backend для запису і читання `.md`-заміток у контейнері
Локальне сховище MLMaiL.

Поверхня Notes Bridge MLMaiL:

- `saveNote(kind: 'work' | 'home', message): Promise<void>`;
- `listNotes(kind: 'work' | 'home'): Promise<Note[]>` (для майбутнього перегляду).

Реалізація `invoke('save_note', …)` чекає на відповідні Tauri-команди у
контейнері MLMaiL Backend (див. нижче).

## Компоненти контейнера MLMaiL Backend

```mermaid
graph TB
    EntryMain[Backend Entry main MLMaiL<br/>app/src-tauri/src/main.rs]
    EntryLib[Backend Entry lib MLMaiL<br/>app/src-tauri/src/lib.rs]
    AuthMod[Auth Module MLMaiL<br/>app/src-tauri/src/auth/<br/>implemented]
    GmailMod[Gmail Module MLMaiL<br/>app/src-tauri/src/gmail/<br/>implemented]
    NotesCmd[Notes Commands MLMaiL<br/>save_note / list_notes<br/>planned]
    Opener[Plugin Opener MLMaiL<br/>tauri-plugin-opener]
    Capabilities[Capabilities default MLMaiL<br/>capabilities/default.json]

    EntryMain --> EntryLib
    EntryLib --> AuthMod
    EntryLib --> GmailMod
    EntryLib --> NotesCmd
    EntryLib --> Opener
    GmailMod --> AuthMod
    Capabilities -. дозволяє .-> EntryLib
```

### Компонент Backend Entry main MLMaiL

Backend Entry main MLMaiL — файл [app/src-tauri/src/main.rs](../../app/src-tauri/src/main.rs).
Це **тонкий** `main.rs`: на не-debug збірках MLMaiL вимикає консольне вікно
Windows атрибутом `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
і викликає `mlmail_lib::run()`. Уся логіка контейнера MLMaiL Backend живе у
crate-бібліотеці `mlmail_lib` (див. `[lib]` секцію [app/src-tauri/Cargo.toml](../../app/src-tauri/Cargo.toml)).

### Компонент Backend Entry lib MLMaiL

Backend Entry lib MLMaiL — функція `run()` у файлі
[app/src-tauri/src/lib.rs](../../app/src-tauri/src/lib.rs). Вона будує
`tauri::Builder` для MLMaiL, реєструє плагіни і Tauri-команди:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

Атрибут `#[cfg_attr(mobile, tauri::mobile_entry_point)]` робить ту саму
функцію точкою входу для Android-збірки MLMaiL.

### Компонент Auth Module MLMaiL (implemented)

Auth Module MLMaiL — Rust-підсистема у
[app/src-tauri/src/auth/](../../app/src-tauri/src/auth/), що реалізує всю
Google OAuth-механіку MLMaiL. Підкомпоненти:

| Файл | Роль |
| ---- | ---- |
| `mod.rs` | П'ять Tauri-команд: `auth_start_login`, `auth_get_access_token`, `auth_is_authenticated`, `auth_current_email`, `auth_logout`; pub helper `acquire_access_token` (його викликають як `auth_get_access_token`, так і Gmail Module MLMaiL); `on_startup` для відновлення сесії при холодному старті |
| `state.rs` | `AuthState` (in-memory access token + email + expiry); `is_access_token_fresh()` з 30-секундним буфером |
| `pkce.rs` | Генератор PKCE pair (verifier 43 chars URL-safe, challenge = base64url(SHA-256(verifier))) |
| `token_exchange.rs` | HTTPS-обмін до `oauth2.googleapis.com/token`: `exchange_code` (auth code → tokens) і `exchange_refresh` (refresh → access); 400 + `invalid_grant` мапиться в `AuthError::ReauthRequired` |
| `id_token.rs` | Витяг `email` з JWT payload (без верифікації — токен прийшов прямо з Google по HTTPS, потрібен лише для UI) |
| `error.rs` | `AuthError` enum з serde-серіалізацією `{kind, message}`; `StorageError` |
| `config.rs` | OAuth client IDs з `option_env!` (compile-time) |
| `storage/mod.rs` | Trait `RefreshTokenStorage` (save/load/clear) + platform_storage factory |
| `storage/macos.rs` | Реалізація через crate `keyring` → Apple Keychain (service `com.vitaliytv.mlmail`, окремі entries для email і refresh_token) |
| `storage/android.rs` | Реалізація через JNI-міст до Kotlin `MlmailAuthPlugin.saveSession/loadSession/clearSession` |
| `flow/macos.rs` | Loopback HTTP server на `127.0.0.1:RANDOM_PORT` + `tauri-plugin-opener` для системного браузера + 5-хв таймаут + CSRF state перевірка |
| `flow/android.rs` | Tauri 2 mobile plugin виклик до Kotlin `signInAndAuthorize` (Credential Manager + AuthorizationClient) + token exchange |

Покриття тестами: 32 unit-тести (PKCE, ID token, state, token_exchange з
mockito, InMemoryStorage, parser callback-запиту, URL-кодування).

### Компонент Gmail Module MLMaiL (implemented)

Gmail Module MLMaiL — Rust-підсистема у
[app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/), що інкапсулює
HTTPS-виклики до Gmail REST API від імені MLMaiL Backend. У цій ітерації
експортує одну Tauri-команду:

- `gmail_inbox_count() -> Result<u64, GmailError>` — повертає точне
  `messagesTotal` мітки `INBOX` через `GET users/me/labels/INBOX`. Під капотом
  кличе `auth::acquire_access_token` (спільний шлях рефрешу токена). На 401 від
  Gmail повертає `ReauthRequired` — Auth Store MLMaiL переводить UI у стан «не
  залогінений».
- `gmail_random_message() -> Result<GmailMessage, GmailError>` — обирає
  випадковий id серед перших 100 листів INBOX (`messages.list?maxResults=100`),
  тягне `messages.get?format=full`, парсить headers (From/Subject/Date) і body
  (через `extract_plain_text`). На порожню скриньку повертає `GmailError::Empty`.

Підкомпоненти:

| Файл | Роль |
| ---- | ---- |
| `mod.rs` | Tauri-команди `gmail_inbox_count` / `gmail_random_message`; HTTP helpers `fetch_inbox_count_at` / `list_inbox_ids_at` / `get_message_at` (URL — параметр для unit-тестів через `mockito`); `parse_messages_total` |
| `message.rs` | `GmailMessage` DTO (`id, from, subject, date, body`), `extract_header` (case-insensitive header lookup), `extract_plain_text` (рекурсивний обхід `payload.parts` з пріоритетом `text/plain` і fallback на `text/html` зі стрипом тегів через `regex` + `html-escape`) |
| `error.rs` | `GmailError { Network, Http{status,body}, Parse, ReauthRequired, Platform, Empty }` + конверсії з `reqwest::Error` і `AuthError` |

Покриття тестами: 30 unit-тестів (parse_messages_total, fetch_inbox_count_at,
list_inbox_ids_at, get_message_at з mockito; extract_header,
extract_plain_text для plain/html/multipart-alternative/nested/empty;
GmailError serde + конверсії).

Залежить від: Auth Module MLMaiL (через `acquire_access_token`), `reqwest`,
`serde_json`.

### Компонент Notes Commands MLMaiL (planned)

Notes Commands MLMaiL — набір Tauri-команд для роботи з контейнером Локальне
сховище MLMaiL:

- `save_note(kind: NoteKind, message: GmailMessage) -> Result<NotePath, AppError>`;
- `list_notes(kind: NoteKind) -> Result<Vec<NoteSummary>, AppError>`;
- (можливо) `delete_note(path: NotePath) -> Result<(), AppError>`.

Команди Notes Commands MLMaiL використовують Tauri `app_data_dir()` як корінь
для теки `notes/`, нормалізують ім'я файлу за шаблоном
`YYYYMMDD-HHMMSS-<gmail-message-id>.md` і записують Markdown-замітку MLMaiL з
frontmatter (від кого, дата, тема). Точну схему фіксуватиме ADR.

### Компонент Plugin Opener MLMaiL

Plugin Opener MLMaiL — `tauri-plugin-opener`, ініціалізований у Backend Entry
lib MLMaiL. На клієнтській стороні MLMaiL надає API для відкриття URL/файлів у
системному застосунку — використовується, зокрема, для відкриття системного
браузера у OAuth flow MLMaiL (див. Auth Component MLMaiL).

### Компонент Capabilities default MLMaiL

Capabilities default MLMaiL — файл
[app/src-tauri/capabilities/default.json](../../app/src-tauri/capabilities/default.json).
Описує дозволи для головного вікна MLMaiL:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default"]
}
```

Будь-який новий Tauri-API у MLMaiL (наприклад, `fs:allow-write-text-file` для
записів `.md`) має бути доданий саме сюди — без цього інакше IPC-виклики MLMaiL
будуть відхилені рантаймом Tauri.

## Тести рівня Components MLMaiL

Юніт- і компонентних тестів MLMaiL поки немає. Цільові мінімальні тести
компонентів MLMaiL:

- Vue-компоненти MLMaiL — Vitest + `@vue/test-utils` (mount + поведінка);
- Tauri-команди MLMaiL — `cargo test` для чистої Rust-логіки і
  `tauri::test::mock_runtime` для команд з handle;
- Notes Bridge MLMaiL — тест на круг `saveNote → list_notes` проти
  тимчасової теки `app_data_dir()`.

Це **прогалина**, яку слід заповнити одночасно з реалізацією відповідних
компонентів MLMaiL.
