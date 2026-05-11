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

Кореневий компонент App Shell MLMaiL. Стартовий вміст — демо-форма для виклику
Tauri-команди `greet`:

```vue
<script setup>
import { invoke } from '@tauri-apps/api/core'

const greetMsg = ref('')
const name = ref('')

async function greet() {
  greetMsg.value = await invoke('greet', { name: name.value })
}
</script>
```

Демо-код буде **замінений** на корінь маршрутизатора при першій бойовій
ітерації MLMaiL (видалити шаблонні стилі, логотипи, форму `greet`).

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
`unplugin-auto-import`, `vite`, `vite-plugin-vue-layouts-next`, `vue-macros`.

### Файл [app/jsconfig.json](../../app/jsconfig.json)

JS-конфіг IDE для MLMaiL Frontend (path-aliases, `module: ESNext`). Уплив на
рантайм MLMaiL відсутній — лише підказки редактора.

### Планова сигнатура: Gmail Client MLMaiL

```js
// app/src/services/gmail-client.js — planned
export async function listInbox(maxResults = 25) { /* ... */ }
export async function getMessage(id) { /* ... */ }
export async function trashMessage(id) { /* ... */ }
export async function createFilter(criteria, action) { /* ... */ }
export async function createDraft(message) { /* ... */ }
```

Файл `app/src/services/gmail-client.js` ще не існує (planned). Сигнатури вище —
**договір**, від якого має відштовхуватися реалізація і тести MLMaiL.

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
  (з `derive`), `serde_json = "1"`.

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

Реальна точка ініціалізації MLMaiL Backend. Поточний код:

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Точки розширення MLMaiL Backend:

- `invoke_handler` — додавати нові Tauri-команди MLMaiL (`save_note`, `list_notes`);
- `.plugin(...)` — додавати нові Tauri-плагіни MLMaiL (наприклад, плагін для
  захищеного сховища токенів при бойовій реалізації Auth Store MLMaiL);
- `#[cfg_attr(mobile, tauri::mobile_entry_point)]` — та сама функція `run()`
  є точкою входу і для Android-збірки MLMaiL, окремий `main.rs` для mobile
  **не потрібен**.

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

- `app/src/main.js`, `app/src/App.vue`, `app/index.html`, `app/vite.config.js`;
- `app/src-tauri/src/main.rs`, `app/src-tauri/src/lib.rs` з командою `greet`;
- `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json`;
- `app/src-tauri/Cargo.toml`, `app/src-tauri/build.rs`.

Не реалізовано (planned):

- `app/src/services/gmail-client.js`, `app/src/services/notes-bridge.js`;
- `app/src/services/summary.js`, `app/src/services/speech.js`;
- `app/src/views/Inbox.vue`, `app/src/views/Reader.vue`, `app/src/views/Login.vue`;
- `app/src-tauri/src/notes.rs` та реєстрація `save_note`, `list_notes` у
  `tauri::generate_handler![…]`.

## Тести рівня Code MLMaiL

Юніт-тестів MLMaiL поки немає у репозиторії. Цільові команди MLMaiL після
реалізації мінімальної функціональності:

- `bun run test` (Vitest) — для Vue-компонентів і JS-сервісів MLMaiL;
- `cargo test --manifest-path app/src-tauri/Cargo.toml` — для Rust-команд MLMaiL.

Це **прогалина** — заповнюємо одночасно з відповідними файлами MLMaiL.
