# Зведення ADR-впливів на C4-модель MLMaiL

Цей файл зводить **архітектурні рішення MLMaiL**, які формують поточну
C4-модель проєкту MLMaiL, і фіксує, на які рівні моделі вони впливають
([01-context.md](01-context.md), [02-containers.md](02-containers.md),
[03-components.md](03-components.md), [04-code.md](04-code.md)).

Формальні ADR живуть у `docs/adr/`. Поточний стан: тека `docs/adr/_inbox/`
існує, проте формальних ADR-документів **ще немає** — список нижче описує
рішення, **які вже закодовані** у репозиторії MLMaiL, і **рішення, які
очікують ADR**.

## Прийняті рішення MLMaiL (закодовані у репозиторії)

### Рішення: Tauri 2 як рамка крос-платформеного застосунку MLMaiL

Закодовано у [app/src-tauri/Cargo.toml](../../app/src-tauri/Cargo.toml)
(`tauri = "2"`, `tauri-build = "2"`) і
[app/src-tauri/tauri.conf.json](../../app/src-tauri/tauri.conf.json)
(`$schema: https://schema.tauri.app/config/2`).

Вплив на C4-модель MLMaiL:

- [02-containers.md](02-containers.md) — описує контейнер MLMaiL Backend саме
  як Tauri 2 + Rust і контейнер WebView (WKWebView / Android System WebView)
  як рендер контейнера MLMaiL Frontend.
- [03-components.md](03-components.md) і [04-code.md](04-code.md) —
  передбачають IPC через `@tauri-apps/api/core invoke()` як єдиний міст між
  Vue-кодом MLMaiL і Rust-кодом MLMaiL.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0001-tauri-2.md`.

### Рішення: Vue 3 + Vite + auto-import + layouts як фронтенд MLMaiL

Закодовано у [app/package.json](../../app/package.json) (`vue: ^3`,
`vite: ^8`, `unplugin-auto-import`, `vite-plugin-vue-layouts-next`,
`vue-macros`) і [app/vite.config.js](../../app/vite.config.js).

Вплив на C4-модель MLMaiL:

- [02-containers.md](02-containers.md) — контейнер MLMaiL Frontend описаний
  саме як Vue 3 + Vite SPA;
- [03-components.md](03-components.md) — Auth Component MLMaiL, Inbox List
  Component MLMaiL і Mail Reader Component MLMaiL передбачені як
  layout/route-driven Vue-компоненти.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0002-vue3-vite-stack.md`.

### Рішення: Bun monorepo з єдиним workspace `app`

Закодовано у кореневому [package.json](../../package.json)
(`workspaces: ["app"]`), [bunfig.toml](../../bunfig.toml)
(`linker = "hoisted"`), `bun.lock` і обмеженням «у `devDependencies` кореня —
тільки `@nitra/*`». Узгоджено з `.cursor/rules/n-bun.mdc`.

Вплив на C4-модель MLMaiL:

- [02-containers.md](02-containers.md) — підтверджує, що контейнер MLMaiL
  Frontend і контейнер MLMaiL Backend живуть у єдиному монорепо MLMaiL і
  збираються через `bun run` усередині `app/`;
- [04-code.md](04-code.md) — фіксує лінт-pipeline MLMaiL у кореневому
  `package.json`.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0003-bun-monorepo.md`.

### Рішення: цільові платформи MLMaiL — macOS і Android

Закодовано у [app/package.json](../../app/package.json) (скрипт
`android: tauri android dev`), наявності
[app/src-tauri](../../app/src-tauri/) із підтримкою mobile-entry
(`#[cfg_attr(mobile, tauri::mobile_entry_point)]` у
[app/src-tauri/src/lib.rs](../../app/src-tauri/src/lib.rs)) та історії
коміту `android` (e49e5ab `cursor`, 05cf22d `android`).

Вплив на C4-модель MLMaiL:

- [01-context.md](01-context.md) — користувач застосунку MLMaiL описаний як
  такий, що працює з MLMaiL на macOS або Android;
- [02-containers.md](02-containers.md) — контейнерна діаграма MLMaiL із
  двома підграфами (`MLMaiL на macOS` і `MLMaiL на Android`) з тими ж
  іменами логічних контейнерів MLMaiL.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0004-target-platforms.md`.

### Рішення: `tauri-plugin-opener` як офіційний механізм відкриття зовнішніх URL

Закодовано у [app/src-tauri/Cargo.toml](../../app/src-tauri/Cargo.toml)
(`tauri-plugin-opener = "2"`), [app/src-tauri/src/lib.rs](../../app/src-tauri/src/lib.rs)
(`.plugin(tauri_plugin_opener::init())`) і
[app/src-tauri/capabilities/default.json](../../app/src-tauri/capabilities/default.json)
(дозвіл `opener:default`).

Вплив на C4-модель MLMaiL:

- [03-components.md](03-components.md) — Auth Component MLMaiL описаний як
  такий, що відкриває системний браузер MLMaiL через Plugin Opener MLMaiL у
  Authorization Code flow з PKCE.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0005-opener-plugin.md`.

### Рішення: Google OAuth-архітектура MLMaiL

Закодовано у [app/src-tauri/src/auth/](../../app/src-tauri/src/auth/) (Rust),
[app/src/views/Login.vue](../../app/src/views/Login.vue),
[app/src/services/auth-store.js](../../app/src/services/auth-store.js),
[app/src/i18n/auth-errors.js](../../app/src/i18n/auth-errors.js) та Kotlin
plugin `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/`.

Три зв'язаних рішення зафіксовано у єдиному ADR:

1. **OAuth flow** — Authorization Code + PKCE через системний браузер і
   loopback `127.0.0.1` на macOS; Credential Manager + AuthorizationClient
   через Tauri mobile plugin на Android.
2. **Token storage** — platform-specific: Apple Keychain (macOS) і
   EncryptedSharedPreferences (Android). Не Stronghold, не plaintext store.
3. **Token surface** — Rust тримає всі токени; Vue Frontend кличе
   `auth_get_access_token` Tauri command при потребі. Refresh token не
   покидає Rust.

Вплив на C4-модель MLMaiL:

- [01-context.md](01-context.md) — System Context лишається той самий;
- [02-containers.md](02-containers.md) — стрілки HTTPS до Google Identity
  Services переміщено з Frontend на Backend;
- [03-components.md](03-components.md) — Auth Component і Auth Store позначено
  `implemented`, додано Auth Module MLMaiL (Rust) з повним описом
  підкомпонентів і Auth Errors i18n MLMaiL;
- [04-code.md](04-code.md) — додано секції коду для нових файлів,
  оновлено приклад `lib.rs::run()`, видалено згадку про команду `greet`.

ADR оформлений як `docs/adr/ADR-0006-google-oauth.md`.

### Рішення: Кількість листів у скриньці — точне число через `users.labels.get`

Закодовано у [app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/) і
[app/src/services/auth-store.js](../../app/src/services/auth-store.js).

Endpoint `GET users/me/labels/INBOX` повертає точне `messagesTotal` одним
викликом. Альтернативи (`users.getProfile` — рахує всю скриньку, не INBOX;
`messages.list?labelIds=INBOX` — повертає приблизне `resultSizeEstimate`)
відкинуто свідомо. HTTPS-виклик живе у Rust, не у WebView fetch — узгоджено з
ADR-0006 (token surface).

Вплив на C4-модель MLMaiL:

- [03-components.md](03-components.md) — додано Gmail Module MLMaiL (Rust);
  Auth Store MLMaiL отримав `inboxCount`, `inboxErrorKind`, `refreshInboxCount`;
  Auth Errors i18n MLMaiL — девʼять ключів (додано `Http`, `Parse`).
- [04-code.md](04-code.md) — додано секції `app/src-tauri/src/gmail/`,
  оновлено приклад `invoke_handler!` у `lib.rs`, опис `auth-store.js`.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0007-inbox-count.md`.
Чернетка інбоксу — у `docs/adr/_inbox/`.

## Рішення, що очікують ADR для MLMaiL

Нижче перелічено архітектурні питання MLMaiL, які **впливатимуть** на C4-модель
MLMaiL і мають бути зафіксовані ADR до або під час відповідної реалізації. Без
ADR — реалізація **заборонена** (правило Spec-as-Source з
`.cursor/rules/n-ci4.mdc`).

### Очікує ADR: вибір LLM-провайдера для MLMaiL

Кандидати MLMaiL: Anthropic Claude API, OpenAI Chat Completions, локальна
модель через зовнішній runtime. Вибір впливає на:

- [01-context.md](01-context.md) — конкретизація зовнішньої системи
  `LLM-провайдер MLMaiL`;
- [03-components.md](03-components.md) — реалізація Summary Service MLMaiL і
  Reply Drafter Component MLMaiL;
- безпеку токенів і місце зберігання API-ключа MLMaiL (див. наступний пункт).

### Очікує ADR: де живе API-ключ LLM/TTS у MLMaiL і хто робить HTTPS-виклики

Для Google Identity Services рішення зафіксовано ADR-0006 (Rust робить
token exchange; токени не покидають Backend). Питання залишається відкритим
для **LLM/TTS-провайдерів**:

- Два варіанти: HTTPS прямо з MLMaiL Frontend (WebView) — або проксі через
  Backend.
- Вибір впливає на [02-containers.md](02-containers.md) (напрямок стрілок) і
  [03-components.md](03-components.md) (структура Summary Service MLMaiL,
  Speech Service MLMaiL).

### Очікує ADR: вибір TTS-провайдера для MLMaiL

Кандидати MLMaiL: браузерний `SpeechSynthesis` API, хмарні TTS (Google Cloud
TTS, ElevenLabs), локальна модель. Вибір впливає на:

- [01-context.md](01-context.md) — конкретизація зовнішньої системи
  `TTS-провайдер MLMaiL`;
- [03-components.md](03-components.md) — реалізацію Speech Service MLMaiL.

### Очікує ADR: схема Markdown-замітки MLMaiL у `notes/work/` і `notes/home/`

MLMaiL пише `.md`-замітку на лист (frontmatter + тіло). Вибір схеми впливає на:

- [02-containers.md](02-containers.md) — формат контейнера Локальне сховище
  MLMaiL;
- [04-code.md](04-code.md) — сигнатуру `GmailMessage`/`NotePath` у Notes
  Commands MLMaiL.

### Очікує ADR: Content Security Policy для MLMaiL у `tauri.conf.json`

Зараз `app.security.csp: null` у
[app/src-tauri/tauri.conf.json](../../app/src-tauri/tauri.conf.json). Для
бойової версії MLMaiL CSP має бути заданий явно — це обмежить, до яких
зовнішніх доменів MLMaiL фронтенд може робити прямі HTTPS-запити.

<!-- "збереження токенів" закрите ADR-0006: Keychain (macOS) +
EncryptedSharedPreferences (Android), не Stronghold. -->

## Правило синхронізації MLMaiL

Будь-яка зміна, що **впливає** на список вище — нова прийнята архітектурна
опція, новий ADR у `docs/adr/`, новий зовнішній сервіс MLMaiL — оновлює:

1. цей файл (`docs/ci4/decisions.md`);
2. відповідний рівень C4-моделі MLMaiL;
3. (для прийнятих ADR) тіло самого ADR з явним переліком C4-схем, які потрібно
   оновити (правило `.cursor/rules/n-adr.mdc`, секція «Зв'язок із ADR»).

Усе у тому ж PR — розсинхрон між кодом, ADR і C4-моделлю MLMaiL заборонено.
