# ADR-0006: Google OAuth-авторизація MLMaiL

**Дата:** 2026-05-11
**Статус:** Прийнято
**Заміняє:** —
**Заміщено:** —

## Контекст

MLMaiL — крос-платформений Tauri 2 + Vue 3 застосунок (macOS desktop і
Android), що читає Gmail користувача (`gmail.modify` scope) і працює з листами
через AI. Стартовий каркас репозиторію MLMaiL не мав жодної авторизаційної
поверхні — лише демо-команду `greet`. Без OAuth-логіну MLMaiL не може
функціонувати, тому це **перша бойова ітерація** проєкту.

C4-модель MLMaiL у [docs/ci4/](../ci4/) вже описувала цільову архітектуру
авторизації, але залишала три **відкритих ADR-питання**:

- Який саме механізм OAuth flow використати на кожній платформі.
- Де живе сховище refresh token (keychain / EncryptedSharedPreferences /
  Stronghold / інше).
- Хто робить HTTPS-виклики до Google Identity Services — Frontend (WebView)
  чи Backend (Rust).

Ці питання тісно пов'язані: вибір механізму flow визначає, хто отримує
authorization code, що своєю чергою визначає, хто може зробити token
exchange. Тому всі три рішення зафіксовано **одним ADR**.

## Рішення

### 1. OAuth flow — platform-native

| Платформа | Механізм |
| --------- | -------- |
| macOS | Authorization Code + PKCE через системний браузер + Rust loopback HTTP-server на `127.0.0.1:RANDOM_PORT` |
| Android | Credential Manager (sign-in → ID token) + Google Identity AuthorizationClient (scope `gmail.modify` → server auth code) через Tauri 2 mobile plugin (Kotlin) |

Реалізація — [app/src-tauri/src/auth/flow/macos.rs](../../app/src-tauri/src/auth/flow/macos.rs)
та [app/src-tauri/src/auth/flow/android.rs](../../app/src-tauri/src/auth/flow/android.rs)

- Kotlin модуль
[app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/](../../app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/).

### 2. Token storage — platform-specific

| Платформа | Сховище |
| --------- | ------- |
| macOS | Apple Keychain через Rust crate `keyring` (`apple-native` feature, Security framework) |
| Android | EncryptedSharedPreferences з master key у Android Keystore (через Kotlin `MlmailAuthPlugin.saveSession/loadSession/clearSession`) |

Реалізація — [app/src-tauri/src/auth/storage/macos.rs](../../app/src-tauri/src/auth/storage/macos.rs)
та [app/src-tauri/src/auth/storage/android.rs](../../app/src-tauri/src/auth/storage/android.rs)

- Kotlin `SecureStore.kt`.

### 3. Token surface — Rust holds all tokens

- Rust зберігає `refresh_token` у platform storage і `access_token` у пам'яті
  процесу (`AuthState`).
- Vue Frontend кличе `auth_get_access_token` Tauri command перед кожним
  Gmail-запитом. Refresh-логіка живе в Rust і прозоро рефрешить access_token
  через refresh_token при потребі.
- Vue Auth Store зберігає лише `email`, `isAuthenticated`, `isLoading`,
  `errorKind` — жодних токенів у JS-пам'яті.

Реалізація — [app/src-tauri/src/auth/mod.rs](../../app/src-tauri/src/auth/mod.rs)
та [app/src/services/auth-store.js](../../app/src/services/auth-store.js).

## Обґрунтування

**Чому native flow на Android, а не AppAuth / Custom Tabs:** Credential
Manager — рекомендований Google інтерфейс для нативних застосунків на Android
14+. Він дає native account picker з облікових записів пристрою, без web-форми
Google логіну при наявності збереженого акаунту. AuthorizationClient
домальовує consent для додаткових scopes (`gmail.modify`) — і повертає
**серверний auth code**, який Rust обмінює на access/refresh tokens. Це той
самий tokens-endpoint, що й на macOS — отже логіка обміну спільна.

**Чому Keychain / EncryptedSharedPreferences, а не Stronghold:** Stronghold —
encrypted vault від IOTA, який тримає секрети у файлі і вимагає
розблокування master password або стороннім ключем (який однаково треба десь
зберегти). Це додатковий рівень над system keychain без виграшу в безпеці для
нашого випадку. Keychain і EncryptedSharedPreferences — system-managed,
hardware-backed (на сучасних пристроях), без UX-кроку master password.

**Чому Rust тримає токени, а не Vue:** access token у JS-пам'яті залишається
доступним для XSS-атак, якщо колись скомпрометується якась dependency. Rust
не має такої поверхні. Vue знає лише email і authenticated-флаг — цього
достатньо для UI, не достатньо для крадіжки доступу до Gmail.

**Чому HTTPS до `oauth2.googleapis.com/token` робить Rust:** authorization
code потрапляє в Rust (через loopback на macOS, через JNI з Kotlin на
Android), і логічно завершити обмін там само. Це також гарантує, що client
secret (не потрібен для PKCE, але може знадобитися для майбутніх scope) і
refresh token ніколи не покидають Rust.

## Розглянуті альтернативи

1. **AppAuth-Android** (OpenID Foundation). Дає access/refresh tokens
   напряму через Chrome Custom Tabs з PKCE. Відкинуто на користь Credential
   Manager — native UX на Android краще за вбудований браузер для повторних
   логінів. Якби Credential Manager виявився проблемним, AppAuth-Android — це
   найбільш природний fallback (мінімальні зміни Rust-коду).
2. **GoogleSignIn** (`com.google.android.gms:play-services-auth`
   `GoogleSignInOptions`). Старе API, deprecated Google у 2024. Відкинуто.
3. **Stronghold** (`tauri-plugin-stronghold`). Кросплатформений encrypted
   vault. Відкинуто — зайвий рівень абстракції над уже наявними
   платформенними сховищами.
4. **tauri-plugin-store з шифруванням** або **plaintext store** для refresh
   token. Відкинуто — refresh token чутливий, JSON-файл у `app_data_dir`
   надто слабка ізоляція.
5. **Token у JS-пам'яті** (Vue Auth Store тримає access token). Відкинуто
   через XSS-поверхню; деталі вище.
6. **Rust як проксі для всіх Gmail HTTPS-викликів.** Це найбезпечніший
   варіант, але виходить за межі цієї ітерації (Gmail Client MLMaiL ще не
   реалізований). Архітектура з `auth_get_access_token` сумісна з обома
   варіантами — Gmail Client можна додати як JS-side або як Rust-проксі без
   зміни auth-поверхні.

## Зачіпає

Документація C4-моделі MLMaiL:

- [docs/ci4/02-containers.md](../ci4/02-containers.md) — стрілки HTTPS до
  Google Identity Services переміщено з Frontend на Backend; оновлено опис
  обов'язків Backend.
- [docs/ci4/03-components.md](../ci4/03-components.md) — Auth Component і
  Auth Store позначено `implemented`; додано Auth Module MLMaiL (Rust) і
  Auth Errors i18n MLMaiL.
- [docs/ci4/04-code.md](../ci4/04-code.md) — додано секції коду для нових
  файлів; видалено команду `greet`.
- [docs/ci4/decisions.md](../ci4/decisions.md) — закрито питання збереження
  токенів; уточнено межі питання HTTPS-викликів для Google (закрите) vs
  LLM/TTS (відкрите).

Код:

- Новий каталог `app/src-tauri/src/auth/` (12 файлів, 32 unit-тести).
- Новий каталог
  `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/`
  (4 Kotlin-файли).
- Нові `app/src/views/Login.vue`, `app/src/services/auth-store.js`,
  `app/src/i18n/auth-errors.js` + 22 unit-тести (vitest +
  `@vue/test-utils`).
- Змінено `app/src-tauri/Cargo.toml`, `app/src-tauri/src/lib.rs`,
  `app/src-tauri/src/main.rs`, `app/src/App.vue`, `app/package.json`,
  `app/vite.config.js`,
  `app/src-tauri/gen/android/app/build.gradle.kts`,
  `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/MainActivity.kt`,
  `.cspell.json`, `.gitignore`.
- Новий `app/src-tauri/.env.example` (`MLMAIL_GOOGLE_*` client IDs).

## Відомі обмеження і ризики

1. **Tauri 2 mobile plugin registration API** — точний шлях передачі
   `PluginHandle` з `tauri::Builder::setup` у Tauri-managed state ще потребує
   уточнення під час першої Android збірки. Поки що `flow/android.rs` чекає
   `PluginHandle` у `tauri::State<Mutex<Option<PluginHandle<Wry>>>>` — це
   треба підключити в `lib.rs::run` для Android-target.
2. **Google app verification для `gmail.modify`** не пройдена — поки що
   OAuth consent screen у режимі `Testing`, кожен користувач має бути доданий
   у Test users.
3. **Debug SHA-1 для Android OAuth client** має додати розробник вручну у
   Google Cloud Console перед першим запуском на Android. Інакше
   AuthorizationClient падатиме з `DEVELOPER_ERROR`.
4. **PKCE не використовується на Android** — це обмеження AuthorizationClient
   API. Безпеку забезпечує SHA-1 fingerprint APK у Cloud Console (Android
   OAuth client type).
5. **macOS Keychain access** показує системний prompt при першому збереженні
   — це очікуваний UX.
6. **`security-crypto:1.1.0-alpha06`** на Android — alpha-версія. Якщо
   виявить проблеми у білді, fallback до `1.0.0` (старіший API, але stable).
7. **Loopback redirect URI** на macOS — Google автоматично приймає
   `http://127.0.0.1` будь-який порт для Desktop OAuth client type.

## Як перевірити

Rust:

```sh
cd app/src-tauri && cargo test --lib auth
```

Очікувано: 32 passed.

JavaScript:

```sh
cd app && bun run test
```

Очікувано: 22 passed (8 auth-errors + 9 auth-store + 5 Login).

Manual end-to-end (потребує налаштованих Google Cloud OAuth clients і
`.env`):

- macOS: `bun run tauri dev` → "Увійти через Google" → системний браузер →
  consent → callback → "Ви увійшли як {email}" → перезапуск → стан логіну
  відновлено → "Вийти" → знову Login.
- Android: `bun run android` на емуляторі з Google account → "Увійти через
  Google" → Credential Manager bottom sheet → consent на gmail.modify → email
  у Login → перезапуск → стан логіну відновлено → "Вийти" → знову Login.

Деталі — [docs/superpowers/specs/2026-05-11-google-oauth-design.md](../superpowers/specs/2026-05-11-google-oauth-design.md).
