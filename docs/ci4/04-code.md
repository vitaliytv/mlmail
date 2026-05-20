# C4 рівень 4 — Code для MLMaiL

Code-level діаграма MLMaiL описує **ключові файли, функції і структури даних** із компонентів [03-components.md](03-components.md). Це найдетальніший рівень C4-моделі MLMaiL; він **навмисно** обмежений тим, що дійсно живе у репозиторії, плюс цільовими сигнатурами для запланованих елементів. Усе, що інакше випливає з конфігів чи коду, не дублюється тут.

## Code: контейнер MLMaiL Frontend

### Файл [app/index.html](../../app/index.html)

HTML-shell контейнера MLMaiL Frontend. Тримає `<div id="app">` і завантажує `/src/main.js` як ES-модуль. Заголовок вікна MLMaiL — `MLMaiL`.

### Файл [app/src/main.js](../../app/src/main.js)

Точка входу MLMaiL Frontend Bootstrap. Реєструє Quasar 2 як plugin із Material Symbols Outlined іконками та `config.dark: 'auto'` (наслідує OS prefers-color-scheme). `createApp` доступний без імпорту завдяки `unplugin-auto-import`. Quasar bootstraps базовий CSS та шрифт-набір. Плагіни `Notify`/`Dialog` не підключено.

### Файл [app/src/quasar-variables.sass](../../app/src/quasar-variables.sass)

Sass-таблиця, що перевизначає Quasar-defaults для macOS material-look: `$primary: #0a84ff` (macOS Accent Blue), `$typography-font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Inter', system-ui, sans-serif`, `$button-border-radius: 6px`, `$generic-border-radius: 8px`. Підхоплюється через `@quasar/vite-plugin` у файлі `app/vite.config.js` — поле `sassVariables` резолвиться як абсолютний шлях через `fileURLToPath` (відносний шлях не працює — Sass шукає відносно `node_modules/quasar/src/css/`).

### Файл [app/src/test-utils/quasar.js](../../app/src/test-utils/quasar.js)

Test helper `mountWithQuasar(component, options)` — обгортка над `mount()` з `@vue/test-utils`, що рендерить компонент усередині `q-layout > q-page-container` і глобально реєструє Quasar plugin із `dark: false`. Потрібен бо `q-page` вимагає Layout-контексту через CSS custom properties — без цієї обгортки `q-page` рендерить порожній вивід. Використовується Vue-компонентними тестами MLMaiL.

### Файл [app/src/App.vue](../../app/src/App.vue)

Кореневий компонент App Shell MLMaiL. Тонка обгортка над `<q-layout view="hHh lpR fFf">` + `<q-page-container>` (Quasar bootstrap layout, без drawer/header/footer поки що), рендерить `<Login />`. Global CSS / dark mode — на боці Quasar (`config.dark: 'auto'`).

### Файл [app/vite.config.js](../../app/vite.config.js)

Vite-конфіг MLMaiL Frontend. Ключові рішення збірки MLMaiL:

- плагін Vue через `vue-macros` (а не прямий `@vitejs/plugin-vue`); опція `template: { transformAssetUrls }` передається з `@quasar/vite-plugin`;
- Quasar через `@quasar/vite-plugin` (не Quasar CLI — зберігає Tauri-сумісність із `unplugin-auto-import`, `vue-macros`, `vite-plugin-vue-layouts-next`) з `sassVariables` (абсолютний шлях через `fileURLToPath`);
- auto-import API `vue` і `vue-router` через `unplugin-auto-import`;
- layouts через `vite-plugin-vue-layouts-next`;
- поле `include` у Vitest-конфізі звужено до `'src/**/*.vitest.{js,vue}'` (dual-runner стратегія);
- dev-сервер MLMaiL — порт `1420` зі `strictPort: true` (точно той порт, що очікує Tauri);
- HMR через WebSocket при `TAURI_DEV_HOST` (потрібно для Android dev);
- ігнор watch для `src-tauri/` (зміни у Rust не повинні тригерити Vite HMR).

### Файл [app/package.json](../../app/package.json)

Маніфест MLMaiL Frontend і workspace `app`. Скрипти:

- `dev` — `vite`;
- `build` — `vite build`;
- `preview` — `vite preview`;
- `tauri` — обгортка `tauri` CLI;
- `android` — `tauri android dev`;
- `test` — `bun test src/services src/i18n` (bun:test для pure-JS);
- `test:ui` — `vitest run` (Vitest для Vue SFC, файли `*.vitest.js`);
- `test:rust` — `cargo test --manifest-path src-tauri/Cargo.toml`;
- `test:all` — послідовно `bun test`, `bunx vitest run`, `bun run test:rust`.

Виробничі залежності MLMaiL Frontend: `@tauri-apps/api`, `vue`, `quasar`, `@quasar/extras`. Дев-залежності: `@tauri-apps/cli`, `@quasar/vite-plugin`, `@vue/test-utils`, `happy-dom`, `unplugin-auto-import`, `vite`, `vite-plugin-vue-layouts-next`, `vue-macros`, `sass`, `vitest`.

### Файл [app/bunfig.toml](../../app/bunfig.toml)

```toml
[install]
linker = "hoisted"

[test]
preload = ["./setup-happy-dom.ts"]
```

Hosted-лінкер потрібен для сумісності з інструментами, що не розуміють ізольований layout Bun. Секція `[test]` preload завантажує `happy-dom` через `GlobalRegistrator.register()` перед кожним тест-файлом — надає `document`, `window`, `HTMLElement` для `@vue/test-utils`.

### Файл [app/setup-happy-dom.ts](../../app/setup-happy-dom.ts)

```ts
import { GlobalRegistrator } from '@happy-dom/global-registrator'
GlobalRegistrator.register()
```

DOM-ініціалізація для bun:test. Виконується перед кожним тест-файлом через `bunfig.toml` preload. Надає повний DOM API для Vue-компонентних тестів MLMaiL.

### Файл [app/jsconfig.json](../../app/jsconfig.json)

JS-конфіг IDE для MLMaiL Frontend (path-aliases, `module: ESNext`). Вплив на рантайм MLMaiL відсутній — лише підказки редактора.

### Файл [app/src/views/Login.vue](../../app/src/views/Login.vue)

Auth Component MLMaiL — Vue-компонент Login-екрану. Побудований на Quasar-компонентах (`q-page` з класом `column items-center`, `q-btn`, `q-card`, `q-chip`, `q-banner`, `q-skeleton`, `q-separator`).

Клас `column items-center` замість `flex flex-center` — щоб контент із довгим тілом листа не зміщувався за нижній край viewport: `flex-center` встановлює `justify-content: center` і `align-items: center` одночасно, що при контенті вищому за viewport перекриває прокрутку.

Іконки з префіксом `sym_o_` (material-symbols-outlined): `sym_o_mail`, `sym_o_refresh`, `sym_o_logout`, `sym_o_login`. Без цього префікса Quasar генерує CSS-клас `.material-icons` замість `.material-symbols-outlined` — браузер відображає сирий текст рядка.

Дві гілки шаблону (авторизований / не авторизований), обидві українською. Підключає Auth Store MLMaiL і Auth Errors i18n MLMaiL.

Тести: `app/src/views/Login.vitest.js` (Vitest + `mountWithQuasar` + `vi.hoisted()` для mock-об'єктів + мок `@tauri-apps/api/core`). `vi.hoisted()` обов'язковий для mock-об'єктів із `vi.fn()` — Vitest hoistує `vi.mock()` на початок файлу, тому `vi.fn()` у звичайній `const` недоступний у момент hoisting.

### Файл [app/src/services/auth-store.js](../../app/src/services/auth-store.js)

Auth Store MLMaiL — singleton-composable з реактивними `ref`-ами (`email`, `isAuthenticated`, `isLoading`, `errorKind`, `inboxCount`, `inboxErrorKind`, `currentMessage`, `messageErrorKind`, `isMessageLoading`) і методами `initialize`, `login`, `getAccessToken`, `logout`, `refreshInboxCount`, `loadRandomMessage`.

`initialize()` і `login()` після успіху самі викликають `refreshInboxCount` (Tauri-команда `gmail_inbox_count`) і `loadRandomMessage` (Tauri-команда `gmail_random_message`). На `ReauthRequired` стор сам скидає `email`/`isAuthenticated`. Жодних OAuth-токенів у JS-пам'яті — стор зберігає лише `email`, `isAuthenticated`, `isLoading`, `errorKind` та Gmail-дані.

Експортує `_resetForTest()` для ізоляції тестів. Тести: `app/src/services/auth-store.test.js` (bun:test, `mock.module()`).

### Файл [app/src/i18n/auth-errors.js](../../app/src/i18n/auth-errors.js)

Auth Errors i18n MLMaiL — словник `kind` → українська строка. Десять ключів: `Cancelled`, `Network`, `OAuth`, `Storage`, `ReauthRequired`, `Platform`, `Http`, `Parse`, `Empty`, `Unknown`. Fallback — `"Невідома помилка."`. `Empty` → «Скринька порожня.». Тести: `app/src/i18n/auth-errors.test.js` (bun:test).

### Планова сигнатура: Gmail Client MLMaiL

```js
// app/src/services/gmail-client.js — planned
export async function listInbox(maxResults = 25) { /* ... */ }
export async function getMessage(id) { /* ... */ }
export async function trashMessage(id) { /* ... */ }
export async function createFilter(criteria, action) { /* ... */ }
export async function createDraft(message) { /* ... */ }
```

Файл `app/src/services/gmail-client.js` ще не існує (planned). Сигнатури вище — договір, від якого має відштовхуватися реалізація і тести MLMaiL.

### Планова сигнатура: Notes Bridge MLMaiL

```js
// app/src/services/notes-bridge.js — planned
import { invoke } from '@tauri-apps/api/core'

export async function saveNote(kind /* 'work' | 'home' */, message) {
  return invoke('save_note', { kind, message })
}
export async function listNotes(kind /* 'work' | 'home' */) {
  return invoke('list_notes', { kind })
}
```

Файл `app/src/services/notes-bridge.js` ще не існує (planned). Парний контракт — з боку контейнера MLMaiL Backend.

## Code: контейнер MLMaiL Backend

### Файл [app/src-tauri/Cargo.toml](../../app/src-tauri/Cargo.toml)

Маніфест Rust-крейту MLMaiL. Ключові рішення:

- `name = "mlmail"`, `edition = "2021"`;
- `[lib] name = "mlmail_lib"` і `crate-type = ["staticlib", "cdylib", "rlib"]`;
- `build-dependencies`: `tauri-build = "2"`;
- `dependencies`: `tauri = "2"`, `tauri-plugin-opener = "2"`, `serde = "1"` (з `derive`), `serde_json = "1"`, `reqwest = "0.12"` (json + rustls-tls), `tokio = "1"` (net/io-util/time/macros/rt/sync), `base64 = "0.22"`, `rand = "0.9"`, `sha2 = "0.10"`, `log = "0.4"`, `thiserror = "2"`, `dotenvy = "0.15.7"`, `regex`, `html-escape`;
- `[target.'cfg(target_os = "android")'.dependencies]`: `jni = "0.21"`;
- `[dev-dependencies]`: `mockito = "1"`, `tokio` з повним runtime, `tempfile`.

Залежність `keyring` (apple-native) видалена після переходу на `FileStorage` — системний Keychain-промпт при кожній перебудові `tauri dev` більше не виникає.

### Файл [app/src-tauri/build.rs](../../app/src-tauri/build.rs)

Скрипт збірки MLMaiL Backend, генерує контекст Tauri (іконки, capabilities). Не несе додаткової логіки MLMaiL.

### Файл [app/src-tauri/src/main.rs](../../app/src-tauri/src/main.rs)

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mlmail_lib::run()
}
```

Точка входу desktop-бінарника MLMaiL. На реліз-збірках під Windows вимкнено консольне вікно атрибутом `cfg_attr`. Уся логіка делегована функції `run()` крейту `mlmail_lib`.

### Файл [app/src-tauri/src/lib.rs](../../app/src-tauri/src/lib.rs)

Реальна точка ініціалізації MLMaiL Backend. Декларує `pub mod auth;`, `pub mod gmail;`, `pub mod endpoints;`. Послідовність старту:

1. `dotenvy::dotenv().ok()` та `dotenvy::from_filename(".env.secret").ok()` — завантажує `app/src-tauri/.env` і `.env.secret` у runtime (не compile-time); зміна `.env` підхоплюється перезапуском без `cargo clean`.
2. Створює `SharedStorage` через `auth::make_storage(&app)` і реєструє через `.manage()`.
3. Реєструє `Endpoints::default()` (реальні Google URLs) через `.manage()`.
4. Викликає `auth::on_startup(&app.handle())`.
5. Реєструє у `invoke_handler!` сім команд: `auth_start_login`, `auth_finalize_login`, `auth_get_access_token`, `auth_is_authenticated`, `auth_current_email`, `auth_logout`, `gmail_inbox_count`, `gmail_random_message`.

Точки розширення MLMaiL Backend: `invoke_handler` — додавати нові Tauri-команди (`save_note`, `list_notes`); `.plugin(...)` — нові Tauri-плагіни; `#[cfg_attr(mobile, tauri::mobile_entry_point)]` — та сама `run()` є точкою входу для Android-збірки.

### Файл [app/src-tauri/src/endpoints.rs](../../app/src-tauri/src/endpoints.rs)

```rust
pub struct Endpoints {
    pub google_token: String,
    pub gmail_label_inbox: String,
    pub gmail_messages_list: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            google_token: "https://oauth2.googleapis.com/token".into(),
            gmail_label_inbox:
                "https://gmail.googleapis.com/gmail/v1/users/me/labels/INBOX".into(),
            gmail_messages_list:
                "https://gmail.googleapis.com/gmail/v1/users/me/messages".into(),
        }
    }
}
```

Dependency-injection структура для URL-ів зовнішніх сервісів MLMaiL. Реєструється у `app/src-tauri/src/lib.rs` через `.manage(Endpoints::default())`. У тестах підставляється `mockito`-адреса замість реальних Google URL — без зміни prod-коду.

### Каталог [app/src-tauri/src/auth/](../../app/src-tauri/src/auth/)

Auth Module MLMaiL — реалізація Google OAuth для контейнера MLMaiL Backend. Структура файлів:

```text
auth/
├── mod.rs              — Tauri commands, on_startup, acquire_access_token
├── state.rs            — AuthState (access_token, email, expiry: Option<Instant>)
├── pkce.rs             — PKCE generator (verifier, challenge)
├── token_exchange.rs   — exchange_code_at / exchange_refresh_at → oauth2.googleapis.com/token
├── id_token.rs         — JWT email extract
├── error.rs            — AuthError + StorageError
├── config.rs           — std::env::var() + OnceLock OAuth client IDs
├── storage/
│   ├── mod.rs          — RefreshTokenStorage trait + SharedStorage alias + platform_storage()
│   ├── file.rs         — FileStorage (session.json, mode 0600, атомарний rename)
│   ├── android.rs      — JNI до MlmailAuthPlugin
│   └── in_memory.rs    — тест-only InMemoryStorage
└── flow/
    ├── mod.rs
    ├── macos.rs        — loopback HTTP server + tauri-plugin-opener
    └── android.rs      — Tauri mobile plugin виклик до Kotlin
```

`SharedStorage = Arc<dyn RefreshTokenStorage + Send + Sync>` — реєструється через Tauri managed state. `platform_storage(path)` повертає `FileStorage` на macOS і `AndroidStorage` на Android.

Шість Tauri-команд: `auth_start_login`, `auth_finalize_login`, `auth_get_access_token`, `auth_is_authenticated`, `auth_current_email`, `auth_logout`. Внутрішній helper `acquire_access_token` — рефрешить access token при потребі (`expiry` + 30 с буфер).

`app/src-tauri/src/auth/config.rs` читає `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID`, `MLMAIL_GOOGLE_ANDROID_CLIENT_ID`, `MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID`, `MLMAIL_GOOGLE_DESKTOP_CLIENT_SECRET` через `std::env::var()` + `OnceLock<String>` при першому зверненні. Вибір `std::env::var()` замість `option_env!()` (compile-time) — зміна `.env` підхоплюється перезапуском без `cargo clean`; помилки відсутнього env var виявляються в runtime, а не compile time.

Тести: `cargo test --lib auth` (≥32 unit-тести). Integration-тести: `app/src-tauri/tests/auth_commands.rs`, `auth_logout.rs`, `auth_finalize_login.rs`, `acquire_access_token.rs`.

TBD: tracing-storage (auth module виконує OAuth-виклики до Google Identity Services).

### Файл [app/src-tauri/src/auth/storage/file.rs](../../app/src-tauri/src/auth/storage/file.rs)

`FileStorage` MLMaiL — зберігає refresh token у JSON-файлі `{app_data_dir}/session.json` з правами `0600`. Атомарний запис: temp-файл відкривається одразу з `OpenOptions::mode(0o600)`, потім POSIX `rename` замінює цільовий — закриває TOCTOU-вразливість схеми write→chmod.

`FileStorage` використовується на macOS і в release (не лише debug): системний Keychain-промпт зникає повністю, бо токен прив'язується до файлу, а не до ACL бінарника. При переході з Keychain на `FileStorage` потрібен одноразовий повторний логін.

Реалізує `RefreshTokenStorage`. 7 unit-тестів (всі зелені).

### Каталог [app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/)

Gmail Module MLMaiL — Rust-підсистема для Gmail REST API. Структура:

```text
gmail/
├── mod.rs     — Tauri commands + HTTP helpers
├── message.rs — GmailMessage DTO + extract_header + extract_plain_text
└── error.rs   — GmailError + From-конверсії
```

**`gmail_inbox_count(app, state, endpoints) -> Result<u64, GmailError>`** — викликає `acquire_access_token` та `fetch_inbox_count_at(&endpoints.gmail_label_inbox, &token)`. Повертає `messagesTotal` з `users.labels.get?id=INBOX` — канонічний однозапитний шлях із точним числом листів INBOX (не `resultSizeEstimate`). Status mapping: 401 → `GmailError::ReauthRequired`; інші не-2xx → `GmailError::Http { status, body }`; JSON-помилки → `GmailError::Parse`.

**`gmail_random_message(app, state, endpoints) -> Result<GmailMessage, GmailError>`** — `list_inbox_ids_at(&endpoints.gmail_messages_list, &token)` з `maxResults=100&fields=messages/id` → випадковий id (`rand::random::<u64>() as usize % ids.len()`) → `get_message_at(id, &token)` → `GmailMessage { id, from, subject, date, body }`. Body truncate до 10 000 символів. Порожній INBOX → `GmailError::Empty`.

`extract_plain_text(payload)` рекурсивно обходить `payload.parts`, обираючи `text/plain` (base64url-decoded). Fallback — `text/html` зі стрипом тегів (`regex` `<[^>]+>`) і decode HTML entities (`html-escape`). Plain-text підхід усуває XSS-ризики без DOMPurify/iframe; HTML-рендер тіла листа — окрема майбутня ітерація.

`GmailError` має `#[serde(tag = "kind", content = "message")]` — frontend дістає `err.kind` так само, як для `AuthError`. `From<AuthError>` мапить `AuthError::ReauthRequired` у `GmailError::ReauthRequired`.

Тести: `cargo test --lib gmail` (≥30 unit-тестів). Integration-тести: `app/src-tauri/tests/gmail_commands.rs`.

TBD: tracing-storage (gmail module виконує HTTP-виклики до Gmail REST API).

### Каталог [app/src-tauri/tests/](../../app/src-tauri/tests/)

Integration-тести MLMaiL Backend через Tauri Mock Runtime (`tauri::test::mock_builder()`). Тестують Tauri-команди зі справжнім DI без реального WebView. WebDriver (`tauri-driver`) не підтримує macOS — Apple не надає публічний WebKit WebDriver для WKWebView у вбудованих застосунках:

```text
tests/
├── auth_commands.rs        — auth_is_authenticated, auth_current_email
├── auth_logout.rs          — auth_logout очищає storage
├── auth_finalize_login.rs  — finalize_login з mockito /token endpoint
├── acquire_access_token.rs — рефреш токена через mockito
└── gmail_commands.rs       — gmail_inbox_count, gmail_random_message з mockito
```

Усі тести підставляють `InMemoryStorage` замість `FileStorage` і `Endpoints` з mockito URL замість реальних Google URLs.

### Файл [app/src-tauri/tauri.conf.json](../../app/src-tauri/tauri.conf.json)

Конфіг застосунку MLMaiL Tauri. Ключові поля:

- `identifier: com.vitaliytv.mlmail`;
- `productName: mlmail`, `version: 0.1.0`;
- `build.beforeDevCommand: bun run dev`, `build.beforeBuildCommand: bun run build`;
- `build.devUrl: http://localhost:1420`, `build.frontendDist: ../dist`;
- одне вікно MLMaiL `800x600` із заголовком `mlmail`;
- `app.security.csp: null` — CSP зараз вимкнено; для бойової версії MLMaiL слід переглянути після ADR з безпеки.

### Файл [app/src-tauri/capabilities/default.json](../../app/src-tauri/capabilities/default.json)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default"]
}
```

Дозволи MLMaiL для головного вікна. Будь-який новий Tauri API (`fs`, `dialog`, `http`, `notification`) має додаватися сюди — без оновлення capabilities IPC-виклики MLMaiL будуть відхилені рантаймом Tauri.

### Файли [app/src-tauri/.env](../../app/src-tauri/.env) та [.env.secret](../../app/src-tauri/.env.secret)

Конфігурація розбита на два файли:

`.env` (трекується у приватному git-репозиторії):

- `MLMAIL_GOOGLE_DESKTOP_CLIENT_ID` — Desktop application OAuth client ID.
- `MLMAIL_GOOGLE_ANDROID_CLIENT_ID` — Android OAuth client ID.
- `MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID` — Web application client ID; потрібен для `setServerClientId` у Kotlin `CredentialManagerFlow.kt` (Web OAuth client ID, не Android client ID).

`.env.secret` (у `.gitignore`):

- `MLMAIL_GOOGLE_DESKTOP_CLIENT_SECRET` — Desktop client secret для token exchange.

Google OAuth Client IDs для нативних desktop/mobile-застосунків є публічними ідентифікаторами — вони передаються відкритим текстом у мережевих запитах і тривіально витягуються з бінарника командою `strings`. Тому `.env` без Secret зберігається у приватному репо.

Обидва файли завантажує `dotenvy` при старті `app/src-tauri/src/lib.rs::run()` до ініціалізації Tauri. Шаблони: `app/src-tauri/.env.example` та `app/src-tauri/.env.secret.example`.

### Планова сигнатура: Notes Commands MLMaiL

```rust
// app/src-tauri/src/notes.rs — planned

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteKind { Work, Home }

#[derive(Deserialize)]
pub struct GmailMessage {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub body_text: String,
}

#[derive(Serialize)]
pub struct NotePath(pub String);

#[derive(Serialize)]
pub struct NoteSummary {
    pub path: String,
    pub subject: String,
    pub from: String,
    pub date: String,
}

#[tauri::command]
pub fn save_note(
    app: AppHandle,
    kind: NoteKind,
    message: GmailMessage,
) -> Result<NotePath, String> {
    // записати notes/<kind>/<YYYYMMDD-HHMMSS-<id>>.md
    todo!()
}

#[tauri::command]
pub fn list_notes(
    app: AppHandle,
    kind: NoteKind,
) -> Result<Vec<NoteSummary>, String> {
    // перелічити notes/<kind>/*.md і повернути summary
    todo!()
}
```

Файл `app/src-tauri/src/notes.rs` ще не існує (planned). Реєстрація у `app/src-tauri/src/lib.rs` — додати `notes::save_note, notes::list_notes` у `tauri::generate_handler![…]`.

## Code: інструменти і конфіги MLMaiL

### Файл [package.json](../../package.json) (корінь репозиторію MLMaiL)

Кореневий маніфест Bun-монорепо MLMaiL. Лінт-pipeline MLMaiL:

- `lint` → `lint-js` + `lint-text` + `lint-style` + `lint-ga` + `lint-security`;
- `lint-js` → `bunx oxlint --fix`, `bunx eslint --fix .`, `bunx jscpd .`, `oxfmt .`;
- `lint-style` → `npx stylelint '**/*.{css,scss,vue}' --fix`;
- `lint-text` → `npx cspell .`, shellcheck через `@nitra/cursor`, `bunx markdownlint-cli2 --fix "**/*.md" "**/*.mdc"`, `v8r` через `@nitra/cursor`;
- `lint-ga` → `bunx zizmor` (GitHub Actions security audit);
- `lint-security` → `gitleaks detect --no-banner`.

`workspaces: ["app"]` — єдиний робочий пакет MLMaiL. У `devDependencies` кореня MLMaiL дозволені лише `@nitra/*` (правило `.cursor/rules/n-bun.mdc`). Пакет `@nitra/cursor` зберігається у кореневому `package.json` і не дублюється у `app/package.json` — там він невикористаний і викликав би lint-попередження `knip`.

### Файл [bunfig.toml](../../bunfig.toml)

```toml
[install]
linker = "hoisted"
```

Hosted-лінкер потрібен для сумісності з інструментами, що не розуміють ізольований layout Bun.

### Файл [.gitleaks.toml](../../.gitleaks.toml)

```toml
[extend]
useDefault = true

[allowlist]
paths = ["node_modules", ".git", "dist", "build", "*.lock", "fixtures?/"]
```

Security-лінтер MLMaiL. Запускається через `bun run lint-security` і є частиною агрегованого `bun run lint`. Відсутність `.gitleaks.toml` або скрипту `lint-security` призводить до помилки `npx @nitra/cursor check` (правило `n-security.mdc`).

### Конфіги лінтерів MLMaiL

Файли `.oxlintrc.json`, `.markdownlint-cli2.jsonc`, `.cspell.json`, `.jscpd.json`, `.stylelintignore`, `.v8rignore`, `.github/zizmor.yml` — конфіги лінтерів MLMaiL. Деталі — правила `.cursor/rules/n-js-lint.mdc`, `.cursor/rules/n-style-lint.mdc`, `.cursor/rules/n-text.mdc`. C4-модель MLMaiL не дублює їхній зміст; pipeline запускається через `bun run lint`.

Особливості конфігурації MLMaiL:

- `.markdownlint-cli2.jsonc` ігнорує `.cursor/rules/**` — ці файли перезаписуються при синхронізації `@nitra/cursor`.
- `.v8rignore` містить `app/src-tauri/target/`, `app/src-tauri/gen/`, `app/src-tauri/capabilities/`, `app/src-tauri/tauri.conf.json`, `.cursor/hooks.json`, `.gitleaks.toml` — файли без схем у Schema Store.
- `.gitignore` містить `.claude/worktrees/` — без цього `jscpd` знаходив дублікати між робочими файлами та воркдеревами.

### Файл [.vscode/settings.json](../../.vscode/settings.json)

IDE-конфіг VS Code для монорепо MLMaiL. Ключове: `"rust-analyzer.linkedProjects": ["app/src-tauri/Cargo.toml"]` — вказує rust-analyzer на `Cargo.toml` у підкаталозі, бо кореневий каталог репозиторію не містить `Cargo.toml`; без цього rust-analyzer генерує `FetchWorkspaceError`.

## Поточний стан коду MLMaiL

### Реалізовано

- `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue` (Quasar 2, `column items-center`, `sym_o_` іконки);
- `app/src/services/auth-store.js` (`inboxCount`, `currentMessage`, `loadRandomMessage`);
- `app/src/i18n/auth-errors.js` (10 ключів включно з `Empty`);
- `app/index.html`, `app/vite.config.js`, `app/src/quasar-variables.sass`;
- `app/src/test-utils/quasar.js`, `app/setup-happy-dom.ts`, `app/bunfig.toml` (test preload);
- `app/src-tauri/src/main.rs`, `app/src-tauri/src/lib.rs` з сімома Tauri-командами;
- `app/src-tauri/src/auth/` (Auth Module MLMaiL, 13 файлів включно з `storage/file.rs`);
- `app/src-tauri/src/gmail/` (Gmail Module MLMaiL, 3 файли);
- `app/src-tauri/src/endpoints.rs` (Endpoints DI для тестованості);
- `app/src-tauri/tests/` (5 integration-тестів через Tauri Mock Runtime);
- `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/` (Kotlin Auth Plugin: 4 файли);
- `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json`;
- `app/src-tauri/Cargo.toml`, `app/src-tauri/build.rs`;
- `app/src-tauri/.env`, `app/src-tauri/.env.example`, `app/src-tauri/.env.secret.example`.

### Planned

- `app/src/services/gmail-client.js`, `app/src/services/notes-bridge.js`;
- `app/src/services/summary.js`, `app/src/services/speech.js`;
- `app/src/views/Inbox.vue`, `app/src/views/Reader.vue`;
- `app/src-tauri/src/notes.rs` та реєстрація `save_note`, `list_notes` у `tauri::generate_handler![…]`.

## Тести рівня Code MLMaiL

| Тип | Команда | Покриття |
| --- | ------- | -------- |
| Rust unit | `cargo test --lib --manifest-path app/src-tauri/Cargo.toml` | auth (≥32), gmail (≥30), FileStorage (7) |
| Rust integration | `cargo test --test '*' --manifest-path app/src-tauri/Cargo.toml` | 5 тест-файлів у `tests/` (Mock Runtime + mockito) |
| JS unit | `cd app && bun test src/services src/i18n` | auth-store, auth-errors |
| Vue компоненти | `cd app && bunx vitest run` | Login.vitest.js (happy-dom + mountWithQuasar) |

Запуск усіх тестів: `cd app && bun run test:all`.