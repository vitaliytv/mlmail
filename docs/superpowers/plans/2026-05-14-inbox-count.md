# Inbox Count on Start Screen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Показати точну кількість листів у мітці `INBOX` Gmail-акаунту користувача на стартовому екрані MLMaiL після успішного логіну.

**Architecture:** Новий Rust-модуль `gmail/` із Tauri-командою `gmail_inbox_count`, що під капотом отримує access token (через рефакторений helper `acquire_access_token`) і робить GET на `users/me/labels/INBOX` Gmail REST API. Vue Auth Store отримує два нових ref-и (`inboxCount`, `inboxErrorKind`) і метод `refreshInboxCount`, який авто-викликається після `initialize()` та `login()`. `Login.vue` рендерить рядок «Листів у скриньці: N» під «Ви увійшли як …».

**Tech Stack:** Rust + Tauri 2, `reqwest` 0.12 (rustls-tls, json), `mockito` 1 для тестів HTTP; Vue 3 + Vitest + `@vue/test-utils`.

**Spec:** [docs/superpowers/specs/2026-05-14-inbox-count-design.md](../specs/2026-05-14-inbox-count-design.md)

---

## File Structure

**Rust (новий модуль `gmail/`):**

- Create: `app/src-tauri/src/gmail/mod.rs` — Tauri-команда `gmail_inbox_count`, internal helpers `fetch_inbox_count_at` (testable URL), `parse_messages_total`, status mapping.
- Create: `app/src-tauri/src/gmail/error.rs` — `GmailError { Network, Http(u16, String), Parse, ReauthRequired, Platform }`.

**Rust (зміни існуючого):**

- Modify: `app/src-tauri/src/auth/mod.rs` — винести тіло `auth_get_access_token` у `pub async fn acquire_access_token(app, state) -> Result<String, AuthError>`; `auth_get_access_token` стає тонкою обгорткою.
- Modify: `app/src-tauri/src/lib.rs` — `pub mod gmail;` + реєстрація `gmail::gmail_inbox_count` у `generate_handler!`.

**Vue:**

- Modify: `app/src/services/auth-store.js` — додати `_inboxCount`, `_inboxErrorKind`, метод `refreshInboxCount`; виклик у `initialize()` (якщо isAuthenticated) і у `login()` (після успіху); скидання у `logout()` і `_resetForTest()`.
- Modify: `app/src/services/auth-store.test.js` — новий suite «inbox count».
- Modify: `app/src/views/Login.vue` — три можливі рядки під «Ви увійшли як …»: число / помилка / плейсхолдер.
- Modify: `app/src/views/Login.test.js` — тест на рендер числа і на рендер помилки.
- Modify: `app/src/i18n/auth-errors.js` — додати ключі `Http` і `Parse`.
- Modify: `app/src/i18n/auth-errors.test.js` — кейси на нові ключі.

**Docs:**

- Modify: `docs/ci4/03-components.md` — додати компонент Gmail Module MLMaiL (Rust) і позначити нові поля Auth Store / Auth Component.
- Modify: `docs/ci4/04-code.md` — секція для `app/src-tauri/src/gmail/`, оновити сигнатуру Auth Store.
- Modify: `docs/ci4/decisions.md` — нове рішення «Inbox count: точне число через `users.labels.get?id=INBOX`».
- Create: `docs/adr/_inbox/<timestamp>-inbox-count.md` — ADR-нотатка про вибір endpoint.

---

## Task 1: Refactor `auth_get_access_token` to expose `acquire_access_token` helper

**Files:**
- Modify: `app/src-tauri/src/auth/mod.rs:123-166`

**Why first:** Інший модуль (`gmail`) має отримати валідний access token однаковим шляхом (з рефрешем при потребі, з тим самим обробленням `ReauthRequired`). Винесення в helper — це чистий рефакторинг, наявні поведінкові тести Auth не повинні впасти.

- [ ] **Step 1: Run existing auth tests (baseline)**

Run: `cd app/src-tauri && cargo test auth`
Expected: усі auth-тести PASS (поточні 32 unit-тести).

- [ ] **Step 2: Refactor — винести логіку у `acquire_access_token`**

Замінити блок у `app/src-tauri/src/auth/mod.rs`, що починається `#[tauri::command] pub async fn auth_get_access_token` і закінчується перед `#[tauri::command] pub fn auth_is_authenticated`, на такий код:

```rust
pub async fn acquire_access_token(
    app: &AppHandle,
    state: &State<'_, Mutex<AuthState>>,
) -> Result<String, AuthError> {
    {
        let s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
        if s.is_access_token_fresh() {
            return Ok(s.access_token.clone().unwrap());
        }
    }

    let storage = make_storage(app);
    let stored = storage
        .load()?
        .ok_or(AuthError::ReauthRequired)?;

    let resp = token_exchange::exchange_refresh(
        client_id_for_refresh(),
        client_secret_for_refresh(),
        &stored.refresh_token,
    )
    .await;

    match resp {
        Ok(resp) => {
            if let Some(new_rt) = resp.refresh_token.as_deref() {
                storage.save(&stored.email, new_rt)?;
            }
            let token = resp.access_token.clone();
            let mut s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
            apply_token_response(&mut s, &resp, None);
            Ok(token)
        }
        Err(AuthError::ReauthRequired) => {
            let _ = storage.clear();
            if let Ok(mut s) = state.lock() {
                s.reset();
            }
            Err(AuthError::ReauthRequired)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn auth_get_access_token(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<String, AuthError> {
    acquire_access_token(&app, &state).await
}
```

- [ ] **Step 3: Build to confirm rename compiles**

Run: `cd app/src-tauri && cargo build`
Expected: SUCCESS, без warnings про `acquire_access_token`.

- [ ] **Step 4: Re-run auth tests**

Run: `cd app/src-tauri && cargo test auth`
Expected: усі auth-тести лишаються PASS.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/auth/mod.rs
git commit -m "$(cat <<'EOF'
refactor(auth): extract acquire_access_token helper

Окремий pub-функція дає gmail-модулю використовувати ту саму логіку
рефрешу та обробки ReauthRequired, що й auth_get_access_token.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Create `GmailError` enum

**Files:**
- Create: `app/src-tauri/src/gmail/error.rs`

- [ ] **Step 1: Write the error module**

Create `app/src-tauri/src/gmail/error.rs` with:

```rust
use crate::auth::error::AuthError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum GmailError {
    #[error("network error: {0}")]
    Network(String),
    #[error("gmail http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("could not parse gmail response: {0}")]
    Parse(String),
    #[error("re-authentication required")]
    ReauthRequired,
    #[error("platform error: {0}")]
    Platform(String),
}

impl From<reqwest::Error> for GmailError {
    fn from(e: reqwest::Error) -> Self {
        GmailError::Network(e.to_string())
    }
}

impl From<AuthError> for GmailError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::ReauthRequired => GmailError::ReauthRequired,
            AuthError::Network(m) => GmailError::Network(m),
            AuthError::Platform(m) => GmailError::Platform(m),
            other => GmailError::Platform(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_reauth_maps_to_gmail_reauth() {
        let e: GmailError = AuthError::ReauthRequired.into();
        assert!(matches!(e, GmailError::ReauthRequired));
    }

    #[test]
    fn auth_network_maps_to_gmail_network() {
        let e: GmailError = AuthError::Network("dns".into()).into();
        match e {
            GmailError::Network(m) => assert_eq!(m, "dns"),
            _ => panic!("expected Network"),
        }
    }

    #[test]
    fn serializes_with_tagged_kind() {
        let e = GmailError::Http { status: 503, body: "boom".into() };
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("\"kind\":\"Http\""));
        assert!(s.contains("\"status\":503"));
    }
}
```

- [ ] **Step 2: Run the tests**

(Module not yet registered, so cargo will not see it. We register the parent module in Task 3 — for now only verify the file parses.)

Run: `cd app/src-tauri && cargo check`
Expected: SUCCESS (or warning «file `gmail/error.rs` found but not module declaration» — that's fine, registered next).

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/gmail/error.rs
git commit -m "$(cat <<'EOF'
feat(gmail): add GmailError with auth-error conversions

Готує помилковий шар для майбутньої команди gmail_inbox_count.
serde-тег kind/message сумісний із форматом AuthError у фронті.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Create `gmail/mod.rs` with `fetch_inbox_count_at` + parse helper + status mapping (TDD with mockito)

**Files:**
- Create: `app/src-tauri/src/gmail/mod.rs`

- [ ] **Step 1: Write the failing tests first**

Create `app/src-tauri/src/gmail/mod.rs` containing module skeleton + tests (no implementation yet — only declarations):

```rust
pub mod error;

use crate::auth::{self, state::AuthState};
use crate::gmail::error::GmailError;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, State};

pub const GMAIL_LABEL_INBOX_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/labels/INBOX";

#[derive(Deserialize)]
struct LabelResponse {
    #[serde(rename = "messagesTotal")]
    messages_total: Option<u64>,
}

pub fn parse_messages_total(body: &str) -> Result<u64, GmailError> {
    let v: LabelResponse =
        serde_json::from_str(body).map_err(|e| GmailError::Parse(e.to_string()))?;
    v.messages_total
        .ok_or_else(|| GmailError::Parse("messagesTotal missing".into()))
}

pub(crate) async fn fetch_inbox_count_at(
    endpoint: &str,
    access_token: &str,
) -> Result<u64, GmailError> {
    let resp = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status.is_success() {
        return parse_messages_total(&body);
    }
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    Err(GmailError::Http {
        status: status.as_u16(),
        body,
    })
}

#[tauri::command]
pub async fn gmail_inbox_count(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<u64, GmailError> {
    let token = auth::acquire_access_token(&app, &state).await?;
    fetch_inbox_count_at(GMAIL_LABEL_INBOX_URL, &token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_messages_total_extracts_number() {
        let body = r#"{"id":"INBOX","name":"INBOX","messagesTotal":348,"threadsTotal":210}"#;
        assert_eq!(parse_messages_total(body).unwrap(), 348);
    }

    #[test]
    fn parse_messages_total_missing_field_returns_parse_error() {
        let body = r#"{"id":"INBOX","name":"INBOX"}"#;
        let err = parse_messages_total(body).unwrap_err();
        assert!(matches!(err, GmailError::Parse(_)));
    }

    #[test]
    fn parse_messages_total_invalid_json_returns_parse_error() {
        let err = parse_messages_total("not json").unwrap_err();
        assert!(matches!(err, GmailError::Parse(_)));
    }

    #[tokio::test]
    async fn fetch_inbox_count_returns_parsed_total_on_200() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/inbox")
            .match_header("authorization", "Bearer AT-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"messagesTotal":42}"#)
            .create_async()
            .await;

        let n = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap();
        assert_eq!(n, 42);
        m.assert_async().await;
    }

    #[tokio::test]
    async fn fetch_inbox_count_maps_401_to_reauth_required() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(401)
            .with_body(r#"{"error":{"code":401,"message":"Invalid Credentials"}}"#)
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn fetch_inbox_count_maps_5xx_to_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(503)
            .with_body("upstream down")
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetch_inbox_count_maps_403_to_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(403)
            .with_body("forbidden")
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 403),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetch_inbox_count_returns_parse_error_when_body_lacks_field() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(200)
            .with_body(r#"{"id":"INBOX"}"#)
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::Parse(_)));
    }
}
```

- [ ] **Step 2: Register module in `lib.rs`**

Modify `app/src-tauri/src/lib.rs` to add `pub mod gmail;` after `pub mod auth;`:

```rust
pub mod auth;
pub mod gmail;

use std::sync::Mutex;
```

(Реєстрація команди у `generate_handler!` йде окремою задачею — Task 4. Тут лише `pub mod gmail;` щоб `cargo test` побачив файл і запустив його тести.)

- [ ] **Step 3: Run gmail unit tests**

Run: `cd app/src-tauri && cargo test gmail`
Expected: усі 7 тестів модуля `gmail` PASS (`parse_messages_total_extracts_number`, `parse_messages_total_missing_field_returns_parse_error`, `parse_messages_total_invalid_json_returns_parse_error`, `fetch_inbox_count_returns_parsed_total_on_200`, `fetch_inbox_count_maps_401_to_reauth_required`, `fetch_inbox_count_maps_5xx_to_http_error`, `fetch_inbox_count_maps_403_to_http_error`, `fetch_inbox_count_returns_parse_error_when_body_lacks_field` — 8 шт.).

- [ ] **Step 4: Run full test suite to verify no regressions**

Run: `cd app/src-tauri && cargo test`
Expected: усі тести PASS (попередні + 8 нових + 3 з error.rs).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/gmail/mod.rs app/src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(gmail): fetch_inbox_count + parse helper + status mapping

Реалізує HTTPS-виклик до users.labels.get?id=INBOX через reqwest,
маппінг 401→ReauthRequired, інші не-2xx→Http{status,body}, JSON parse
errors→Parse. Тести з mockito ізолюють HTTP від реального Google.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Register `gmail_inbox_count` Tauri command in `lib.rs::run()`

**Files:**
- Modify: `app/src-tauri/src/lib.rs:14-21`

- [ ] **Step 1: Append handler**

Replace the `invoke_handler` block in `app/src-tauri/src/lib.rs`:

```rust
        .invoke_handler(tauri::generate_handler![
            auth::auth_start_login,
            auth::auth_get_access_token,
            auth::auth_is_authenticated,
            auth::auth_current_email,
            auth::auth_logout,
            gmail::gmail_inbox_count,
        ])
```

- [ ] **Step 2: Build**

Run: `cd app/src-tauri && cargo build`
Expected: SUCCESS.

- [ ] **Step 3: Run full tests one more time**

Run: `cd app/src-tauri && cargo test`
Expected: усі PASS.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(gmail): wire gmail_inbox_count into Tauri handlers

Тепер фронт може кликати invoke('gmail_inbox_count').

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Extend `auth-store.js` with `inboxCount` / `inboxErrorKind` / `refreshInboxCount` (TDD)

**Files:**
- Modify: `app/src/services/auth-store.js`
- Modify: `app/src/services/auth-store.test.js`

- [ ] **Step 1: Add failing tests to `auth-store.test.js`**

Append to `app/src/services/auth-store.test.js`:

```js
describe('useAuthStore inbox count', () => {
  it('refreshInboxCount sets inboxCount on success', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(348)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxCount.value).toBe(348)
    expect(store.inboxErrorKind.value).toBe(null)
  })

  it('login also refreshes inbox count', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_start_login') return Promise.resolve({ email: 'u@e' })
      if (cmd === 'gmail_inbox_count') return Promise.resolve(12)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.login()
    expect(store.inboxCount.value).toBe(12)
  })

  it('captures error.kind on Gmail failure (Http)', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.reject({ kind: 'Http' })
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxCount.value).toBe(null)
    expect(store.inboxErrorKind.value).toBe('Http')
  })

  it('falls back to Unknown when Gmail error has no kind', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.reject('boom')
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxErrorKind.value).toBe('Unknown')
  })

  it('ReauthRequired from Gmail forces logout state', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.reject({ kind: 'ReauthRequired' })
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
    expect(store.inboxCount.value).toBe(null)
  })

  it('logout clears inboxCount and inboxErrorKind', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(9)
      if (cmd === 'auth_logout') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.inboxCount.value).toBe(9)
    await store.logout()
    expect(store.inboxCount.value).toBe(null)
    expect(store.inboxErrorKind.value).toBe(null)
  })

  it('does not call gmail_inbox_count when not authenticated', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_inbox_count')
    expect(store.inboxCount.value).toBe(null)
  })
})
```

- [ ] **Step 2: Run tests — expect failures**

Run: `cd /Users/vitaliytv/www/vitaliytv/mlmail && bun --cwd app run test`
Expected: усі 7 нових тестів FAIL (`store.inboxCount` undefined).

- [ ] **Step 3: Update `auth-store.js`**

Overwrite `app/src/services/auth-store.js` with:

```js
import { invoke } from '@tauri-apps/api/core'
import { readonly, ref } from 'vue'

const _email = ref(null)
const _isAuthenticated = ref(false)
const _isLoading = ref(false)
const _errorKind = ref(null)
const _inboxCount = ref(null)
const _inboxErrorKind = ref(null)

export function useAuthStore() {
  async function refreshInboxCount() {
    if (!_isAuthenticated.value) return
    try {
      _inboxCount.value = await invoke('gmail_inbox_count')
      _inboxErrorKind.value = null
    } catch (err) {
      const kind = err && typeof err === 'object' && err.kind ? err.kind : 'Unknown'
      _inboxCount.value = null
      _inboxErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
  }

  async function initialize() {
    const ok = await invoke('auth_is_authenticated')
    _isAuthenticated.value = ok
    if (ok) {
      _email.value = await invoke('auth_current_email')
      await refreshInboxCount()
    }
  }

  async function login() {
    _isLoading.value = true
    _errorKind.value = null
    try {
      const session = await invoke('auth_start_login')
      _email.value = session.email
      _isAuthenticated.value = true
      await refreshInboxCount()
    } catch (err) {
      _errorKind.value = err && typeof err === 'object' && err.kind ? err.kind : 'Unknown'
    } finally {
      _isLoading.value = false
    }
  }

  async function getAccessToken() {
    return invoke('auth_get_access_token')
  }

  async function logout() {
    await invoke('auth_logout')
    _email.value = null
    _isAuthenticated.value = false
    _errorKind.value = null
    _inboxCount.value = null
    _inboxErrorKind.value = null
  }

  return {
    email: readonly(_email),
    isAuthenticated: readonly(_isAuthenticated),
    isLoading: readonly(_isLoading),
    errorKind: readonly(_errorKind),
    inboxCount: readonly(_inboxCount),
    inboxErrorKind: readonly(_inboxErrorKind),
    initialize,
    login,
    getAccessToken,
    logout,
    refreshInboxCount
  }
}

export function _resetForTest() {
  _email.value = null
  _isAuthenticated.value = false
  _isLoading.value = false
  _errorKind.value = null
  _inboxCount.value = null
  _inboxErrorKind.value = null
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `bun --cwd app run test`
Expected: усі тести PASS (попередні + 7 нових).

- [ ] **Step 5: Commit**

```bash
git add app/src/services/auth-store.js app/src/services/auth-store.test.js
git commit -m "$(cat <<'EOF'
feat(auth-store): track inbox count via gmail_inbox_count

Додає inboxCount/inboxErrorKind ref-и і метод refreshInboxCount,
що автоматично викликається після initialize/login. ReauthRequired
із Gmail повертає юзера в розлогінений стан, як і відмова refresh.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Update i18n with `Http` and `Parse` kinds (TDD)

**Files:**
- Modify: `app/src/i18n/auth-errors.js`
- Modify: `app/src/i18n/auth-errors.test.js`

- [ ] **Step 1: Read existing test file**

Run: `cat app/src/i18n/auth-errors.test.js`
Expected: знайомий формат testів на `errorMessage(kind)`.

- [ ] **Step 2: Append failing tests**

Append to `app/src/i18n/auth-errors.test.js`:

```js
describe('errorMessage Gmail kinds', () => {
  it('returns Ukrainian text for Http kind', () => {
    expect(errorMessage('Http')).toBe('Gmail повернув помилку. Спробуйте пізніше.')
  })

  it('returns Ukrainian text for Parse kind', () => {
    expect(errorMessage('Parse')).toBe('Несподівана відповідь від Gmail.')
  })
})
```

- [ ] **Step 3: Run tests — expect failures**

Run: `bun --cwd app run test auth-errors`
Expected: дві нові FAIL.

- [ ] **Step 4: Update `auth-errors.js`**

Replace dictionary in `app/src/i18n/auth-errors.js`:

```js
const messages = {
  Cancelled: 'Логін скасовано.',
  Network: "Не вдалося з'єднатися з Google. Перевірте мережу.",
  OAuth: 'Помилка авторизації Google.',
  Storage: 'Не вдалося зберегти токен у захищене сховище пристрою.',
  ReauthRequired: 'Сеанс прострочений — увійдіть знову.',
  Platform: 'Помилка платформи.',
  Http: 'Gmail повернув помилку. Спробуйте пізніше.',
  Parse: 'Несподівана відповідь від Gmail.',
  Unknown: 'Невідома помилка.'
}

export function errorMessage(kind) {
  if (kind === null || kind === undefined) return messages.Unknown
  return messages[kind] ?? messages.Unknown
}
```

- [ ] **Step 5: Run tests — expect pass**

Run: `bun --cwd app run test auth-errors`
Expected: усі PASS.

- [ ] **Step 6: Commit**

```bash
git add app/src/i18n/auth-errors.js app/src/i18n/auth-errors.test.js
git commit -m "$(cat <<'EOF'
feat(i18n): add Ukrainian messages for Gmail Http/Parse kinds

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Render inbox count on `Login.vue` (TDD)

**Files:**
- Modify: `app/src/views/Login.vue`
- Modify: `app/src/views/Login.test.js`

- [ ] **Step 1: Append failing tests**

Append to `app/src/views/Login.test.js`:

```js
describe('Login.vue inbox count', () => {
  it('renders "Листів у скриньці: 348" after successful initialize', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(348)
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('Листів у скриньці: 348')
  })

  it('shows placeholder before count loads', async () => {
    let resolveCount
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return new Promise((r) => { resolveCount = r })
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('Листів у скриньці: …')
    resolveCount(7)
    await flushPromises()
    expect(w.text()).toContain('Листів у скриньці: 7')
  })

  it('shows Ukrainian error when Gmail returns Http error', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.reject({ kind: 'Http' })
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('Gmail повернув помилку. Спробуйте пізніше.')
  })
})
```

- [ ] **Step 2: Run tests — expect failures**

Run: `bun --cwd app run test Login`
Expected: три нові FAIL.

- [ ] **Step 3: Update `Login.vue`**

Replace `app/src/views/Login.vue` with:

```vue
<script setup>
import { onMounted } from 'vue'
import { useAuthStore } from '../services/auth-store.js'
import { errorMessage } from '../i18n/auth-errors.js'

const auth = useAuthStore()
onMounted(() => auth.initialize())
</script>

<template>
  <main class="login">
    <h1>MLMaiL</h1>
    <div v-if="auth.isAuthenticated.value" class="signed-in">
      <p>Ви увійшли як {{ auth.email.value }}</p>
      <p v-if="auth.inboxCount.value !== null" class="inbox-count">
        Листів у скриньці: {{ auth.inboxCount.value }}
      </p>
      <p v-else-if="auth.inboxErrorKind.value" class="error">
        {{ errorMessage(auth.inboxErrorKind.value) }}
      </p>
      <p v-else class="inbox-count muted">Листів у скриньці: …</p>
      <button type="button" @click="auth.logout()">Вийти</button>
    </div>
    <button
      v-else
      type="button"
      :disabled="auth.isLoading.value"
      @click="auth.login()"
    >
      {{ auth.isLoading.value ? 'Зачекайте…' : 'Увійти через Google' }}
    </button>
    <p v-if="auth.errorKind.value" class="error">
      {{ errorMessage(auth.errorKind.value) }}
    </p>
  </main>
</template>

<style scoped>
.login {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding-top: 10vh;
  text-align: center;
}

.login button {
  padding: 0.6em 1.2em;
  border-radius: 8px;
  border: 1px solid transparent;
  background: #fff;
  cursor: pointer;
  font: inherit;
}

.login button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.inbox-count {
  margin: 0;
}

.inbox-count.muted {
  opacity: 0.6;
}

.error {
  color: #b00020;
}
</style>
```

- [ ] **Step 4: Run tests — expect pass**

Run: `bun --cwd app run test`
Expected: усі PASS (попередні + 3 нові).

- [ ] **Step 5: Commit**

```bash
git add app/src/views/Login.vue app/src/views/Login.test.js
git commit -m "$(cat <<'EOF'
feat(login): show inbox count on start screen

Під «Ви увійшли як …» рендериться число листів у мітці INBOX. Поки
число вантажиться — плейсхолдер «Листів у скриньці: …». При помилці
Gmail показуємо український рядок з auth-errors i18n.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Update `docs/ci4/03-components.md`

**Files:**
- Modify: `docs/ci4/03-components.md`

- [ ] **Step 1: Add Gmail Module MLMaiL block + mark Auth Store/Component features**

У секції «Компоненти контейнера MLMaiL Backend» додати новий вузол до Mermaid-діаграми (між `AuthMod` і `NotesCmd`):

```
    GmailMod[Gmail Module MLMaiL<br/>app/src-tauri/src/gmail/<br/>implemented]
```

І стрілку:

```
    EntryLib --> GmailMod
    GmailMod --> AuthMod
```

Після секції «Компонент Auth Module MLMaiL (implemented)» додати:

```markdown
### Компонент Gmail Module MLMaiL (implemented)

Gmail Module MLMaiL — Rust-підсистема у
[app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/), що інкапсулює
HTTPS-виклики до Gmail REST API від імені MLMaiL Backend. У цій ітерації
експортує одну Tauri-команду:

- `gmail_inbox_count() -> Result<u64, GmailError>` — повертає точне
  `messagesTotal` мітки `INBOX` через `GET users/me/labels/INBOX`. Під капотом
  кличе `auth::acquire_access_token`, на 401 від Gmail повертає
  `ReauthRequired` (Auth Store MLMaiL переводить UI у стан «не залогінений»).

Підкомпоненти:

| Файл | Роль |
| ---- | ---- |
| `mod.rs` | Tauri-команда `gmail_inbox_count`, `fetch_inbox_count_at` (URL-параметр для тестів через `mockito`), `parse_messages_total` |
| `error.rs` | `GmailError { Network, Http{status,body}, Parse, ReauthRequired, Platform }` + конверсії з `reqwest::Error` і `AuthError` |

Залежить від: Auth Module MLMaiL (через `acquire_access_token`), `reqwest`,
`serde_json`.
```

У секції «Компонент Auth Store MLMaiL (implemented)» оновити перелік стану:

Замінити рядок «`email`, `isAuthenticated`, `isLoading`, `errorKind`» на:

«`email`, `isAuthenticated`, `isLoading`, `errorKind`, `inboxCount`, `inboxErrorKind`»

І замінити «Поверхня Auth Store MLMaiL: `initialize`, `login`, `getAccessToken`, `logout` …» на:

«Поверхня Auth Store MLMaiL: `initialize`, `login`, `getAccessToken`, `logout`, `refreshInboxCount` …»

У Frontend-діаграмі додати вузол `GmailIPC[Gmail IPC bridge MLMaiL<br/>invoke('gmail_inbox_count')<br/>implemented]` і стрілки `Auth --> GmailIPC` та `GmailIPC --> AuthStore` (опційно — або просто дописати у текстовий блок Auth Store, що той викликає `gmail_inbox_count`).

У описі Auth Component MLMaiL після списку гілок UI додати:

«У гілці «авторизовано» додатково рендериться рядок `Листів у скриньці: N` (число — з Auth Store MLMaiL `inboxCount`). Якщо число ще не отримано — плейсхолдер `…`; при помилці — український рядок з Auth Errors i18n MLMaiL.»

У секції «Компонент Auth Errors i18n MLMaiL» замінити «Сім ключів» на «Девʼять ключів» і доповнити перелік: «`Http`, `Parse`».

- [ ] **Step 2: Commit**

```bash
git add docs/ci4/03-components.md
git commit -m "$(cat <<'EOF'
docs(ci4): components — додати Gmail Module + оновити Auth Store

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Update `docs/ci4/04-code.md`

**Files:**
- Modify: `docs/ci4/04-code.md`

- [ ] **Step 1: Add code sections for `gmail/`**

Після секції «Файл [app/src-tauri/src/auth/…]» (там, де закінчується опис auth-модуля) додати новий блок:

```markdown
### Файл [app/src-tauri/src/gmail/mod.rs](../../app/src-tauri/src/gmail/mod.rs)

Gmail Module MLMaiL — точка входу. Експортує константу `GMAIL_LABEL_INBOX_URL`,
приватний deserializer `LabelResponse`, чисту функцію
`parse_messages_total(body: &str) -> Result<u64, GmailError>`, helper
`fetch_inbox_count_at(endpoint, access_token)` (URL — параметр, щоб
unit-тести з `mockito` ловили реквест) і Tauri-команду:

```rust
#[tauri::command]
pub async fn gmail_inbox_count(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<u64, GmailError> {
    let token = auth::acquire_access_token(&app, &state).await?;
    fetch_inbox_count_at(GMAIL_LABEL_INBOX_URL, &token).await
}
```

Status-мапінг: 401 → `GmailError::ReauthRequired`; інші не-2xx → `GmailError::Http { status, body }`; JSON-помилки → `GmailError::Parse`; `reqwest::Error` → `GmailError::Network`.

### Файл [app/src-tauri/src/gmail/error.rs](../../app/src-tauri/src/gmail/error.rs)

`GmailError` із `#[serde(tag = "kind", content = "message")]` — frontend дістає
`err.kind` так само, як для `AuthError`. Має `From<reqwest::Error>` та
`From<AuthError>` (зокрема `AuthError::ReauthRequired → GmailError::ReauthRequired`).
```

У блоці «Файл [app/src/services/auth-store.js]» оновити список ref-ів і метод-сурфейс: додати `inboxCount`, `inboxErrorKind`, `refreshInboxCount`.

У блоці «Файл [app/src/i18n/auth-errors.js]» оновити «Сім ключів» → «Девʼять ключів» (додані `Http`, `Parse`).

У блоці «Файл [app/src-tauri/src/lib.rs]» оновити приклад `invoke_handler!` (додати `gmail::gmail_inbox_count`) та коментар про `pub mod gmail;`.

- [ ] **Step 2: Commit**

```bash
git add docs/ci4/04-code.md
git commit -m "$(cat <<'EOF'
docs(ci4): code-level — додати gmail/ модуль і оновити auth-store

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Update `docs/ci4/decisions.md` + create ADR-inbox note

**Files:**
- Modify: `docs/ci4/decisions.md`
- Create: `docs/adr/_inbox/<YYYYMMDD-HHMMSS-<hex8>>-inbox-count.md`

- [ ] **Step 1: Append to `decisions.md` під «Прийняті рішення MLMaiL»**

Після секції «Рішення: Google OAuth-архітектура MLMaiL» додати:

```markdown
### Рішення: Кількість листів у скриньці — точне число через `users.labels.get`

Закодовано у [app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/) і
[app/src/services/auth-store.js](../../app/src/services/auth-store.js).

Endpoint `GET users/me/labels/INBOX` повертає точне `messagesTotal` одним
викликом. Альтернативи (`users.getProfile` — рахує всю скриньку, не INBOX;
`messages.list?labelIds=INBOX` — повертає приблизне `resultSizeEstimate`)
відкинуто свідомо. HTTPS-виклик живе у Rust, не у WebView fetch — узгоджено з
ADR-0006 (token surface).

Вплив на C4-модель MLMaiL:

- [03-components.md](03-components.md) — додано Gmail Module MLMaiL (Rust),
  оновлено Auth Store MLMaiL (`inboxCount`, `refreshInboxCount`).
- [04-code.md](04-code.md) — додано секції `app/src-tauri/src/gmail/mod.rs` і
  `error.rs`, оновлено приклад `invoke_handler!` у `lib.rs`.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0007-inbox-count.md`.
Чернетка інбоксу — у `docs/adr/_inbox/`.
```

- [ ] **Step 2: Create ADR-inbox note**

Run: `date +%Y%m%d-%H%M%S` для timestamp і `openssl rand -hex 4` для суфіксу.

Create `docs/adr/_inbox/<timestamp>-<hex8>.md`:

```markdown
---
session: brainstorm
captured: 2026-05-14
---

## Inbox count: endpoint choice

**Контекст:** Стартовий екран MLMaiL показує кількість листів у мітці INBOX.

**Рішення/Процедура/Факт:** Точне `messagesTotal` із `GET users/me/labels/INBOX`. HTTPS-виклик — у Rust-модулі `app/src-tauri/src/gmail/`, через спільний `auth::acquire_access_token`.

**Обґрунтування:** Один HTTP-запит, точне число (а не estimate), мінімальний scope (вже мається `gmail.modify`).

**Розглянуті альтернативи:**
- `users.getProfile` — рахує всю поштову скриньку, не лише INBOX. Відкинуто.
- `messages.list?labelIds=INBOX` + `resultSizeEstimate` — приблизне, не точне. Відкинуто.

**Зачіпає:** Auth Store, Login.vue, новий Rust-модуль gmail, i18n (`Http`/`Parse`).
```

- [ ] **Step 3: Commit**

```bash
git add docs/ci4/decisions.md docs/adr/_inbox/
git commit -m "$(cat <<'EOF'
docs: ADR-inbox + decisions для inbox count рішення

Фіксує вибір endpoint (users.labels.get?id=INBOX) у C4-моделі і ADR-inbox.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Final verification

- [ ] **Step 1: Run full Rust test suite**

Run: `cd app/src-tauri && cargo test`
Expected: усі PASS, без warnings.

- [ ] **Step 2: Run full JS test suite**

Run: `bun --cwd app run test`
Expected: усі PASS.

- [ ] **Step 3: Run repo-wide lint**

Run: `bun run lint`
Expected: clean exit (без порушень).

- [ ] **Step 4: Run Rust build (release path) to make sure new module compiles for both targets**

Run: `cd app/src-tauri && cargo build`
Expected: SUCCESS.

(Android build не запускаємо в цьому плані — `cargo build` достатній, бо нові файли — pure Rust без cfg-targets.)

- [ ] **Step 5: Smoke test — manual checklist**

(виконує користувач на реальному акаунті — план не може це зробити за нього):

1. `bun --cwd app run tauri dev` на macOS;
2. натиснути «Увійти через Google» → пройти OAuth;
3. побачити «Ви увійшли як …» і нижче «Листів у скриньці: N», де N відповідає реальному числу повідомлень у Gmail web (`mail.google.com/mail/u/0/#inbox` нижній правий лічильник «1–50 of N»).
4. натиснути «Вийти» → екран повертається до «Увійти через Google»;
5. знову залогінитися → число знову зʼявляється.

Поведінка під час помилки (вимкнути мережу перед `tauri dev`):
- після логіну має зʼявитись «Не вдалося з'єднатися з Google. Перевірте мережу.» замість числа.

---

## Self-Review

**Spec coverage check (читаючи спеку `2026-05-14-inbox-count-design.md` поряд):**

| Spec requirement | Covered by |
| --- | --- |
| Endpoint `users/me/labels/INBOX` | Task 3 (`GMAIL_LABEL_INBOX_URL`) |
| `messagesTotal` як єдина цифра | Task 3 (`parse_messages_total`) |
| HTTPS-виклик у Rust | Task 3 (`fetch_inbox_count_at`) |
| Refresh-on-demand через спільний helper | Task 1 (`acquire_access_token`) |
| 401 → `ReauthRequired` | Task 3 (status mapping test) |
| Non-2xx → `Http{status,body}` | Task 3 (5xx + 403 tests) |
| JSON parse fail → `Parse` | Task 3 (parse test) |
| `inboxCount` / `inboxErrorKind` у store | Task 5 |
| Auto-fetch у `initialize()` і `login()` | Task 5 (тести покривають обидва) |
| `logout()` ресетить count і error | Task 5 (тест logout) |
| `ReauthRequired` від Gmail знімає логін | Task 5 (тест ReauthRequired) |
| UI: «Листів у скриньці: N» / плейсхолдер / помилка | Task 7 (три тести) |
| Українські повідомлення `Http`, `Parse` | Task 6 |
| `Network` повторно використовується (вже є) | Task 6 (без змін рядка) |
| Docs: `03-components.md` | Task 8 |
| Docs: `04-code.md` | Task 9 |
| Docs: `decisions.md` + ADR-inbox | Task 10 |
| Поза scope: лічильник unread, кнопка оновити, кеш | не реалізуємо (свідомо) |

Прогалин не знайдено.

**Type consistency:** ref-и іменуються однаково в усіх задачах (`inboxCount`, `inboxErrorKind`); метод — `refreshInboxCount`; команда — `gmail_inbox_count`; Rust-енам — `GmailError`. Хелпер — `acquire_access_token`. Все звірено.

**Placeholders:** жодних TBD / TODO. Усі тестові тіла і шаблони — повні.
