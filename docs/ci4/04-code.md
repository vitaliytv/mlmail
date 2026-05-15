# C4 рівень 4 — Code для MLMaiL

Code-level діаграма MLMaiL описує **ключові файли, функції і структури даних**
із компонентів [03-components.md](03-components.md). Це найдетальніший рівень
C4-моделі MLMaiL; він **навмисно** обмежений тим, що дійсно живе у репозиторії,
плюс цільовими сигнатурами для запланованих елементів. Усе, що інакше
випливає з конфігів чи коду, не дублюється тут.

## Code: контейнер MLMaiL Frontend

### Файл [app/index.html](../../app/index.html)

HTML-shell контейнера MLMaiL Frontend. Тримає `<div id="app">` і завантажує
`/src/main.js` як ES-модуль. Заголовок вікна MLMaiL — `MLMaiL`.

### Файл [app/src/main.js](../../app/src/main.js)

Точка входу MLMaiL Frontend Bootstrap. Поточний код:

```js
import App from './App.vue'

createApp(App).mount('#app')
```

`createApp` доступний без імпорту завдяки `unplugin-auto-import` (див.
конфіг Vite нижче). У цільовій реалізації MLMaiL Frontend Bootstrap MLMaiL також
підключає router/layouts і Auth Store MLMaiL.

### Файл [app/src/App.vue](../../app/src/App.vue)

Кореневий компонент App Shell MLMaiL. Тонка обгортка — рендерить
`<Login/>` напряму:

```vue
<script setup>
import Login from './views/Login.vue'
</script>

<template>
  <Login />
</template>
```

Стартова демо-форма `greet` видалена разом з командою `greet` у Backend
(ADR-0006). Router/layouts — окрема ітерація.

### Файл [app/vite.config.js](../../app/vite.config.js)

Vite-конфіг MLMaiL Frontend. Ключові рішення збірки MLMaiL:

- плагін Vue через `vue-macros` (а не прямий `@vitejs/plugin-vue`);
- auto-import API `vue` і `vue-router` (без `vue-router` як залежності — поки
  не доданий, але імпорти готові);
- layouts через `vite-plugin-vue-layouts-next`;
- dev-сервер MLMaiL — порт `1420` зі `strictPort: true` (точно той порт, що
  очікує Tauri);
- HMR через WebSocket при `TAURI_DEV_HOST` (потрібно для Android dev,
  щоб реальний пристрій бачив dev-сервер);
- ігнор watch для `src-tauri/` (зміни у Rust не повинні тригерити Vite HMR).

### Файл [app/package.json](../../app/package.json)

Маніфест MLMaiL Frontend і workspace `app`. Скрипти:

- `dev` — `vite` (dev-сервер на `1420`);
- `build` — `vite build` (бандл у `app/dist/`);
- `preview` — `vite preview`;
- `tauri` — обгортка `tauri` CLI;
- `android` — `tauri android dev`.

Виробничі залежності MLMaiL Frontend: `@tauri-apps/api`, `@tauri-apps/plugin-opener`,
`vue`. Дев-залежності — `@tauri-apps/cli`, `@vitejs/plugin-vue`,
`@vue/test-utils`, `jsdom`, `unplugin-auto-import`, `vite`,
`vite-plugin-vue-layouts-next`, `vitest`, `vue-macros`.

Скрипти тестування MLMaiL Frontend: `bun run test` (vitest, single run),
`bun run test:watch` (watch mode). Vite-конфіг має блок `test` з
`environment: 'jsdom'` для DOM-залежних компонентних тестів MLMaiL.

### Файл [app/jsconfig.json](../../app/jsconfig.json)

JS-конфіг IDE для MLMaiL Frontend (path-aliases, `module: ESNext`). Уплив на
рантайм MLMaiL відсутній — лише підказки редактора.

### Файл [app/src/views/Login.vue](../../app/src/views/Login.vue)

Auth Component MLMaiL — Vue-компонент Login-екрану. Дві гілки шаблону
(авторизований / не авторизований), обидві українською. Підключає Auth Store
MLMaiL і Auth Errors i18n MLMaiL. Тестується у
[app/src/views/Login.test.js](../../app/src/views/Login.test.js) через
Vitest + `@vue/test-utils` з моком `@tauri-apps/api/core`.

### Файл [app/src/services/auth-store.js](../../app/src/services/auth-store.js)

Auth Store MLMaiL — singleton-composable з реактивними `ref`-ами
(`email`, `isAuthenticated`, `isLoading`, `errorKind`, `inboxCount`,
`inboxErrorKind`, `currentMessage`, `messageErrorKind`, `isMessageLoading`) і
методами `initialize`, `login`, `getAccessToken`, `logout`, `refreshInboxCount`,
`loadRandomMessage`. `initialize()` і `login()` після успіху самі дзвонять
`refreshInboxCount` + `loadRandomMessage` (під капотом Tauri-команди
`gmail_inbox_count` та `gmail_random_message`). На `ReauthRequired` стор сам
скидає `email`/`isAuthenticated`. Експортує `_resetForTest()` для ізоляції
тестів. Тести —
[auth-store.test.js](../../app/src/services/auth-store.test.js).

### Файл [app/src/i18n/auth-errors.js](../../app/src/i18n/auth-errors.js)

Auth Errors i18n MLMaiL — словник `kind` → українська строка. Десять ключів
(`Cancelled`, `Network`, `OAuth`, `Storage`, `ReauthRequired`, `Platform`,
`Http`, `Parse`, `Empty`, `Unknown`), fallback `"Невідома помилка."`. Тести —
[auth-errors.test.js](../../app/src/i18n/auth-errors.test.js).

### Планова сигнатура: Gmail Client MLMaiL

```js
// app/src/services/gmail-client.js — planned
/**
 *
 * @param maxResults
 */
export async function listInbox(maxResults = 25) { /* ... */ }
/**
 *
 * @param id
 */
export async function getMessage(id) { /* ... */ }
/**
 *
 * @param id
 */
export async function trashMessage(id) { /* ... */ }
/**
 *
 * @param criteria
 * @param action
 */
export async function createFilter(criteria, action) { /* ... */ }
/**
 *
 * @param message
 */
export async function createDraft(message) { /* ... */ }
```

Файл `app/src/services/gmail-client.js` ще не існує (planned). Сигнатури вище —
**договір**, від якого має відштовхуватися реалізація і тести MLMaiL.

### Планова сигнатура: Notes Bridge MLMaiL

```js
// app/src/services/notes-bridge.js — planned
import { invoke } from '@tauri-apps/api/core'

/**
 *
 * @param kind
 * @param message
 */
export async function saveNote(kind /* 'work' | 'home' */, message) {
  return invoke('save_note', { kind, message })
}

/**
 *
 * @param kind
 */
export async function listNotes(kind /* 'work' | 'home' */) {
  return invoke('list_notes', { kind })
}
```

Файл `app/src/services/notes-bridge.js` ще не існує (planned). Сигнатури вище —
**договір** з боку контейнера MLMaiL Frontend; парний контракт — з боку
контейнера MLMaiL Backend (див. нижче).

## Code: контейнер MLMaiL Backend

### Файл [app/src-tauri/Cargo.toml](../../app/src-tauri/Cargo.toml)

Маніфест Rust-крейту MLMaiL. Ключові рішення:

- `name = "mlmail"`, `edition = "2021"`;
- `[lib] name = "mlmail_lib"` і `crate-type = ["staticlib", "cdylib", "rlib"]` —
  суфікс `_lib` потрібен, щоб не конфліктувати з бінарною ціллю на Windows
  (див. коментар у Cargo.toml MLMaiL);
- `build-dependencies`: `tauri-build = "2"`;
- `dependencies`: `tauri = "2"`, `tauri-plugin-opener = "2"`, `serde = "1"`
  (з `derive`), `serde_json = "1"`, `reqwest = "0.12"` (json + rustls-tls),
  `tokio = "1"` (net/io-util/time/macros/rt/sync), `base64 = "0.22"`,
  `rand = "0.9"`, `sha2 = "0.10"`, `log = "0.4"`, `thiserror = "2"`;
- `[target.'cfg(target_os = "macos")'.dependencies]`: `keyring = "3"` з
  `apple-native` feature (Apple Keychain через Security framework);
- `[target.'cfg(target_os = "android")'.dependencies]`: `jni = "0.21"`;
- `[dev-dependencies]`: `mockito = "1"`, `tokio` з повним runtime для тестів.

### Файл [app/src-tauri/build.rs](../../app/src-tauri/build.rs)

Скрипт збірки MLMaiL Backend, генерує контекст Tauri (іконки, capabilities). Не
несе додаткової логіки MLMaiL.

### Файл [app/src-tauri/src/main.rs](../../app/src-tauri/src/main.rs)

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mlmail_lib::run()
}
```

Точка входу desktop-бінарника MLMaiL. На реліз-збірках MLMaiL під Windows
вимкнено консольне вікно атрибутом `cfg_attr`. Уся логіка делегована
функції `run()` крейту `mlmail_lib`.

### Файл [app/src-tauri/src/lib.rs](../../app/src-tauri/src/lib.rs)

Реальна точка ініціалізації MLMaiL Backend. Декларує `pub mod auth;` та
`pub mod gmail;`, керує `Mutex<AuthState>` через `.manage(...)`, на старті
викликає `auth::on_startup(&app.handle())` і реєструє у `invoke_handler!`
сім команд: пʼять `auth_*` (див. вище), `gmail::gmail_inbox_count`,
`gmail::gmail_random_message`. Точне тіло — у
[lib.rs](../../app/src-tauri/src/lib.rs).

Точки розширення MLMaiL Backend:

- `invoke_handler` — додавати нові Tauri-команди MLMaiL (`save_note`, `list_notes`);
- `.plugin(...)` — додавати нові Tauri-плагіни MLMaiL;
- `#[cfg_attr(mobile, tauri::mobile_entry_point)]` — та сама функція `run()`
  є точкою входу і для Android-збірки MLMaiL, окремий `main.rs` для mobile
  **не потрібен**.

### Каталог [app/src-tauri/src/auth/](../../app/src-tauri/src/auth/)

Auth Module MLMaiL — реалізація Google OAuth для контейнера MLMaiL Backend.
Структура файлів:

```text
auth/
├── mod.rs              — Tauri commands, on_startup, glue
├── state.rs            — AuthState
├── pkce.rs             — PKCE generator
├── token_exchange.rs   — Google /token POST
├── id_token.rs         — JWT email extract
├── error.rs            — AuthError + StorageError
├── config.rs           — option_env! OAuth client IDs
├── storage/
│   ├── mod.rs          — RefreshTokenStorage trait + factory
│   ├── macos.rs        — Apple Keychain (keyring crate)
│   ├── android.rs      — JNI до MlmailAuthPlugin
│   └── in_memory.rs    — тест-only InMemoryStorage
└── flow/
    ├── mod.rs
    ├── macos.rs        — loopback HTTP server + tauri-plugin-opener
    └── android.rs      — Tauri mobile plugin виклик до Kotlin
```

Тести — `cargo test --lib auth` (32 unit-тести).

### Каталог [app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/)

Gmail Module MLMaiL — Rust-підсистема для Gmail REST API. Структура:

```text
gmail/
├── mod.rs     — Tauri commands gmail_inbox_count / gmail_random_message; HTTP helpers fetch_inbox_count_at / list_inbox_ids_at / get_message_at; parse_messages_total
├── message.rs — GmailMessage DTO + extract_header + extract_plain_text
└── error.rs   — GmailError + From-конверсії з reqwest::Error і AuthError
```

Команда `gmail_inbox_count(app, state) -> Result<u64, GmailError>` під капотом
викликає `auth::acquire_access_token` (спільний шлях рефрешу токена) і
`fetch_inbox_count_at(GMAIL_LABEL_INBOX_URL, &token)`. Тіло — два рядки;
див. [mod.rs](../../app/src-tauri/src/gmail/mod.rs).

Команда `gmail_random_message(app, state) -> Result<GmailMessage, GmailError>` —
`list_inbox_ids_at` → випадковий id (через `rand::random`) → `get_message_at` →
`GmailMessage { id, from, subject, date, body }`. Body truncate до 10 000
символів. Порожній INBOX → `GmailError::Empty`. Status mapping ідентичний
`fetch_inbox_count_at`.

`extract_plain_text(payload)` рекурсивно обходить `payload.parts`, обираючи
`text/plain` (base64url-decoded). Fallback — `text/html` зі стрипом тегів
(`regex` `<[^>]+>`) і decode HTML entities (`html-escape`).

`fetch_inbox_count_at(endpoint, access_token)` — приватний helper із URL-параметром
для unit-тестів через `mockito`. Status-маппінг: 401 → `GmailError::ReauthRequired`;
інші не-2xx → `GmailError::Http { status, body }`; JSON-помилки →
`GmailError::Parse`; `reqwest::Error` → `GmailError::Network`.

`GmailError` має `#[serde(tag = "kind", content = "message")]` — frontend дістає
`err.kind` так само, як для `AuthError`. Конверсія `From<AuthError>` мапить
`AuthError::ReauthRequired` у `GmailError::ReauthRequired`, тож і refresh-помилка,
і Gmail-401 виглядають однаково для UI.

Тести — `cargo test --lib gmail` (30 unit-тестів).

### Файл [app/src-tauri/tauri.conf.json](../../app/src-tauri/tauri.conf.json)

Конфіг застосунку MLMaiL Tauri. Ключові поля:

- `identifier: com.vitaliytv.mlmail` — bundle ID для macOS і пакет для Android;
- `productName: mlmail`, `version: 0.1.0`;
- `build.beforeDevCommand: bun run dev`, `build.beforeBuildCommand: bun run build`;
- `build.devUrl: http://localhost:1420`, `build.frontendDist: ../dist`;
- одне вікно MLMaiL `800x600` із заголовком `mlmail`;
- `app.security.csp: null` — Content Security Policy зараз вимкнено; для
  бойової версії MLMaiL це слід переглянути після ADR з безпеки.

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

Дозволи MLMaiL для головного вікна. Будь-який новий Tauri API у MLMaiL
(`fs`, `dialog`, `http`, `notification` тощо) має додаватися саме сюди —
без оновлення capabilities IPC-виклики MLMaiL будуть відхилені рантаймом
Tauri.

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
pub fn save_note(app: AppHandle, kind: NoteKind, message: GmailMessage) -> Result<NotePath, String> {
    /* записати notes/<kind>/<YYYYMMDD-HHMMSS-<id>>.md */
    todo!()
}

#[tauri::command]
pub fn list_notes(app: AppHandle, kind: NoteKind) -> Result<Vec<NoteSummary>, String> {
    /* перелічити notes/<kind>/*.md і повернути summary */
    todo!()
}
```

Файл `app/src-tauri/src/notes.rs` ще не існує (planned). Реєстрація команд у
Backend Entry lib MLMaiL — додати `notes::save_note, notes::list_notes` у
`tauri::generate_handler![…]`.

## Code: інструменти і конфіги MLMaiL

### Файл [package.json](../../package.json) (корінь репозиторію MLMaiL)

Кореневий маніфест Bun-monorepo MLMaiL. Лінт-pipeline MLMaiL:

- `lint` → `lint-js` + `lint-text` + `lint-style`;
- `lint-js` → `bunx oxlint --fix`, `bunx eslint --fix .`, `bunx jscpd .`;
- `lint-style` → `npx stylelint '**/*.{css,scss,vue}' --fix`;
- `lint-text` → `npx cspell .`, shellcheck через `@nitra/cursor`,
  `bunx markdownlint-cli2 --fix "**/*.md" "**/*.mdc"`, `v8r` через `@nitra/cursor`.

`workspaces: ["app"]` — єдиний робочий пакет MLMaiL. У `devDependencies`
кореня MLMaiL дозволені лише `@nitra/*` (правило `.cursor/rules/n-bun.mdc`).

### Файл [bunfig.toml](../../bunfig.toml)

```toml
[install]
linker = "hoisted"
```

Hoisted-лінкер потрібен MLMaiL для сумісності з інструментами, які не розуміють
ізольований layout Bun (правило `.cursor/rules/n-bun.mdc`).

### Файли `.oxlintrc.json`, `.oxfmtrc.json`, `.markdownlint-cli2.jsonc`, `.cspell.json`, `.jscpd.json`, `.stylelintignore`, `.v8rignore`

Конфіги лінтерів MLMaiL — детально розкриваються правилами
`.cursor/rules/n-js-lint.mdc`, `.cursor/rules/n-style-lint.mdc`,
`.cursor/rules/n-text.mdc`. C4-модель MLMaiL не дублює їхній зміст: достатньо
знати, що pipeline MLMaiL працює через `bun run lint`.

## Поточний стан коду MLMaiL

Реалізовано:

- `app/src/main.js`, `app/src/App.vue`, `app/src/views/Login.vue`,
  `app/src/services/auth-store.js`, `app/src/i18n/auth-errors.js`,
  `app/index.html`, `app/vite.config.js`;
- `app/src-tauri/src/main.rs`, `app/src-tauri/src/lib.rs` з п'ятьма `auth_*`
  Tauri-командами;
- `app/src-tauri/src/auth/` (Auth Module MLMaiL, 12 файлів);
- `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/`
  (Kotlin Auth Plugin: 4 файли);
- `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json`;
- `app/src-tauri/Cargo.toml`, `app/src-tauri/build.rs`;
- `app/src-tauri/.env.example`.

Не реалізовано (planned):

- `app/src/services/gmail-client.js`, `app/src/services/notes-bridge.js`;
- `app/src/services/summary.js`, `app/src/services/speech.js`;
- `app/src/views/Inbox.vue`, `app/src/views/Reader.vue`;
- `app/src-tauri/src/notes.rs` та реєстрація `save_note`, `list_notes` у
  `tauri::generate_handler![…]`.

## Тести рівня Code MLMaiL

Юніт-тестів MLMaiL поки немає у репозиторії. Цільові команди MLMaiL після
реалізації мінімальної функціональності:

- `bun run test` (Vitest) — для Vue-компонентів і JS-сервісів MLMaiL;
- `cargo test --manifest-path app/src-tauri/Cargo.toml` — для Rust-команд MLMaiL.

Це **прогалина** — заповнюємо одночасно з відповідними файлами MLMaiL.
