# Google OAuth 2.0 авторизація MLMaiL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Додати Google OAuth 2.0 авторизацію (loopback для macOS, Credential Manager для Android) зі збереженням refresh token у Keychain/EncryptedSharedPreferences і мінімальним Vue UI (Login screen + Vue Router).

**Architecture:** Rust-шар тримає всі токени — access token в пам'яті (AuthState), refresh token у платформному сховищі. Vue Auth Store зберігає лише `isAuthenticated` і `userEmail`. Tauri commands — єдиний контракт між Vue і Rust. Android використовує Tauri mobile plugin для виклику Kotlin Credential Manager API.

**Tech Stack:** Rust (reqwest 0.12, sha2, base64, rand, thiserror, urlencoding, security-framework[macos]), Vue 3 + vue-router 4, Kotlin (credentials-play-services-auth, security-crypto), Tauri 2.

**Spec:** `docs/superpowers/specs/2026-05-11-google-auth-design.md`

---

## Файлова карта

```
app/
├── src/
│   ├── views/
│   │   └── Login.vue                    [new]
│   ├── layouts/
│   │   └── default.vue                  [new — required by vite-plugin-vue-layouts-next]
│   ├── services/
│   │   └── auth-store.js               [new]
│   ├── router/
│   │   └── index.js                    [new]
│   └── App.vue                         [rewrite]
├── src-tauri/
│   ├── tauri.conf.json                  [modify — plugins.mlmail]
│   ├── Cargo.toml                       [modify — нові залежності]
│   └── src/
│       ├── lib.rs                       [modify — auth module + state]
│       └── auth/
│           ├── mod.rs                   [new — Tauri commands]
│           ├── state.rs                 [new — AuthState, MlmailConfig]
│           ├── pkce.rs                  [new — generate_pkce()]
│           ├── token_exchange.rs        [new — exchange_code, refresh_token, parse_email]
│           ├── storage/
│           │   ├── mod.rs              [new — RefreshTokenStorage trait]
│           │   ├── macos.rs            [new — KeychainStorage]
│           │   └── android.rs          [new — EncryptedPrefsStorage via plugin]
│           └── flow/
│               ├── macos.rs            [new — loopback TcpListener flow]
│               └── android.rs          [new — Tauri mobile plugin bridge]
└── gen/android/app/
    ├── src/main/java/com/vitaliytv/mlmail/
    │   ├── MainActivity.kt              [modify — registerPlugin]
    │   └── AuthPlugin.kt               [new — Credential Manager + EncryptedSharedPreferences]
    └── build.gradle.kts                 [modify — нові залежності]
```

---

## Task 1: Rust залежності + конфіг MLMaiL

**Files:**
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src-tauri/tauri.conf.json`

- [ ] **Step 1: Оновити Cargo.toml**

Замінити секцію `[dependencies]` на:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
sha2 = "0.10"
base64 = { version = "0.22", features = [] }
rand = "0.8"
thiserror = "1"
urlencoding = "2"
tokio = { version = "1", features = ["net", "time", "io-util"] }

[target.'cfg(target_os = "macos")'.dependencies]
security-framework = "2"
```

- [ ] **Step 2: Додати plugins.mlmail у tauri.conf.json**

Додати секцію `"plugins"` після секції `"bundle"` у `app/src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "mlmail",
  "version": "0.1.0",
  "identifier": "com.vitaliytv.mlmail",
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "MLMail",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "mlmail": {
      "googleClientIdDesktop": "REPLACE_WITH_DESKTOP_CLIENT_ID.apps.googleusercontent.com",
      "googleClientIdAndroid": "REPLACE_WITH_ANDROID_CLIENT_ID.apps.googleusercontent.com"
    }
  }
}
```

- [ ] **Step 3: Перевірити компіляцію**

```bash
cd app/src-tauri && cargo check 2>&1 | tail -5
```

Очікуваний результат: `Finished` або попередження без помилок.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/Cargo.toml app/src-tauri/tauri.conf.json
git commit -m "chore: add OAuth dependencies and mlmail config to tauri.conf"
```

---

## Task 2: AuthError + AuthState + MlmailConfig

**Files:**
- Create: `app/src-tauri/src/auth/state.rs`
- Create: `app/src-tauri/src/auth/mod.rs` (skeleton)
- Modify: `app/src-tauri/src/lib.rs`

- [ ] **Step 1: Створити `app/src-tauri/src/auth/state.rs`**

```rust
use std::time::Instant;
use crate::auth::storage::RefreshTokenStorage;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MlmailConfig {
    #[serde(rename = "googleClientIdDesktop")]
    pub google_client_id_desktop: String,
    #[serde(rename = "googleClientIdAndroid")]
    pub google_client_id_android: String,
}

pub struct AuthState {
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<Instant>,
    pub http_client: reqwest::Client,
    pub client_id: String,
    pub storage: Box<dyn RefreshTokenStorage>,
}

impl AuthState {
    pub fn new(client_id: String, storage: Box<dyn RefreshTokenStorage>) -> Self {
        Self {
            email: None,
            access_token: None,
            access_token_expires_at: None,
            http_client: reqwest::Client::new(),
            client_id,
            storage,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.email.is_some()
    }

    pub fn set_tokens(&mut self, access_token: String, expires_in: u64, email: String) {
        self.access_token = Some(access_token);
        self.access_token_expires_at =
            Some(Instant::now() + std::time::Duration::from_secs(expires_in));
        self.email = Some(email);
    }

    pub fn is_access_token_valid(&self) -> bool {
        match (&self.access_token, &self.access_token_expires_at) {
            (Some(_), Some(expires_at)) => {
                *expires_at > Instant::now() + std::time::Duration::from_secs(30)
            }
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        self.email = None;
        self.access_token = None;
        self.access_token_expires_at = None;
    }
}
```

- [ ] **Step 2: Створити `app/src-tauri/src/auth/storage/mod.rs`**

```rust
#[derive(Debug)]
pub struct StorageError(pub String);

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storage error: {}", self.0)
    }
}

pub trait RefreshTokenStorage: Send + Sync {
    fn save(&self, token: &str, email: &str) -> Result<(), StorageError>;
    fn load(&self) -> Result<Option<(String, String)>, StorageError>;
    fn clear(&self) -> Result<(), StorageError>;
}

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "android")]
pub mod android;
```

- [ ] **Step 3: Створити `app/src-tauri/src/auth/mod.rs` (skeleton)**

```rust
pub mod storage;
pub mod state;
pub mod pkce;
pub mod token_exchange;
pub mod flow;

use thiserror::Error;

#[derive(Debug, Error, serde::Serialize)]
pub enum AuthError {
    #[error("OAuth failed: {0}")]
    OAuthFailed(String),
    #[error("User cancelled")]
    UserCancelled,
    #[error("Timeout waiting for OAuth")]
    Timeout,
    #[error("Re-authentication required")]
    ReauthRequired,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}
```

- [ ] **Step 4: Оголосити `mod auth` у `app/src-tauri/src/lib.rs`**

Додати рядок після `// Learn more...` коментаря, до функції `greet`:

```rust
mod auth;
```

- [ ] **Step 5: Створити порожні модулі щоб компілятор не скаржився**

Створити `app/src-tauri/src/auth/pkce.rs`:
```rust
// placeholder — реалізація у Task 3
pub fn generate_pkce() -> (String, String) {
    todo!()
}
```

Створити `app/src-tauri/src/auth/token_exchange.rs`:
```rust
// placeholder — реалізація у Task 4
```

Створити `app/src-tauri/src/auth/flow/mod.rs` (порожній файл — Rust потребує `mod.rs` або named file):
```rust
#[cfg(not(target_os = "android"))]
pub mod macos;
#[cfg(target_os = "android")]
pub mod android;
```

Створити `app/src-tauri/src/auth/flow/macos.rs`:
```rust
// placeholder — реалізація у Task 6
```

Створити `app/src-tauri/src/auth/flow/android.rs`:
```rust
// placeholder — реалізація у Task 9
```

Створити `app/src-tauri/src/auth/storage/macos.rs`:
```rust
// placeholder — реалізація у Task 5
use super::{RefreshTokenStorage, StorageError};
pub struct KeychainStorage;
impl RefreshTokenStorage for KeychainStorage {
    fn save(&self, _token: &str, _email: &str) -> Result<(), StorageError> { todo!() }
    fn load(&self) -> Result<Option<(String, String)>, StorageError> { todo!() }
    fn clear(&self) -> Result<(), StorageError> { todo!() }
}
```

Створити `app/src-tauri/src/auth/storage/android.rs`:
```rust
// placeholder — реалізація у Task 9
use super::{RefreshTokenStorage, StorageError};
pub struct EncryptedPrefsStorage;
impl RefreshTokenStorage for EncryptedPrefsStorage {
    fn save(&self, _token: &str, _email: &str) -> Result<(), StorageError> { todo!() }
    fn load(&self) -> Result<Option<(String, String)>, StorageError> { todo!() }
    fn clear(&self) -> Result<(), StorageError> { todo!() }
}
```

- [ ] **Step 6: Перевірити компіляцію**

```bash
cd app/src-tauri && cargo check 2>&1 | grep -E "^error"
```

Очікуваний результат: порожній вивід (без помилок).

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/
git commit -m "feat(auth): scaffold auth module — AuthError, AuthState, storage trait"
```

---

## Task 3: PKCE модуль (TDD)

**Files:**
- Modify: `app/src-tauri/src/auth/pkce.rs`

- [ ] **Step 1: Написати тест (замінити placeholder у pkce.rs)**

```rust
use rand::RngCore;
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

pub fn generate_pkce() -> (String, String) {
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(bytes);
    let hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);
    (code_verifier, code_challenge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_length_in_spec_range() {
        let (verifier, _) = generate_pkce();
        assert!(
            verifier.len() >= 43 && verifier.len() <= 128,
            "verifier length {} out of range [43, 128]",
            verifier.len()
        );
    }

    #[test]
    fn challenge_is_sha256_base64url_of_verifier() {
        let (verifier, challenge) = generate_pkce();
        let hash = Sha256::digest(verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(hash);
        assert_eq!(challenge, expected);
    }

    #[test]
    fn verifier_is_url_safe_chars() {
        let (verifier, _) = generate_pkce();
        for c in verifier.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '-' || c == '_',
                "non-url-safe char: {c}"
            );
        }
    }

    #[test]
    fn two_calls_produce_different_verifiers() {
        let (v1, _) = generate_pkce();
        let (v2, _) = generate_pkce();
        assert_ne!(v1, v2);
    }
}
```

- [ ] **Step 2: Запустити тести (мають пройти)**

```bash
cd app/src-tauri && cargo test auth::pkce 2>&1 | tail -15
```

Очікуваний результат:
```
test auth::pkce::tests::challenge_is_sha256_base64url_of_verifier ... ok
test auth::pkce::tests::two_calls_produce_different_verifiers ... ok
test auth::pkce::tests::verifier_is_url_safe_chars ... ok
test auth::pkce::tests::verifier_length_in_spec_range ... ok
test result: ok. 4 passed
```

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/auth/pkce.rs
git commit -m "feat(auth): PKCE generation with SHA-256 code_challenge (tested)"
```

---

## Task 4: Token exchange + email parsing (TDD)

**Files:**
- Modify: `app/src-tauri/src/auth/token_exchange.rs`

- [ ] **Step 1: Написати token_exchange.rs з тестами**

```rust
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use crate::auth::AuthError;

#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub id_token: Option<String>,
}

pub async fn exchange_code(
    client: &reqwest::Client,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
    client_id: &str,
) -> Result<TokenResponse, AuthError> {
    let params = [
        ("code", code),
        ("code_verifier", code_verifier),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("grant_type", "authorization_code"),
    ];
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| AuthError::NetworkError(e.to_string()))?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AuthError::OAuthFailed(body));
    }
    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| AuthError::OAuthFailed(e.to_string()))
}

pub async fn refresh_access_token(
    client: &reqwest::Client,
    refresh_token: &str,
    client_id: &str,
) -> Result<TokenResponse, AuthError> {
    let params = [
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("grant_type", "refresh_token"),
    ];
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| AuthError::NetworkError(e.to_string()))?;
    let body = resp.text().await.unwrap_or_default();
    if body.contains("invalid_grant") {
        return Err(AuthError::ReauthRequired);
    }
    serde_json::from_str::<TokenResponse>(&body)
        .map_err(|e| AuthError::OAuthFailed(e.to_string()))
}

pub fn parse_email_from_id_token(id_token: &str) -> Option<String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    // Pad base64url to multiple of 4 for standard decoder
    let payload_b64 = parts[1];
    let padding = (4 - payload_b64.len() % 4) % 4;
    let padded = format!("{}{}", payload_b64, "=".repeat(padding));
    let payload_bytes = base64::engine::general_purpose::URL_SAFE.decode(&padded).ok()?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;
    payload.get("email")?.as_str().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jwt(payload_json: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(b"{\"alg\":\"RS256\",\"typ\":\"JWT\"}");
        let payload = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
        format!("{header}.{payload}.fakesig")
    }

    #[test]
    fn parse_email_from_valid_id_token() {
        let jwt = make_jwt(r#"{"email":"user@example.com","sub":"123"}"#);
        assert_eq!(parse_email_from_id_token(&jwt), Some("user@example.com".to_string()));
    }

    #[test]
    fn parse_email_returns_none_for_invalid_jwt() {
        assert_eq!(parse_email_from_id_token("notajwt"), None);
    }

    #[test]
    fn parse_email_returns_none_when_email_missing() {
        let jwt = make_jwt(r#"{"sub":"123"}"#);
        assert_eq!(parse_email_from_id_token(&jwt), None);
    }
}
```

- [ ] **Step 2: Запустити тести**

```bash
cd app/src-tauri && cargo test auth::token_exchange 2>&1 | tail -10
```

Очікуваний результат:
```
test auth::token_exchange::tests::parse_email_from_valid_id_token ... ok
test auth::token_exchange::tests::parse_email_returns_none_for_invalid_jwt ... ok
test auth::token_exchange::tests::parse_email_returns_none_when_email_missing ... ok
test result: ok. 3 passed
```

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/auth/token_exchange.rs
git commit -m "feat(auth): token exchange and email parsing from id_token (tested)"
```

---

## Task 5: macOS Keychain storage

**Files:**
- Modify: `app/src-tauri/src/auth/storage/macos.rs`

- [ ] **Step 1: Замінити placeholder у `storage/macos.rs`**

```rust
use super::{RefreshTokenStorage, StorageError};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

const SERVICE: &str = "com.vitaliytv.mlmail";
const ACCOUNT_TOKEN: &str = "google_refresh_token";
const ACCOUNT_EMAIL: &str = "google_email";

pub struct KeychainStorage;

impl KeychainStorage {
    pub fn new() -> Self {
        Self
    }
}

impl RefreshTokenStorage for KeychainStorage {
    fn save(&self, token: &str, email: &str) -> Result<(), StorageError> {
        set_generic_password(SERVICE, ACCOUNT_TOKEN, token.as_bytes())
            .map_err(|e| StorageError(e.to_string()))?;
        set_generic_password(SERVICE, ACCOUNT_EMAIL, email.as_bytes())
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    fn load(&self) -> Result<Option<(String, String)>, StorageError> {
        let token = match get_generic_password(SERVICE, ACCOUNT_TOKEN) {
            Ok(bytes) => String::from_utf8(bytes).map_err(|e| StorageError(e.to_string()))?,
            Err(_) => return Ok(None),
        };
        let email = match get_generic_password(SERVICE, ACCOUNT_EMAIL) {
            Ok(bytes) => String::from_utf8(bytes).map_err(|e| StorageError(e.to_string()))?,
            Err(_) => return Ok(None),
        };
        Ok(Some((token, email)))
    }

    fn clear(&self) -> Result<(), StorageError> {
        let _ = delete_generic_password(SERVICE, ACCOUNT_TOKEN);
        let _ = delete_generic_password(SERVICE, ACCOUNT_EMAIL);
        Ok(())
    }
}
```

- [ ] **Step 2: Перевірити компіляцію на macOS**

```bash
cd app/src-tauri && cargo check 2>&1 | grep -E "^error"
```

Очікуваний результат: порожній вивід.

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/auth/storage/macos.rs
git commit -m "feat(auth): macOS Keychain storage for refresh token"
```

---

## Task 6: macOS OAuth loopback flow

**Files:**
- Modify: `app/src-tauri/src/auth/flow/macos.rs`

- [ ] **Step 1: Замінити placeholder у `flow/macos.rs`**

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use std::time::Duration;
use crate::auth::AuthError;

pub struct PendingOAuth {
    pub listener: TcpListener,
    pub redirect_uri: String,
    pub auth_url: String,
}

pub fn prepare(client_id: &str, code_challenge: &str) -> Result<PendingOAuth, AuthError> {
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AuthError::OAuthFailed(format!("bind failed: {e}")))?;
    let port = std_listener.local_addr().unwrap().port();
    let listener = TcpListener::from_std(std_listener)
        .map_err(|e| AuthError::OAuthFailed(format!("listener convert: {e}")))?;

    let redirect_uri = format!("http://127.0.0.1:{port}");
    let auth_url = build_auth_url(client_id, &redirect_uri, code_challenge);

    Ok(PendingOAuth { listener, redirect_uri, auth_url })
}

pub async fn wait_for_code(listener: TcpListener) -> Result<String, AuthError> {
    let (mut stream, _) = timeout(Duration::from_secs(300), listener.accept())
        .await
        .map_err(|_| AuthError::Timeout)?
        .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let code = parse_code_from_request(&request)?;

    let response = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Type: text/html; charset=utf-8\r\n\r\n",
        "<html><body>Authentication successful. You can close this window.</body></html>"
    );
    let _ = stream.write_all(response.as_bytes()).await;

    Ok(code)
}

fn build_auth_url(client_id: &str, redirect_uri: &str, code_challenge: &str) -> String {
    let scope = urlencoding::encode(
        "openid email https://www.googleapis.com/auth/gmail.modify",
    );
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
        ?client_id={client_id}\
        &redirect_uri={redirect_uri_enc}\
        &response_type=code\
        &scope={scope}\
        &code_challenge={code_challenge}\
        &code_challenge_method=S256\
        &access_type=offline\
        &prompt=consent",
        client_id = urlencoding::encode(client_id),
        redirect_uri_enc = urlencoding::encode(redirect_uri),
        scope = scope,
        code_challenge = urlencoding::encode(code_challenge),
    )
}

fn parse_code_from_request(request: &str) -> Result<String, AuthError> {
    // First line: "GET /?code=XXX&scope=... HTTP/1.1"
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");
    let query = path.split('?').nth(1).unwrap_or("");

    for param in query.split('&') {
        if let Some(code) = param.strip_prefix("code=") {
            return Ok(urlencoding::decode(code)
                .unwrap_or_default()
                .to_string());
        }
        if let Some(error) = param.strip_prefix("error=") {
            if error == "access_denied" {
                return Err(AuthError::UserCancelled);
            }
            return Err(AuthError::OAuthFailed(error.to_string()));
        }
    }
    Err(AuthError::OAuthFailed("No authorization code in redirect".to_string()))
}
```

- [ ] **Step 2: Перевірити компіляцію**

```bash
cd app/src-tauri && cargo check 2>&1 | grep -E "^error"
```

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/auth/flow/macos.rs
git commit -m "feat(auth): macOS loopback OAuth flow"
```

---

## Task 7: Tauri auth commands + lib.rs wiring

**Files:**
- Modify: `app/src-tauri/src/auth/mod.rs`
- Modify: `app/src-tauri/src/lib.rs`

- [ ] **Step 1: Замінити `auth/mod.rs` на повну реалізацію команд**

```rust
pub mod flow;
pub mod pkce;
pub mod state;
pub mod storage;
pub mod token_exchange;

use state::{AuthState, MlmailConfig};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;

#[derive(Debug, Error, serde::Serialize)]
pub enum AuthError {
    #[error("OAuth failed: {0}")]
    OAuthFailed(String),
    #[error("User cancelled")]
    UserCancelled,
    #[error("Timeout waiting for OAuth")]
    Timeout,
    #[error("Re-authentication required")]
    ReauthRequired,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

#[derive(serde::Serialize)]
pub struct LoginResult {
    pub email: String,
}

pub fn load_config(app: &AppHandle) -> MlmailConfig {
    app.config()
        .plugins
        .0
        .get("mlmail")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .expect("plugins.mlmail missing in tauri.conf.json — add googleClientIdDesktop and googleClientIdAndroid")
}

pub fn create_storage() -> Box<dyn storage::RefreshTokenStorage> {
    #[cfg(target_os = "macos")]
    return Box::new(storage::macos::KeychainStorage::new());

    #[cfg(target_os = "android")]
    return Box::new(storage::android::EncryptedPrefsStorage::new());

    #[cfg(not(any(target_os = "macos", target_os = "android")))]
    panic!("Unsupported platform — only macOS and Android are supported");
}

#[tauri::command]
pub async fn auth_start_login(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<LoginResult, AuthError> {
    let (code_verifier, code_challenge) = pkce::generate_pkce();

    #[cfg(not(target_os = "android"))]
    let (code, redirect_uri) = {
        let client_id = state.lock().unwrap().client_id.clone();
        let pending = flow::macos::prepare(&client_id, &code_challenge)?;
        let redirect_uri = pending.redirect_uri.clone();
        let auth_url = pending.auth_url.clone();
        tauri_plugin_opener::open_url(&app, &auth_url, None::<String>)
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;
        let code = flow::macos::wait_for_code(pending.listener).await?;
        (code, redirect_uri)
    };

    #[cfg(target_os = "android")]
    let (code, redirect_uri) = {
        let code = flow::android::start_android_oauth(&app).await?;
        (code, String::new()) // Android redirect URI is handled by the OS
    };

    let (client_id, http_client) = {
        let s = state.lock().unwrap();
        (s.client_id.clone(), s.http_client.clone())
    };

    let tokens =
        token_exchange::exchange_code(&http_client, &code, &code_verifier, &redirect_uri, &client_id)
            .await?;

    let email = tokens
        .id_token
        .as_deref()
        .and_then(token_exchange::parse_email_from_id_token)
        .ok_or_else(|| AuthError::OAuthFailed("No email in id_token".to_string()))?;

    let refresh_token = tokens
        .refresh_token
        .ok_or_else(|| AuthError::OAuthFailed("No refresh_token received".to_string()))?;

    {
        let mut s = state.lock().unwrap();
        s.storage
            .save(&refresh_token, &email)
            .map_err(|e| AuthError::StorageError(e.to_string()))?;
        s.set_tokens(tokens.access_token, tokens.expires_in, email.clone());
    }

    Ok(LoginResult { email })
}

#[tauri::command]
pub async fn auth_get_access_token(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<String, AuthError> {
    if state.lock().unwrap().is_access_token_valid() {
        return Ok(state.lock().unwrap().access_token.clone().unwrap());
    }

    let (refresh_token, email) = {
        let s = state.lock().unwrap();
        s.storage
            .load()
            .map_err(|e| AuthError::StorageError(e.to_string()))?
            .ok_or(AuthError::ReauthRequired)?
    };

    let (client_id, http_client) = {
        let s = state.lock().unwrap();
        (s.client_id.clone(), s.http_client.clone())
    };

    let tokens =
        token_exchange::refresh_access_token(&http_client, &refresh_token, &client_id).await?;

    {
        let mut s = state.lock().unwrap();
        s.set_tokens(tokens.access_token.clone(), tokens.expires_in, email);
    }

    Ok(tokens.access_token)
}

#[tauri::command]
pub fn auth_logout(state: State<'_, Mutex<AuthState>>) {
    let mut s = state.lock().unwrap();
    let _ = s.storage.clear();
    s.clear();
}

#[tauri::command]
pub fn auth_is_authenticated(state: State<'_, Mutex<AuthState>>) -> bool {
    state.lock().unwrap().is_authenticated()
}

#[tauri::command]
pub fn auth_get_email(state: State<'_, Mutex<AuthState>>) -> Option<String> {
    state.lock().unwrap().email.clone()
}
```

- [ ] **Step 2: Оновити `lib.rs` — реєструвати auth state і команди**

Замінити повний вміст `app/src-tauri/src/lib.rs`:

```rust
mod auth;

use auth::{
    auth_get_access_token, auth_get_email, auth_is_authenticated, auth_logout, auth_start_login,
    create_storage, load_config, state::AuthState,
};
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = load_config(app.handle());

            #[cfg(not(target_os = "android"))]
            let client_id = config.google_client_id_desktop.clone();
            #[cfg(target_os = "android")]
            let client_id = config.google_client_id_android.clone();

            let storage = create_storage();
            let mut auth_state = AuthState::new(client_id, storage);

            // Cold start: try loading saved tokens
            if let Ok(Some((_, email))) = auth_state.storage.load() {
                auth_state.email = Some(email);
            }

            app.manage(Mutex::new(auth_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_start_login,
            auth_get_access_token,
            auth_logout,
            auth_is_authenticated,
            auth_get_email,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Перевірити компіляцію**

```bash
cd app/src-tauri && cargo check 2>&1 | grep -E "^error"
```

- [ ] **Step 4: Запустити всі тести**

```bash
cd app/src-tauri && cargo test 2>&1 | tail -10
```

Очікуваний результат: `test result: ok. 7 passed; 0 failed`

- [ ] **Step 5: Ручний тест macOS (потрібен реальний Google Client ID у tauri.conf.json)**

1. Замінити `REPLACE_WITH_DESKTOP_CLIENT_ID` у `tauri.conf.json` реальним Client ID.
2. `cd app && bun run tauri dev`
3. Перейти на `http://localhost:1420` — має відображатись Login screen (реалізація в Task 10).
4. (Поки без UI — перевіримо після Task 10.)

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/auth/mod.rs app/src-tauri/src/lib.rs
git commit -m "feat(auth): Tauri commands — start_login, get_access_token, logout, is_authenticated"
```

---

## Task 8: Android Kotlin AuthPlugin + Gradle

**Files:**
- Create: `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/AuthPlugin.kt`
- Modify: `app/src-tauri/gen/android/app/src/main/java/com/vitaliytv/mlmail/MainActivity.kt`
- Modify: `app/src-tauri/gen/android/app/build.gradle.kts`

- [ ] **Step 1: Оновити `build.gradle.kts` — додати залежності**

У секцію `dependencies { ... }` перед `apply(from = ...)` додати:

```kotlin
    implementation("androidx.credentials:credentials:1.3.0")
    implementation("androidx.credentials:credentials-play-services-auth:1.3.0")
    implementation("com.google.android.gms:play-services-auth:21.2.0")
    implementation("androidx.security:security-crypto:1.1.0-alpha06")
```

- [ ] **Step 2: Створити `AuthPlugin.kt`**

```kotlin
package com.vitaliytv.mlmail

import android.content.Context
import androidx.credentials.CredentialManager
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.google.android.gms.auth.api.identity.AuthorizationRequest
import com.google.android.gms.auth.api.identity.Identity
import com.google.android.gms.common.api.Scope

@TauriPlugin
class AuthPlugin(private val activity: android.app.Activity) : Plugin(activity) {

    private val prefs by lazy {
        val masterKey = MasterKey.Builder(activity)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        EncryptedSharedPreferences.create(
            activity,
            "mlmail_auth",
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    @Command
    fun startOAuth(invoke: Invoke) {
        val clientId = invoke.getArgs(JSObject::class.java).getString("clientId")
            ?: return invoke.reject("clientId required")

        val authRequest = AuthorizationRequest.builder()
            .setRequestedScopes(
                listOf(
                    Scope("openid"),
                    Scope("email"),
                    Scope("https://www.googleapis.com/auth/gmail.modify"),
                )
            )
            .requestOfflineAccess(clientId)
            .build()

        Identity.getAuthorizationClient(activity)
            .authorize(authRequest)
            .addOnSuccessListener { result ->
                val code = result.serverAuthCode
                if (code != null) {
                    val ret = JSObject()
                    ret.put("code", code)
                    invoke.resolve(ret)
                } else {
                    invoke.reject("No server auth code returned")
                }
            }
            .addOnFailureListener { e ->
                val msg = e.message ?: "Authorization failed"
                if (msg.contains("cancel", ignoreCase = true)) {
                    invoke.reject("UserCancelled")
                } else {
                    invoke.reject(msg)
                }
            }
    }

    @Command
    fun saveToken(invoke: Invoke) {
        val args = invoke.getArgs(JSObject::class.java)
        val token = args.getString("token") ?: return invoke.reject("token required")
        val email = args.getString("email") ?: return invoke.reject("email required")
        prefs.edit()
            .putString("refresh_token", token)
            .putString("email", email)
            .apply()
        invoke.resolve()
    }

    @Command
    fun loadToken(invoke: Invoke) {
        val token = prefs.getString("refresh_token", null)
        val email = prefs.getString("email", null)
        val ret = JSObject()
        if (token != null && email != null) {
            ret.put("token", token)
            ret.put("email", email)
        }
        invoke.resolve(ret)
    }

    @Command
    fun clearToken(invoke: Invoke) {
        prefs.edit().clear().apply()
        invoke.resolve()
    }
}
```

- [ ] **Step 3: Оновити `MainActivity.kt` — зареєструвати плагін**

```kotlin
package com.vitaliytv.mlmail

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        registerPlugin(AuthPlugin::class.java)
    }
}
```

- [ ] **Step 4: Перевірити Gradle sync (потрібен Android Studio або CLI)**

```bash
cd app/src-tauri/gen/android && ./gradlew assembleDebug 2>&1 | grep -E "BUILD|error:" | head -20
```

Очікуваний результат: `BUILD SUCCESSFUL`

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/gen/android/
git commit -m "feat(auth/android): AuthPlugin — Credential Manager, EncryptedSharedPreferences"
```

---

## Task 9: Android Rust bridge + storage

**Files:**
- Modify: `app/src-tauri/src/auth/flow/android.rs`
- Modify: `app/src-tauri/src/auth/storage/android.rs`

- [ ] **Step 1: Реалізувати `flow/android.rs`**

```rust
use tauri::AppHandle;
use crate::auth::AuthError;

#[cfg(target_os = "android")]
pub async fn start_android_oauth(app: &AppHandle) -> Result<String, AuthError> {
    use tauri::Manager;

    let client_id = {
        let state = app.state::<std::sync::Mutex<crate::auth::state::AuthState>>();
        state.lock().unwrap().client_id.clone()
    };

    #[derive(serde::Deserialize)]
    struct OAuthResult {
        code: String,
    }

    let result = app
        .run_mobile_plugin::<OAuthResult>("AuthPlugin", "startOAuth", serde_json::json!({ "clientId": client_id }))
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UserCancelled") || msg.contains("cancel") {
                AuthError::UserCancelled
            } else {
                AuthError::OAuthFailed(msg)
            }
        })?;

    Ok(result.code)
}
```

- [ ] **Step 2: Реалізувати `storage/android.rs`**

```rust
use tauri::AppHandle;
use super::{RefreshTokenStorage, StorageError};

pub struct EncryptedPrefsStorage {
    app: AppHandle,
}

impl EncryptedPrefsStorage {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl RefreshTokenStorage for EncryptedPrefsStorage {
    fn save(&self, token: &str, email: &str) -> Result<(), StorageError> {
        self.app
            .run_mobile_plugin::<()>(
                "AuthPlugin",
                "saveToken",
                serde_json::json!({ "token": token, "email": email }),
            )
            .map_err(|e| StorageError(e.to_string()))
    }

    fn load(&self) -> Result<Option<(String, String)>, StorageError> {
        #[derive(serde::Deserialize)]
        struct TokenData {
            token: Option<String>,
            email: Option<String>,
        }
        let data = self.app
            .run_mobile_plugin::<TokenData>("AuthPlugin", "loadToken", serde_json::Value::Null)
            .map_err(|e| StorageError(e.to_string()))?;
        match (data.token, data.email) {
            (Some(t), Some(e)) => Ok(Some((t, e))),
            _ => Ok(None),
        }
    }

    fn clear(&self) -> Result<(), StorageError> {
        self.app
            .run_mobile_plugin::<()>("AuthPlugin", "clearToken", serde_json::Value::Null)
            .map_err(|e| StorageError(e.to_string()))
    }
}
```

- [ ] **Step 3: Оновити `create_storage()` в `auth/mod.rs` — передати AppHandle для Android**

У `lib.rs` у секції `setup` оновити створення storage:

```rust
        #[cfg(not(target_os = "android"))]
        let storage = create_storage();
        #[cfg(target_os = "android")]
        let storage: Box<dyn auth::storage::RefreshTokenStorage> =
            Box::new(auth::storage::android::EncryptedPrefsStorage::new(app.handle().clone()));
```

І видалити виклик `create_storage()` на Android. Функцію `create_storage` у `auth/mod.rs` залишити лише для macOS:

```rust
pub fn create_storage() -> Box<dyn storage::RefreshTokenStorage> {
    #[cfg(target_os = "macos")]
    return Box::new(storage::macos::KeychainStorage::new());

    #[cfg(not(target_os = "macos"))]
    panic!("Use platform-specific storage construction");
}
```

- [ ] **Step 4: Перевірити компіляцію macOS**

```bash
cd app/src-tauri && cargo check 2>&1 | grep -E "^error"
```

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/auth/flow/android.rs app/src-tauri/src/auth/storage/android.rs app/src-tauri/src/lib.rs
git commit -m "feat(auth/android): Rust bridge to AuthPlugin via run_mobile_plugin"
```

---

## Task 10: Vue Router + Auth Store + Login.vue + App.vue

**Files:**
- Create: `app/src/router/index.js`
- Create: `app/src/services/auth-store.js`
- Create: `app/src/views/Login.vue`
- Create: `app/src/layouts/default.vue`
- Modify: `app/src/App.vue`

- [ ] **Step 1: Встановити vue-router**

```bash
cd app && bun add vue-router
```

Очікуваний результат: `bun add` завершується, `vue-router` з'являється у `package.json` dependencies.

- [ ] **Step 2: Створити `app/src/router/index.js`**

```js
import { createRouter, createWebHistory } from 'vue-router'
import Login from '../views/Login.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', component: Login },
    { path: '/', component: () => import('../views/Home.vue') },
  ],
})

export default router
```

- [ ] **Step 3: Створити `app/src/views/Home.vue` (placeholder)**

```vue
<script setup>
import { useAuth } from '../services/auth-store.js'

const { userEmail, logout } = useAuth()
</script>

<template>
  <div>
    <p>Logged in as {{ userEmail }}</p>
    <button @click="logout">Logout</button>
  </div>
</template>
```

- [ ] **Step 4: Створити `app/src/services/auth-store.js`**

```js
import { invoke } from '@tauri-apps/api/core'

const isAuthenticated = ref(false)
const userEmail = ref(null)

async function init() {
  isAuthenticated.value = await invoke('auth_is_authenticated')
  if (isAuthenticated.value) {
    userEmail.value = await invoke('auth_get_email')
  }
}

async function startLogin() {
  const result = await invoke('auth_start_login')
  userEmail.value = result.email
  isAuthenticated.value = true
}

async function logout() {
  await invoke('auth_logout')
  userEmail.value = null
  isAuthenticated.value = false
}

export function useAuth() {
  return { isAuthenticated, userEmail, init, startLogin, logout }
}
```

- [ ] **Step 5: Створити `app/src/views/Login.vue`**

```vue
<script setup>
import { useAuth } from '../services/auth-store.js'

const router = useRouter()
const { startLogin } = useAuth()

const loading = ref(false)
const error = ref(null)

async function handleLogin() {
  loading.value = true
  error.value = null
  try {
    await startLogin()
    router.replace('/')
  } catch (e) {
    if (e !== 'UserCancelled') {
      error.value = String(e)
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="login">
    <h1>MLMaiL</h1>
    <button :disabled="loading" @click="handleLogin">
      {{ loading ? 'Connecting…' : 'Sign in with Google' }}
    </button>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.login {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  gap: 1rem;
}
.error {
  color: red;
  font-size: 0.875rem;
}
</style>
```

- [ ] **Step 6: Створити `app/src/layouts/default.vue` (required by vite-plugin-vue-layouts-next)**

```vue
<template>
  <RouterView />
</template>
```

- [ ] **Step 7: Переписати `app/src/App.vue`**

```vue
<script setup>
import { useAuth } from './services/auth-store.js'

const router = useRouter()
const { isAuthenticated, init } = useAuth()

onMounted(async () => {
  await init()
  if (isAuthenticated.value) {
    router.replace('/')
  } else {
    router.replace('/login')
  }
})
</script>

<template>
  <RouterView />
</template>
```

- [ ] **Step 8: Підключити vue-router у `app/src/main.js`**

```js
import App from './App.vue'
import router from './router/index.js'

createApp(App).use(router).mount('#app')
```

- [ ] **Step 9: Запустити dev server і перевірити вручну**

```bash
cd app && bun run tauri dev
```

Перевірити:
1. Відкривається застосунок → перенаправляє на `/login` (cold start без токена).
2. Натиснути "Sign in with Google" → відкривається системний браузер з Google OAuth.
3. Пройти авторизацію → браузер закривається, застосунок показує Home з email.
4. Закрити і відкрити знову → одразу Home без Login (cold start з токеном).
5. Натиснути Logout → Login screen.
6. Закрити браузер під час OAuth → залишаємось на Login (без error).

- [ ] **Step 10: Commit**

```bash
git add app/src/ app/package.json app/bun.lock
git commit -m "feat(ui): Vue Router + Login screen + Auth Store"
```

---

## Фінальна перевірка

- [ ] Запустити всі Rust тести: `cd app/src-tauri && cargo test 2>&1 | tail -5` → 7 passed
- [ ] Перевірити macOS build: `cd app && bun run tauri build` → завершується успішно
- [ ] Пройти весь тест-план зі spec: `docs/superpowers/specs/2026-05-11-google-auth-design.md` секція 13

---

## Обмеження і відомі ризики

1. **Android `run_mobile_plugin` API** — якщо цей метод недоступний у вашій версії Tauri 2, перевірте [Tauri 2 mobile plugin docs](https://tauri.app/develop/plugins/). Альтернатива: event-based bridge через `app.emit` / `app.listen`.
2. **`requestOfflineAccess(clientId)`** у Kotlin — метод доступний у `play-services-auth:21.x`. Якщо версія відрізняється, перевірте сумісність API.
3. **`security-framework` версія 2** — може потребувати macOS 10.13+. Якщо потрібна нижча версія, використайте `v1`.
4. **Client ID у `tauri.conf.json`** — замінити placeholder значення реальними перед тестуванням.
