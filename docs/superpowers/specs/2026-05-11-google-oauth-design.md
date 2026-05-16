# Google OAuth-авторизація для MLMaiL — design spec

**Дата:** 2026-05-11
**Статус:** Approved, готовий до плану імплементації
**Scope:** Лише OAuth flow + безпечне збереження токенів. Без Gmail Client, без Inbox UI, без router/layouts.

## Мета

Реалізувати наскрізну Google-авторизацію для застосунку MLMaiL на двох цільових платформах (macOS desktop і Android) з такою поведінкою:

1. Користувач MLMaiL натискає "Увійти через Google" → проходить OAuth 2.0 flow → бачить "Ви увійшли як {email}".
2. `refresh_token` зберігається у захищеному сховищі пристрою (Keychain на macOS, EncryptedSharedPreferences на Android).
3. `access_token` живе у пам'яті Rust-процесу і автоматично рефрешиться при потребі.
4. Перезапуск застосунку відновлює стан логіну без додаткових кроків від користувача.
5. Vue Frontend отримує доступ до `access_token` лише через Tauri-команду `auth_get_access_token` — токени не зберігаються у JS-пам'яті.
6. Користувач може вийти з акаунту (`Logout` — очищає сховище і in-memory state).

Що **поза scope** цієї ітерації:

- Реальні виклики до Gmail REST API (Gmail Client MLMaiL — окрема ітерація).
- Inbox List Component, Mail Reader, Action Bar, Reply Drafter — окремі ітерації.
- Router / layouts — `App.vue` показує `<Login/>` напряму.
- CSP у `tauri.conf.json` — окремий ADR.
- Release signing key для Android — фіксується при першому Play-релізі.
- Multi-account — один акаунт за раз; повторний login переписує існуючий refresh token.
- Automatic e2e — лише manual checklists.

## Ключові архітектурні рішення

1. **OAuth механіка — platform-native:**
   - **macOS:** Authorization Code + PKCE через системний браузер + loopback HTTP server у Rust (`tokio::net::TcpListener` на `127.0.0.1:0`).
   - **Android:** Credential Manager (sign-in → ID token) + Google Identity AuthorizationClient (scope `gmail.modify` → server auth code) через JNI-міст з Rust до Kotlin.
2. **Token storage — platform-specific:**
   - **macOS:** Apple Keychain через Rust crate `keyring`.
   - **Android:** EncryptedSharedPreferences через JNI до того ж Kotlin-модуля, що обслуговує auth flow.
3. **Token surface:** Rust — єдиний holder токенів. Frontend кличе `auth_get_access_token` Tauri command при потребі. Refresh token ніколи не покидає Rust.
4. **HTTPS до Google `oauth2.googleapis.com/token`** робить Rust через `reqwest` з `rustls-tls`.
5. **UI MLMaiL — українською мовою.** Rust повертає англомовний `error.kind`, Vue мапить його через i18n-таблицю на українські рядки.

## Архітектура

### Нові компоненти

| Шар                    | Файл / шлях                                                                                              | Призначення                                                                                                        |
| ---------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Vue Frontend           | `app/src/views/Login.vue`                                                                                | Login екран MLMaiL — кнопка "Увійти через Google", email + "Вийти" коли залогінено, повідомлення про помилку       |
| Vue Frontend           | `app/src/services/auth-store.js`                                                                         | Composable-фасад над `auth_*` Tauri-командами. Реактивні поля `email`, `isAuthenticated`, `isLoading`, `errorKind` |
| Vue Frontend           | `app/src/i18n/auth-errors.js`                                                                            | Локалізація `kind` → українська строка                                                                             |
| Rust Backend           | `app/src-tauri/src/auth/mod.rs`                                                                          | Публічні Tauri-команди `auth_*`, реєстрація                                                                        |
| Rust Backend           | `app/src-tauri/src/auth/state.rs`                                                                        | `AuthState` (in-memory access token + email + expiry)                                                              |
| Rust Backend           | `app/src-tauri/src/auth/pkce.rs`                                                                         | Генерація `code_verifier` і `code_challenge`                                                                       |
| Rust Backend           | `app/src-tauri/src/auth/token_exchange.rs`                                                               | HTTPS POST до Google token endpoint                                                                                |
| Rust Backend           | `app/src-tauri/src/auth/id_token.rs`                                                                     | Парсинг email з JWT payload                                                                                        |
| Rust Backend           | `app/src-tauri/src/auth/config.rs`                                                                       | OAuth client IDs (з `option_env!`)                                                                                 |
| Rust Backend           | `app/src-tauri/src/auth/storage/mod.rs`                                                                  | Trait `RefreshTokenStorage`, factory                                                                               |
| Rust Backend (macOS)   | `app/src-tauri/src/auth/storage/macos.rs`                                                                | Keychain через `keyring` crate                                                                                     |
| Rust Backend (Android) | `app/src-tauri/src/auth/storage/android.rs`                                                              | JNI до `MlmailAuth.saveSession/loadSession/clearSession`                                                           |
| Rust Backend (macOS)   | `app/src-tauri/src/auth/flow/macos.rs`                                                                   | Loopback + системний браузер + token exchange                                                                      |
| Rust Backend (Android) | `app/src-tauri/src/auth/flow/android.rs`                                                                 | JNI до `MlmailAuth.signInAndAuthorize` + token exchange                                                            |
| Kotlin (Android)       | `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/auth/MlmailAuth.kt`                    | Public JNI entry points                                                                                            |
| Kotlin (Android)       | `…/auth/CredentialManagerFlow.kt`                                                                        | Sign-in етап (ID token)                                                                                            |
| Kotlin (Android)       | `…/auth/AuthorizationFlow.kt`                                                                            | Scope authorization (server auth code)                                                                             |
| Kotlin (Android)       | `…/auth/SecureStore.kt`                                                                                  | EncryptedSharedPreferences wrapper                                                                                 |
| Тести Rust             | `app/src-tauri/src/auth/*::tests`                                                                        | Unit-тести (PKCE, ID token, token exchange з `mockito`, state, in-memory storage)                                  |
| Тести JS               | `app/src/services/auth-store.test.js`, `app/src/views/Login.test.js`, `app/src/i18n/auth-errors.test.js` | Vitest + `@vue/test-utils`                                                                                         |

### Зміни до існуючих файлів

| Файл                                             | Зміна                                                                                                                                                     |
| ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `app/src-tauri/src/lib.rs`                       | Видалити команду `greet`; реєстрація нових `auth_*` команд; `.manage(Mutex::new(AuthState::default()))`; `.setup(\|app\| auth::on_startup(app.handle()))` |
| `app/src/App.vue`                                | Замінити демо-форму `greet` на `<Login/>`. Видалити пов'язані стилі логотипів і input/button styles, що належали тільки демці.                            |
| `app/src-tauri/Cargo.toml`                       | Нові залежності (нижче)                                                                                                                                   |
| `app/src-tauri/gen/android/app/build.gradle.kts` | Нові Android dependencies (нижче)                                                                                                                         |
| `app/package.json`                               | Додати `vitest`, `@vue/test-utils`, `jsdom` у devDependencies; новий скрипт `test`                                                                        |
| `.cspell.json`                                   | Додати нові англомовні технічні терміни (нижче)                                                                                                           |
| `docs/ci4/02-containers.md`                      | Стрілки HTTPS до Google Identity Services — від Backend, не Frontend                                                                                      |
| `docs/ci4/03-components.md`                      | Auth Component / Auth Store — `implemented`, додати Auth Module (Rust) з підкомпонентами                                                                  |
| `docs/ci4/04-code.md`                            | Нова секція code для `app/src-tauri/src/auth/*`; оновлений Backend Entry lib без `greet`                                                                  |
| `docs/ci4/decisions.md`                          | Закрити три відкритих ADR-питання (token storage, HTTPS-виклики, частина secrets), додати посилання на ADR-0006                                           |
| `docs/adr/ADR-0006-google-oauth.md`              | Новий ADR — три тісно пов'язаних рішення (flow, storage, token surface)                                                                                   |

## Потоки даних

### Login flow (sequence)

1. Користувач натискає **"Увійти через Google"** у `Login.vue`.
2. `Login.vue` → `auth.login()` → `invoke('auth_start_login')`.
3. Rust `auth_start_login`:
   - Генерує PKCE pair (на macOS; на Android PKCE не використовується — див. примітку нижче).
   - **macOS:** `tokio::net::TcpListener::bind("127.0.0.1:0")` → отримує `port` → формує `redirect_uri = http://127.0.0.1:{port}/callback` → будує auth URL → `app.opener().open_url(...)` → чекає `GET /callback?code=…&state=…` з таймаутом 5 хв → перевіряє CSRF state.
   - **Android:** JNI виклик `MlmailAuth.signInAndAuthorize(activity, webClientId, scopes)` → Kotlin запускає Credential Manager → отримує `id_token` (GoogleIdTokenCredential) → запускає `AuthorizationClient.authorize(request)` з `requestOfflineAccess(webClientId, forceCodeForRefreshToken=true)` → отримує `server_auth_code` → повертає JSON-стрічку у Rust.
4. Rust викликає `token_exchange::exchange_code(...)`:
   - **macOS:** POST `https://oauth2.googleapis.com/token` з `client_id` (Desktop), `code`, `code_verifier`, `redirect_uri`, `grant_type=authorization_code`.
   - **Android:** POST з `client_id` (Android), `code`, `redirect_uri=""` (для Android-типу client це OK), `grant_type=authorization_code`. **PKCE не використовується** — Google автентифікує APK через SHA-1 fingerprint.
5. Rust парсить `access_token`, `expires_in`, `refresh_token`, `id_token`.
6. `id_token::extract_email(...)` декодує JWT payload (без верифікації підпису — токен прийшов прямо з Google по HTTPS) і витягує `email`.
7. Rust зберігає `email` + `refresh_token` через `RefreshTokenStorage::save(...)`.
8. Rust оновлює `AuthState { email, access_token, access_token_expires_at }`.
9. Tauri команда повертає `Ok(AuthSession { email })`.
10. Auth Store оновлює `email`, `isAuthenticated = true`. `Login.vue` показує "Ви увійшли як {email}".

### Refresh flow (sequence)

1. Будь-який майбутній код кличе `invoke('auth_get_access_token')`.
2. Rust `auth_get_access_token`:
   - Якщо `AuthState.is_access_token_fresh()` (експірація >30 секунд) — повертає кешований.
   - Інакше: завантажує `refresh_token` зі сховища → `token_exchange::exchange_refresh(client_id, refresh_token)` → POST до `/token` з `grant_type=refresh_token` → оновлює `AuthState` → повертає новий access_token.
   - Якщо відповідь містить новий `refresh_token` (Google іноді ротує його при чутливих scope) — Rust перезаписує `refresh_token` у сховищі (`storage.save(email, new_refresh_token)`). Інакше старий лишається.
3. Якщо `exchange_refresh` повертає HTTP 400 з `invalid_grant`:
   - Rust очищає сховище (`storage.clear()`).
   - Скидає `AuthState`.
   - Повертає `Err(AuthError::ReauthRequired)`.
   - Frontend ловить помилку, скидає Auth Store, показує Login екран.

### Cold start (app re-open)

1. У `lib.rs::run()`, всередині `.setup(|app| ...)` → `auth::on_startup(app.handle())`.
2. `on_startup` зчитує `StoredSession` через `storage.load()`.
3. Якщо є — `AuthState = { email: Some(...), access_token: None, access_token_expires_at: None }`.
4. Frontend на старті: `Login.vue::onMounted` → `auth.initialize()` → `invoke('auth_is_authenticated')` → `true`.
5. `invoke('auth_current_email')` повертає збережений email — Login показує "Ви увійшли як {email}".
6. Перший виклик `auth_get_access_token` тригерить refresh.

### Logout

1. Користувач натискає "Вийти".
2. `auth.logout()` → `invoke('auth_logout')`.
3. Rust: `storage.clear()` + скидає `AuthState`.
4. Frontend: скидає Auth Store. Login показує "Увійти через Google".

## Tauri-команди (API surface)

```rust
#[tauri::command]
async fn auth_start_login(app: AppHandle, state: State<'_, Mutex<AuthState>>)
    -> Result<AuthSession, AuthError>;

#[tauri::command]
async fn auth_get_access_token(state: State<'_, Mutex<AuthState>>)
    -> Result<String, AuthError>;

#[tauri::command]
fn auth_is_authenticated(state: State<'_, Mutex<AuthState>>) -> bool;

#[tauri::command]
async fn auth_current_email(state: State<'_, Mutex<AuthState>>) -> Option<String>;

#[tauri::command]
async fn auth_logout(state: State<'_, Mutex<AuthState>>) -> Result<(), AuthError>;
```

**DTOs:**

```rust
#[derive(Serialize)]
pub struct AuthSession { pub email: String }

#[derive(Serialize, thiserror::Error, Debug)]
#[serde(tag = "kind", content = "message")]
pub enum AuthError {
    #[error("login cancelled by user")] Cancelled,
    #[error("network error: {0}")]     Network(String),
    #[error("OAuth error: {0}")]       OAuth(String),
    #[error("storage error: {0}")]     Storage(String),
    #[error("re-authentication required")] ReauthRequired,
    #[error("platform error: {0}")]    Platform(String),
}
```

Серіалізація: `{ "kind": "Network", "message": "timeout" }` — Vue розрізняє типи через `kind`, повідомлення з `message` йде у лог, не у UI.

## Auth Store і UI (JavaScript)

### `app/src/services/auth-store.js` (singleton-composable)

```js
import { invoke } from '@tauri-apps/api/core'
import { ref, readonly } from 'vue'

const _email = ref(null)
const _isAuthenticated = ref(false)
const _isLoading = ref(false)
const _errorKind = ref(null)

/**
 *
 */
export function useAuthStore() {
  /**
   *
   */
  async function initialize() {
    _isAuthenticated.value = await invoke('auth_is_authenticated')
    if (_isAuthenticated.value) {
      _email.value = await invoke('auth_current_email')
    }
  }

  /**
   *
   */
  async function login() {
    _isLoading.value = true
    _errorKind.value = null
    try {
      const session = await invoke('auth_start_login')
      _email.value = session.email
      _isAuthenticated.value = true
    } catch (error) {
      _errorKind.value = error?.kind ?? 'Unknown'
    } finally {
      _isLoading.value = false
    }
  }

  /**
   *
   */
  async function getAccessToken() {
    return invoke('auth_get_access_token')
  }

  /**
   *
   */
  async function logout() {
    await invoke('auth_logout')
    _email.value = null
    _isAuthenticated.value = false
  }

  return {
    email: readonly(_email),
    isAuthenticated: readonly(_isAuthenticated),
    isLoading: readonly(_isLoading),
    errorKind: readonly(_errorKind),
    initialize,
    login,
    getAccessToken,
    logout
  }
}
```

Один модульний state — будь-який компонент MLMaiL отримує той самий `useAuthStore()`.

### `app/src/i18n/auth-errors.js`

```js
const messages = {
  Cancelled: 'Логін скасовано.',
  Network: "Не вдалося з'єднатися з Google. Перевірте мережу.",
  OAuth: 'Помилка авторизації Google.',
  Storage: 'Не вдалося зберегти токен у захищене сховище пристрою.',
  ReauthRequired: 'Сеанс прострочений — увійдіть знову.',
  Platform: 'Помилка платформи.',
  Unknown: 'Невідома помилка.'
}

/**
 *
 * @param kind
 */
export function errorMessage(kind) {
  return messages[kind] ?? messages.Unknown
}
```

Без vue-i18n у цій ітерації — простий switch достатній для семи рядків.

### `app/src/views/Login.vue`

```vue
<script setup>
import { onMounted } from 'vue'
import { useAuthStore } from '../services/auth-store.js'
import { errorMessage } from '../i18n/auth-errors.js'

const auth = useAuthStore()
onMounted(() => auth.initialize())
</script>

<template>
  <main>
    <h1>MLMaiL</h1>
    <div v-if="auth.isAuthenticated.value">
      <p>Ви увійшли як {{ auth.email }}</p>
      <button @click="auth.logout()">Вийти</button>
    </div>
    <button v-else :disabled="auth.isLoading.value" @click="auth.login()">
      {{ auth.isLoading.value ? 'Зачекайте…' : 'Увійти через Google' }}
    </button>
    <p v-if="auth.errorKind.value" class="error">
      {{ errorMessage(auth.errorKind.value) }}
    </p>
  </main>
</template>
```

### `app/src/App.vue`

Спрощується до:

```vue
<script setup>
import Login from './views/Login.vue'
</script>

<template>
  <Login />
</template>
```

Всі демо-стилі і логотипи з шаблону Tauri-Vue видаляються разом з командою `greet`.

## Rust core

### `auth/state.rs`

```rust
use std::time::Instant;

#[derive(Default)]
pub struct AuthState {
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<Instant>,
}

impl AuthState {
    pub fn is_access_token_fresh(&self) -> bool {
        match (&self.access_token, self.access_token_expires_at) {
            (Some(_), Some(exp)) => exp.saturating_duration_since(Instant::now()).as_secs() > 30,
            _ => false,
        }
    }
}
```

30-секундний буфер — щоб запит, що щойно отримав токен, не несвіжав посеред HTTP-виклику до Gmail.

### `auth/pkce.rs`

```rust
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct PkcePair { pub verifier: String, pub challenge: String }

pub fn generate() -> PkcePair {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let mut h = Sha256::new();
    h.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(h.finalize());
    PkcePair { verifier, challenge }
}
```

### `auth/token_exchange.rs`

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

pub enum FlowKind { Desktop, Android }

pub async fn exchange_code(
    client_id: &str,
    code: &str,
    code_verifier: &str,    // ігнорується для Android
    redirect_uri: &str,     // "" для Android
    flow: FlowKind,
) -> Result<TokenResponse, AuthError> {
    let mut form: Vec<(&str, &str)> = vec![
        ("client_id", client_id),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];
    if matches!(flow, FlowKind::Desktop) {
        form.push(("code_verifier", code_verifier));
    }

    reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&form)
        .send().await
        .map_err(|e| AuthError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| AuthError::OAuth(e.to_string()))?
        .json().await
        .map_err(|e| AuthError::OAuth(e.to_string()))
}

pub async fn exchange_refresh(
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    let resp = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send().await
        .map_err(|e| AuthError::Network(e.to_string()))?;

    if resp.status() == 400 {
        return Err(AuthError::ReauthRequired);
    }
    resp.error_for_status()
        .map_err(|e| AuthError::OAuth(e.to_string()))?
        .json().await
        .map_err(|e| AuthError::OAuth(e.to_string()))
}
```

### `auth/id_token.rs`

```rust
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

pub fn extract_email(id_token: &str) -> Option<String> {
    let payload_b64 = id_token.split('.').nth(1)?;
    let payload = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    json.get("email")?.as_str().map(|s| s.to_string())
}
```

Без верифікації підпису — токен прийшов прямо з `oauth2.googleapis.com` по HTTPS, і використовується лише для UI display, не для authorization.

### `auth/config.rs`

```rust
pub const DESKTOP_CLIENT_ID: &str = match option_env!("MLMAIL_GOOGLE_DESKTOP_CLIENT_ID") {
    Some(v) => v,
    None => "REPLACE_ME_DESKTOP_CLIENT_ID",
};

pub const ANDROID_CLIENT_ID: &str = match option_env!("MLMAIL_GOOGLE_ANDROID_CLIENT_ID") {
    Some(v) => v,
    None => "REPLACE_ME_ANDROID_CLIENT_ID",
};

pub const ANDROID_WEB_CLIENT_ID: &str = match option_env!("MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID") {
    Some(v) => v,
    None => "REPLACE_ME_ANDROID_WEB_CLIENT_ID",
};
```

Client IDs підтягуються з env vars під час `cargo build`. Локально розробник тримає їх у `app/src-tauri/.env` (у gitignore).

### `auth/storage/mod.rs`

```rust
pub trait RefreshTokenStorage: Send + Sync {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError>;
    fn load(&self) -> Result<Option<StoredSession>, StorageError>;
    fn clear(&self) -> Result<(), StorageError>;
}

pub struct StoredSession {
    pub email: String,
    pub refresh_token: String,
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("backend error: {0}")] Backend(String),
}
```

Реалізації — `macos::Keychain` і `android::EncryptedPrefs`. У тестах підставляється `InMemoryStorage` через trait object.

## Platform: macOS

### Loopback flow `auth/flow/macos.rs`

```rust
pub async fn run_login_flow(
    app: &AppHandle,
    client_id: &str,
) -> Result<TokenResponse, AuthError> {
    let pkce = pkce::generate();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await
        .map_err(|e| AuthError::Platform(format!("bind loopback: {e}")))?;
    let port = listener.local_addr().unwrap().port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let state = generate_random_state();

    let auth_url = build_auth_url(client_id, &redirect_uri, &pkce.challenge, &state);
    app.opener().open_url(&auth_url, None::<&str>)
        .map_err(|e| AuthError::Platform(format!("open browser: {e}")))?;

    let (code, returned_state) = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        wait_for_callback(listener),
    ).await
        .map_err(|_| AuthError::Cancelled)??;

    if returned_state != state {
        return Err(AuthError::OAuth("CSRF state mismatch".into()));
    }

    token_exchange::exchange_code(
        client_id, &code, &pkce.verifier, &redirect_uri, FlowKind::Desktop,
    ).await
}
```

`wait_for_callback` приймає одне TCP-підключення, парсить `GET /callback?code=…&state=…` (regex або simple split), відповідає `HTTP/1.1 200 OK` з HTML `<h1>Готово, можете закрити це вікно.</h1>`, закриває socket.

### Auth URL формат

```
https://accounts.google.com/o/oauth2/v2/auth
  ?client_id={DESKTOP_CLIENT_ID}
  &redirect_uri=http://127.0.0.1:{port}/callback
  &response_type=code
  &scope=openid%20email%20https://www.googleapis.com/auth/gmail.modify
  &code_challenge={pkce.challenge}
  &code_challenge_method=S256
  &state={random}
  &access_type=offline
  &prompt=consent
```

- `openid email` — щоб отримати `id_token` з полем `email`.
- `access_type=offline` — обов'язково, щоб Google видав refresh_token.
- `prompt=consent` — обов'язково при повторному логіні, інакше Google може не повернути refresh_token.

### Keychain `auth/storage/macos.rs`

```rust
use keyring::Entry;

const SERVICE: &str = "com.vitaliytv.mlmail";
const REFRESH_KEY: &str = "google.refresh_token";
const EMAIL_KEY: &str = "google.email";

pub struct Keychain;

impl RefreshTokenStorage for Keychain {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError> {
        Entry::new(SERVICE, EMAIL_KEY)
            .and_then(|e| e.set_password(email))
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Entry::new(SERVICE, REFRESH_KEY)
            .and_then(|e| e.set_password(refresh_token))
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn load(&self) -> Result<Option<StoredSession>, StorageError> {
        let email = match Entry::new(SERVICE, EMAIL_KEY)
            .and_then(|e| e.get_password()) {
            Ok(v) => v,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(StorageError::Backend(e.to_string())),
        };
        let refresh_token = match Entry::new(SERVICE, REFRESH_KEY)
            .and_then(|e| e.get_password()) {
            Ok(v) => v,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(StorageError::Backend(e.to_string())),
        };
        Ok(Some(StoredSession { email, refresh_token }))
    }

    fn clear(&self) -> Result<(), StorageError> {
        let _ = Entry::new(SERVICE, EMAIL_KEY)
            .and_then(|e| e.delete_credential());
        let _ = Entry::new(SERVICE, REFRESH_KEY)
            .and_then(|e| e.delete_credential());
        Ok(())
    }
}
```

### Tauri capabilities — без змін

`opener:default` уже у `capabilities/default.json` — потрібно для відкриття системного браузера. Loopback HTTP server — це звичайний `tokio::net::TcpListener`, не Tauri API.

## Platform: Android

### Kotlin модуль

**`MlmailAuth.kt`** — public JNI entry points (`@JvmStatic` методи):

```kotlin
package com.vitaliytv.mlmail.auth

object MlmailAuth {
    @JvmStatic
    fun signInAndAuthorize(
        activity: android.app.Activity,
        webClientId: String,
        scopes: Array<String>,
    ): String {
        return runBlocking {
            try {
                val cred = CredentialManagerFlow.signIn(activity, webClientId)
                val authz = AuthorizationFlow.authorize(activity, webClientId, scopes)
                """{"server_auth_code":"${authz.serverAuthCode}","id_token":"${cred.idToken}"}"""
            } catch (e: GetCredentialCancellationException) {
                """{"error":"Cancelled"}"""
            } catch (e: Exception) {
                """{"error":"${e.message?.replace("\"","'") ?: "unknown"}"}"""
            }
        }
    }

    @JvmStatic fun saveSession(ctx: Context, email: String, refreshToken: String) =
        SecureStore.save(ctx, email, refreshToken)

    @JvmStatic fun loadSession(ctx: Context): String? = SecureStore.load(ctx)

    @JvmStatic fun clearSession(ctx: Context) = SecureStore.clear(ctx)
}
```

**`CredentialManagerFlow.kt`** — sign-in етап:

```kotlin
suspend fun signIn(activity: Activity, webClientId: String): GoogleIdTokenCredential {
    val request = GetCredentialRequest.Builder()
        .addCredentialOption(
            GetGoogleIdOption.Builder()
                .setServerClientId(webClientId)
                .setFilterByAuthorizedAccounts(false)
                .build()
        ).build()
    val result = CredentialManager.create(activity).getCredential(activity, request)
    return GoogleIdTokenCredential.createFrom(result.credential.data)
}
```

**Критична деталь:** `setServerClientId` приймає **Web** OAuth client ID, не Android (так вимагає Credential Manager API).

**`AuthorizationFlow.kt`** — scope authorization з потенційним consent UI:

```kotlin
suspend fun authorize(
    activity: Activity,
    webClientId: String,
    scopes: Array<String>,
): AuthorizationResult {
    val request = AuthorizationRequest.Builder()
        .setRequestedScopes(scopes.map(::Scope))
        .requestOfflineAccess(webClientId, /*forceCodeForRefreshToken=*/ true)
        .build()

    val client = Identity.getAuthorizationClient(activity)
    val result = client.authorize(request).await()

    if (result.hasResolution()) {
        return launchConsentAndAwait(activity, result.pendingIntent, client)
    }
    return result
}
```

`launchConsentAndAwait` запускає `PendingIntent` через `ActivityResultLauncher` (зареєстрований у `MainActivity` Tauri-generated проєкту), чекає `onActivityResult`, потім `client.getAuthorizationResultFromIntent(data)`. Це **єдиний** runtime-патч у Tauri-generated `MainActivity` — все інше живе в окремих Kotlin-файлах.

**`SecureStore.kt`** — EncryptedSharedPreferences:

```kotlin
private const val PREFS = "mlmail_secure"
private const val KEY_EMAIL = "google_email"
private const val KEY_REFRESH = "google_refresh_token"

private fun prefs(ctx: Context): SharedPreferences {
    val masterKey = MasterKey.Builder(ctx).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build()
    return EncryptedSharedPreferences.create(
        ctx, PREFS, masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )
}

object SecureStore {
    fun save(ctx: Context, email: String, refreshToken: String) =
        prefs(ctx).edit().putString(KEY_EMAIL, email).putString(KEY_REFRESH, refreshToken).apply()

    fun load(ctx: Context): String? {
        val p = prefs(ctx)
        val email = p.getString(KEY_EMAIL, null) ?: return null
        val token = p.getString(KEY_REFRESH, null) ?: return null
        return """{"email":"$email","refresh_token":"$token"}"""
    }

    fun clear(ctx: Context) = prefs(ctx).edit().clear().apply()
}
```

Master key зберігається у Android Keystore (hardware-backed на більшості сучасних пристроїв).

### Gradle залежності

Додати до `app/src-tauri/gen/android/app/build.gradle.kts`:

```kotlin
dependencies {
    implementation("androidx.credentials:credentials:1.3.0")
    implementation("androidx.credentials:credentials-play-services-auth:1.3.0")
    implementation("com.google.android.libraries.identity.googleid:googleid:1.1.1")
    implementation("com.google.android.gms:play-services-auth:21.2.0")
    implementation("androidx.security:security-crypto:1.1.0-alpha06")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-play-services:1.8.1")
}
```

### AndroidManifest

Без змін — `INTERNET` уже є за замовчуванням; Credential Manager / Authorization API не вимагають окремих permissions.

### JNI bridge `auth/flow/android.rs` і `auth/storage/android.rs`

```rust
use jni::objects::{JObject, JString, JValue};

pub async fn run_login_flow(
    app: &AppHandle,
    web_client_id: &str,
) -> Result<TokenResponse, AuthError> {
    let scopes = ["openid", "email", "https://www.googleapis.com/auth/gmail.modify"];

    let (vm, activity) = android_context(app)?;
    let mut env = vm.attach_current_thread()
        .map_err(|e| AuthError::Platform(format!("attach JVM: {e}")))?;

    let j_client_id = env.new_string(web_client_id)?;
    let j_scopes = env.new_object_array(scopes.len() as i32, "java/lang/String", JObject::null())?;
    for (i, s) in scopes.iter().enumerate() {
        env.set_object_array_element(&j_scopes, i as i32, env.new_string(s)?)?;
    }

    let result_obj = env.call_static_method(
        "com/vitaliytv/mlmail/auth/MlmailAuth",
        "signInAndAuthorize",
        "(Landroid/app/Activity;Ljava/lang/String;[Ljava/lang/String;)Ljava/lang/String;",
        &[(&activity).into(), (&j_client_id).into(), (&j_scopes).into()],
    )?.l()?;

    let result_str: String = env.get_string(&JString::from(result_obj))?.into();
    let json: serde_json::Value = serde_json::from_str(&result_str)
        .map_err(|e| AuthError::Platform(format!("parse kotlin result: {e}")))?;

    if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
        return match err {
            "Cancelled" => Err(AuthError::Cancelled),
            other => Err(AuthError::Platform(other.into())),
        };
    }
    let server_auth_code = json["server_auth_code"].as_str().ok_or_else(||
        AuthError::OAuth("missing server_auth_code".into()))?;

    token_exchange::exchange_code(
        config::ANDROID_CLIENT_ID,
        server_auth_code,
        "",  // PKCE не використовується на Android
        "",  // redirect_uri порожній для Android-типу client
        FlowKind::Android,
    ).await
}
```

`android_context(app)` — helper, що повертає `(&JavaVM, JObject(Activity))`. У Tauri 2 доступ до Android handle — через `app.android()` або `tauri::mobile::PluginHandle`; точний API підтвердиться під час реалізації (буде у плані як технічна задача "lookup correct Tauri 2 Android API for JavaVM + Activity").

Storage analogous — JNI виклики до `MlmailAuth.saveSession/loadSession/clearSession` з `app_context`.

## OAuth client setup у Google Cloud Console

Створити **три** OAuth clients:

| Client      | Тип                 | Призначення                                                | Налаштування                                                                                                         |
| ----------- | ------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Web**     | Web application     | `serverClientId` для Credential Manager на Android         | Authorized redirect URIs — порожньо. Authorized JavaScript origins — порожньо. Назва: `MLMaiL Web (Android sign-in)` |
| **Android** | Android             | AuthorizationClient на Android (server auth code → tokens) | Package: `com.vitaliytv.mlmail`. SHA-1: debug keystore (зараз) + release keystore (пізніше)                          |
| **Desktop** | Desktop application | macOS loopback flow                                        | Без додаткових налаштувань. Назва: `MLMaiL Desktop (macOS)`                                                          |

**OAuth consent screen:**

- User type: `External` (підтримує Gmail-акаунти).
- App name: `MLMaiL`.
- Scopes:
  - `openid`
  - `https://www.googleapis.com/auth/userinfo.email`
  - `https://www.googleapis.com/auth/gmail.modify`
- Publishing status: `Testing` (поки немає Google app verification). Test users: email власника.

**Debug SHA-1:**

```sh
keytool -list -v \
  -keystore ~/.android/debug.keystore \
  -alias androiddebugkey \
  -storepass android
```

## Конфігурація і build

### `app/src-tauri/Cargo.toml` — нові залежності

```toml
[dependencies]
# існуючі
tauri        = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
# нові
reqwest      = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tokio        = { version = "1", features = ["net", "io-util", "time", "macros", "rt"] }
base64       = "0.22"
rand         = "0.9"
sha2         = "0.10"
log          = "0.4"
thiserror    = "1"

[target.'cfg(target_os = "macos")'.dependencies]
keyring      = { version = "3", default-features = false, features = ["apple-native"] }

[target.'cfg(target_os = "android")'.dependencies]
jni          = "0.21"

[dev-dependencies]
mockito      = "1"
tokio        = { version = "1", features = ["macros", "rt", "rt-multi-thread"] }
```

### `app/package.json` — нові скрипти і devDependencies

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "devDependencies": {
    "vitest": "^2",
    "@vue/test-utils": "^2",
    "jsdom": "^25"
  }
}
```

### `app/src-tauri/.env.example`

```
MLMAIL_GOOGLE_DESKTOP_CLIENT_ID=YOUR_DESKTOP_CLIENT_ID.apps.googleusercontent.com
MLMAIL_GOOGLE_ANDROID_CLIENT_ID=YOUR_ANDROID_CLIENT_ID.apps.googleusercontent.com
MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID=YOUR_WEB_CLIENT_ID.apps.googleusercontent.com
```

`.env` додається у `.gitignore`. `.env.example` коммітимо.

### `.cspell.json` — нові словникові терміни

Додати: `PKCE`, `keyring`, `keychain`, `loopback`, `JNI`, `EncryptedSharedPreferences`, `serverClientId`, `serverAuthCode`, `googleid`, `tokio`, `reqwest`, `rustls`, `oauth`, `Keystore`, `appauth`, `Credential`, `Stronghold`, `idtoken`, `mlmail`.

## Тести

### Rust (`cargo test --manifest-path app/src-tauri/Cargo.toml`)

| Модуль                   | Покриття                                                                                                                                              |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `auth/pkce.rs`           | `generate()` → verifier 43-128 chars URL-safe; `challenge == base64url(sha256(verifier))`; повторні виклики дають різні пари                          |
| `auth/id_token.rs`       | валідний JWT з `email` → `Some(...)`; невалідний → `None`; JWT без email → `None`; не-JWT → `None`                                                    |
| `auth/token_exchange.rs` | mockito-сервер: `exchange_code` 200 → парс OK; `exchange_refresh` 400 + `invalid_grant` → `ReauthRequired`; 5xx → `Network`; malformed JSON → `OAuth` |
| `auth/state.rs`          | `is_access_token_fresh` true коли expiry > now+30s; false коли менше; false коли токен None                                                           |
| `auth/storage`           | тест-only `InMemoryStorage`: save→load round-trip; clear→load = None                                                                                  |

### JS (`bun run test`)

| Файл                          | Покриття                                                                                                                                                 |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `services/auth-store.test.js` | mock invoke: `initialize()` оновлює `isAuthenticated`; `login()` тимчасово `isLoading=true`; `login()` помилка → `errorKind`; `logout()` зкидає стан     |
| `views/Login.test.js`         | рендер "Увійти через Google" коли не залогінений; "Ви увійшли як X" + "Вийти" коли так; "Зачекайте…" під час логіну; українські повідомлення про помилку |
| `i18n/auth-errors.test.js`    | кожен відомий `kind` повертає унікальну українську строку; невідомий → fallback "Невідома помилка"                                                       |

### Manual e2e (не автоматизовано — нотатки у цьому specі)

**macOS:**

1. `cd app && bun install && cd src-tauri && cp .env.example .env`, заповнити IDs, `cd .. && bun run tauri dev`.
2. Натиснути "Увійти через Google" → відкривається Safari/Chrome → consent screen → редірект `127.0.0.1:PORT/callback` → "Готово, можете закрити це вікно".
3. У застосунку MLMaiL з'являється "Ви увійшли як {email}".
4. Закрити застосунок → `bun run tauri dev` знову → стан логіну відновлено.
5. Перевірити через тимчасову debug-кнопку (видалити перед merge) `get access token` — повертає рядок; натиснути двічі — той самий (кешований); чекати годину — змінився (refresh).
6. Натиснути "Вийти" → Login екран. Перезапустити → знову Login (Keychain очищений).

**Android (емулятор з Google account):**

1. Додати debug SHA-1 у Android OAuth client у Google Cloud Console.
2. `bun run android` на емуляторі.
3. Натиснути "Увійти через Google" → з'являється Credential Manager bottom sheet → обрати акаунт → consent на gmail.modify → email у Login.
4. Перезапустити застосунок → email відновлений.
5. Перевірити refresh аналогічно.
6. Logout → EncryptedSharedPreferences очищені.

## Документація: оновлення C4 та ADR

### `docs/ci4/02-containers.md`

Стрілка HTTPS до Google Identity Services переміщується з Frontend на Backend (Rust). Текстове пояснення про "контейнер MLMaiL Frontend спілкується із зовнішнім Google Identity Services — HTTPS, прямо з WebView" заміняємо на "контейнер MLMaiL Backend спілкується із зовнішнім Google Identity Services — HTTPS через `reqwest`/rustls; токени не виходять з Backend у Frontend".

### `docs/ci4/03-components.md`

- **Auth Component MLMaiL** і **Auth Store MLMaiL** — змінити з `planned` на `implemented`.
- Додати новий блок з підкомпонентами **Auth Module MLMaiL (Rust)**: PKCE, Token Exchange, State, Storage (з двома реалізаціями), Desktop Flow, Android Flow, JNI Bridge.
- Видалити з опису "Auth Component MLMaiL відкриває системний браузер через Plugin Opener" згадку про `tauri-plugin-opener` — це деталь, що жила лише у Backend.

### `docs/ci4/04-code.md`

- Видалити блок про Demo Command `greet` (актуальний код); згадка про неї як про "стартову, що буде видалена" — залишити з оновленим статусом "видалена у ADR-0006".
- Додати секцію Code для нових файлів (всі під `app/src-tauri/src/auth/` + Vue файли).
- Оновити приклад `tauri::Builder::default().…run(...)` у `lib.rs` — без `greet`, з новими auth-командами.

### `docs/ci4/decisions.md`

- "Очікує ADR: де живе API-ключ LLM/TTS у MLMaiL і хто робить HTTPS-виклики" — частково закрите (рішення для Google OAuth: Rust). Уточнити, що LLM/TTS — окреме рішення.
- "Очікує ADR: збереження токенів MLMaiL (keychain / EncryptedSharedPreferences)" — **закрите ADR-0006**: Keychain + EncryptedSharedPreferences, не Stronghold.
- Додати новий блок "Прийняте рішення: Google OAuth у MLMaiL" з посиланням на ADR-0006.

### `docs/adr/ADR-0006-google-oauth.md` (новий)

Один комбінований ADR — три тісно пов'язані рішення:

1. OAuth механіка: PKCE+loopback (macOS) + Credential Manager + AuthorizationClient (Android).
2. Token storage: Keychain (macOS) + EncryptedSharedPreferences (Android).
3. Token surface: Rust holds tokens, Vue queries through Tauri command.

Альтернативи з обґрунтуванням відмови:

- AppAuth-Android — відкинуто на користь Credential Manager (native UX).
- GoogleSignIn — deprecated.
- Stronghold cross-platform — додатковий рівень над system keychains, зайвий.
- Token у JS-пам'яті — слабша безпека проти XSS.
- Rust як проксі для Gmail HTTPS-викликів — поза scope цієї ітерації, але архітектурно сумісне (можна додати, не міняючи auth surface).

Зачіпає: `02-containers.md`, `03-components.md`, `04-code.md`, `decisions.md`, нові файли під `app/src-tauri/src/auth/`, `app/src/views/Login.vue`, `app/src/services/auth-store.js`, `app/src/i18n/auth-errors.js`.

## Ризики і відомі обмеження

1. **Tauri 2 Android API для отримання JavaVM + Activity** — точний публічний API на момент написання spec не зафіксовано у репо MLMaiL. У плані імплементації перший крок Android-частини — встановити правильний шлях (через `tauri::mobile` чи `ndk-context::android_context()`). Низький ризик: один з двох шляхів точно працює, питання лише в API surface.
2. **Master key для EncryptedSharedPreferences** при першому виклику може бути повільним (~100-300 мс). Прийнятно для login flow.
3. **`security-crypto:1.1.0-alpha06`** — alpha. Альтернатива: версія `1.0.0` (stable, але старіша API). Якщо `alpha` створить проблеми у білді — спадаємо на `1.0.0`.
4. **`keyring` v3 на macOS** — користувач при першому збереженні бачить системний промт keychain. Це очікуваний UX.
5. **Google app verification** для `gmail.modify` поки відсутня — кожен test user має бути доданий вручну у OAuth consent screen.
6. **PKCE на Android не використовується** — це обмеження AuthorizationClient API. Безпеку забезпечує SHA-1 fingerprint APK у Google Cloud Console (Android OAuth client).
7. **Loopback redirect URI на macOS** — Google автоматично приймає будь-який порт на `http://127.0.0.1` для Desktop OAuth client type; жодних змін у Cloud Console при перебудові.
