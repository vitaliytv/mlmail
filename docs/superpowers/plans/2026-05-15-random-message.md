# Random INBOX Message on Start Screen Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Показати plain-text вміст випадкового листа з INBOX поряд із числом листів на стартовому екрані MLMaiL і дати кнопку «Показати інший».

**Architecture:** Розширюємо існуючий Rust-модуль `gmail/` новою командою `gmail_random_message`: дзвонить `messages.list?labelIds=INBOX&maxResults=100&fields=messages/id`, бере випадковий id, дзвонить `messages.get?format=full`, парсить headers + body. Body extraction живе у новому файлі `gmail/message.rs` (рекурсивний обхід `payload.parts`, перевага `text/plain`, fallback на `text/html` зі стрипом тегів). Auth Store отримує `currentMessage`, `messageErrorKind`, `isMessageLoading` і метод `loadRandomMessage()`. `Login.vue` рендерить картку з листом і кнопку «Показати інший».

**Tech Stack:** Rust + Tauri 2, `reqwest` 0.12, `rand` 0.9, `base64` 0.22 (вже є), нові: `regex = "1"`, `html-escape = "0.2"`. Vue 3 + Vitest + `@vue/test-utils`.

**Spec:** [docs/superpowers/specs/2026-05-15-random-message-design.md](../specs/2026-05-15-random-message-design.md)

---

## File Structure

**Rust (новий і змінений):**

- Modify: `app/src-tauri/Cargo.toml` — додати `regex = "1"`, `html-escape = "0.2"`.
- Create: `app/src-tauri/src/gmail/message.rs` — `GmailMessage` DTO, `extract_plain_text`, `extract_header`.
- Modify: `app/src-tauri/src/gmail/error.rs` — додати `GmailError::Empty`.
- Modify: `app/src-tauri/src/gmail/mod.rs` — додати `pub mod message;`, `list_inbox_ids_at`, `get_message_at`, Tauri-команду `gmail_random_message`.
- Modify: `app/src-tauri/src/lib.rs` — реєстрація `gmail::gmail_random_message`.

**Vue (змінений):**

- Modify: `app/src/services/auth-store.js` — `_currentMessage`, `_messageErrorKind`, `_isMessageLoading`, метод `loadRandomMessage`; виклики у `initialize`/`login`; скидання у `logout`/`_resetForTest`.
- Modify: `app/src/services/auth-store.test.js` — новий suite «random message».
- Modify: `app/src/views/Login.vue` — секція з карткою листа + кнопка «Показати інший».
- Modify: `app/src/views/Login.test.js` — тести на рендер картки, кнопку, Empty.
- Modify: `app/src/i18n/auth-errors.js` — додати ключ `Empty`.
- Modify: `app/src/i18n/auth-errors.test.js` — тест на новий ключ.

**Docs:**

- Modify: `docs/ci4/03-components.md` — оновити Auth Store, Gmail Module з новою командою.
- Modify: `docs/ci4/04-code.md` — секція `gmail/message.rs`, оновити список команд у `lib.rs`.
- Modify: `docs/ci4/decisions.md` — рішення про sample-space 100 останніх.
- Create: `docs/adr/_inbox/<timestamp>-random-message.md` — ADR-нотатка.
- Modify: `.cspell.json` — слова, які виникнуть під час лінту (додамо реактивно).

---

## Task 1: Add Rust dependencies for HTML→text fallback

**Files:**

- Modify: `app/src-tauri/Cargo.toml`

- [ ] **Step 1: Add `regex` and `html-escape` to dependencies**

Append to the `[dependencies]` section of `app/src-tauri/Cargo.toml` (after `dotenvy = "0.15.7"`):

```toml
regex = "1"
html-escape = "0.2"
```

- [ ] **Step 2: Build to confirm crates resolve**

Run: `cd app/src-tauri && cargo build`
Expected: SUCCESS (downloads/compiles `regex` + `html-escape`).

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/Cargo.toml app/src-tauri/Cargo.lock
git commit -m "$(cat <<'EOF'
feat(deps): add regex + html-escape for HTML→text fallback in gmail

Use case: коли в листі немає text/plain part, треба перетворити text/html
у plain text для відображення на стартовому екрані.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add `GmailError::Empty` variant (TDD)

**Files:**

- Modify: `app/src-tauri/src/gmail/error.rs`

- [ ] **Step 1: Append failing test**

Append inside the existing `#[cfg(test)] mod tests` block in `app/src-tauri/src/gmail/error.rs`:

```rust
    #[test]
    fn empty_serializes_with_tagged_kind() {
        let e = GmailError::Empty;
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("\"kind\":\"Empty\""), "serialized: {s}");
    }
```

- [ ] **Step 2: Run test — expect failure**

Run: `cd app/src-tauri && cargo test --lib gmail::error::tests::empty_serializes_with_tagged_kind`
Expected: COMPILE ERROR (variant `Empty` does not exist).

- [ ] **Step 3: Add the variant**

Modify the `GmailError` enum in `app/src-tauri/src/gmail/error.rs` — append a new variant before the closing `}`:

```rust
    #[error("inbox is empty")]
    Empty,
```

The full enum now reads:

```rust
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
    #[error("inbox is empty")]
    Empty,
}
```

- [ ] **Step 4: Run test — expect pass**

Run: `cd app/src-tauri && cargo test --lib gmail::error`
Expected: усі 4 тести PASS.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/gmail/error.rs
git commit -m "$(cat <<'EOF'
feat(gmail): add GmailError::Empty for empty INBOX

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Create `gmail/message.rs` with `GmailMessage` DTO + `extract_header` (TDD)

**Files:**

- Create: `app/src-tauri/src/gmail/message.rs`

- [ ] **Step 1: Create the file with DTO + header helper + failing tests**

Create `app/src-tauri/src/gmail/message.rs`:

```rust
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GmailMessage {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub body: String,
}

pub fn extract_header(headers: &[Value], name: &str) -> String {
    let target = name.to_ascii_lowercase();
    for h in headers {
        let key = h.get("name").and_then(Value::as_str).unwrap_or("");
        if key.to_ascii_lowercase() == target {
            return h
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_header_finds_case_insensitive() {
        let headers = vec![
            json!({"name": "From", "value": "alice@example.com"}),
            json!({"name": "Subject", "value": "Hi"}),
        ];
        assert_eq!(extract_header(&headers, "from"), "alice@example.com");
        assert_eq!(extract_header(&headers, "SUBJECT"), "Hi");
    }

    #[test]
    fn extract_header_returns_empty_when_missing() {
        let headers = vec![json!({"name": "From", "value": "a@b"})];
        assert_eq!(extract_header(&headers, "Date"), "");
    }

    #[test]
    fn extract_header_returns_empty_for_empty_list() {
        assert_eq!(extract_header(&[], "From"), "");
    }
}
```

- [ ] **Step 2: Register module so cargo picks up tests**

Edit `app/src-tauri/src/gmail/mod.rs`. Find the first line `pub mod error;` and append:

```rust
pub mod error;
pub mod message;
```

- [ ] **Step 3: Run tests — expect pass**

Run: `cd app/src-tauri && cargo test --lib gmail::message`
Expected: 3 PASS.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/gmail/message.rs app/src-tauri/src/gmail/mod.rs
git commit -m "$(cat <<'EOF'
feat(gmail): add GmailMessage DTO + extract_header helper

Готує DTO для нової команди gmail_random_message. extract_header — це
case-insensitive пошук серед payload.headers (Gmail API віддає масив
обʼєктів {name, value}, регістр не нормалізований).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Implement `extract_plain_text` (TDD)

**Files:**

- Modify: `app/src-tauri/src/gmail/message.rs`

- [ ] **Step 1: Append failing tests for `extract_plain_text`**

Add inside the existing `#[cfg(test)] mod tests` block of `app/src-tauri/src/gmail/message.rs`, after the last `#[test]`:

```rust
    fn b64url(s: &str) -> String {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(s.as_bytes())
    }

    #[test]
    fn extract_plain_text_from_plain_payload() {
        let payload = json!({
            "mimeType": "text/plain",
            "body": { "data": b64url("hello world") }
        });
        assert_eq!(extract_plain_text(&payload), "hello world");
    }

    #[test]
    fn extract_plain_text_from_html_payload_strips_tags() {
        let html = "<p>Hello <b>world</b>!</p>";
        let payload = json!({
            "mimeType": "text/html",
            "body": { "data": b64url(html) }
        });
        assert_eq!(extract_plain_text(&payload), "Hello world!");
    }

    #[test]
    fn extract_plain_text_decodes_html_entities() {
        let html = "Hello &amp; bye &lt;3";
        let payload = json!({
            "mimeType": "text/html",
            "body": { "data": b64url(html) }
        });
        assert_eq!(extract_plain_text(&payload), "Hello & bye <3");
    }

    #[test]
    fn extract_plain_text_prefers_plain_in_alternative() {
        let payload = json!({
            "mimeType": "multipart/alternative",
            "parts": [
                {"mimeType": "text/plain", "body": {"data": b64url("plain version")}},
                {"mimeType": "text/html", "body": {"data": b64url("<b>html version</b>")}}
            ]
        });
        assert_eq!(extract_plain_text(&payload), "plain version");
    }

    #[test]
    fn extract_plain_text_falls_back_to_html_when_no_plain() {
        let payload = json!({
            "mimeType": "multipart/alternative",
            "parts": [
                {"mimeType": "text/html", "body": {"data": b64url("<p>only html</p>")}}
            ]
        });
        assert_eq!(extract_plain_text(&payload), "only html");
    }

    #[test]
    fn extract_plain_text_recurses_into_nested_multipart() {
        let payload = json!({
            "mimeType": "multipart/mixed",
            "parts": [
                {
                    "mimeType": "multipart/alternative",
                    "parts": [
                        {"mimeType": "text/plain", "body": {"data": b64url("nested plain")}}
                    ]
                }
            ]
        });
        assert_eq!(extract_plain_text(&payload), "nested plain");
    }

    #[test]
    fn extract_plain_text_returns_empty_when_no_text_part() {
        let payload = json!({
            "mimeType": "multipart/mixed",
            "parts": [
                {"mimeType": "image/png", "body": {"data": b64url("BINARY")}}
            ]
        });
        assert_eq!(extract_plain_text(&payload), "");
    }

    #[test]
    fn extract_plain_text_returns_empty_for_missing_data() {
        let payload = json!({"mimeType": "text/plain"});
        assert_eq!(extract_plain_text(&payload), "");
    }
```

- [ ] **Step 2: Run — expect compile failure**

Run: `cd app/src-tauri && cargo test --lib gmail::message`
Expected: COMPILE ERROR — function `extract_plain_text` not found.

- [ ] **Step 3: Implement `extract_plain_text`**

Insert after the `extract_header` function in `app/src-tauri/src/gmail/message.rs`:

```rust
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub fn extract_plain_text(payload: &Value) -> String {
    if let Some(text) = find_part(payload, "text/plain") {
        return text;
    }
    if let Some(html) = find_part(payload, "text/html") {
        return strip_html(&html);
    }
    String::new()
}

fn find_part(node: &Value, target_mime: &str) -> Option<String> {
    let mime = node
        .get("mimeType")
        .and_then(Value::as_str)
        .unwrap_or("");
    if mime == target_mime {
        if let Some(data) = node.get("body").and_then(|b| b.get("data")).and_then(Value::as_str) {
            return decode_base64url(data);
        }
        return None;
    }
    if let Some(parts) = node.get("parts").and_then(Value::as_array) {
        for p in parts {
            if let Some(found) = find_part(p, target_mime) {
                return Some(found);
            }
        }
    }
    None
}

fn decode_base64url(data: &str) -> Option<String> {
    let bytes = URL_SAFE_NO_PAD.decode(data.as_bytes()).ok()?;
    String::from_utf8(bytes).ok()
}

fn strip_html(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").expect("static regex");
    let stripped = re.replace_all(html, "");
    let decoded = html_escape::decode_html_entities(&stripped);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

Note: `String::from_utf8` will fail for invalid UTF-8 — for the MVP we treat that as "no body" (`None` flows up). The `extract_plain_text` test «empty for missing data» also covers this implicitly.

Also: `strip_html` collapses whitespace via `split_whitespace().join(" ")` — that normalises tags-induced newlines into single spaces so tests with `<p>Hello <b>world</b>!</p>` → `Hello world!` work.

- [ ] **Step 4: Run tests — expect all pass**

Run: `cd app/src-tauri && cargo test --lib gmail::message`
Expected: 11 PASS (3 existing + 8 new).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/gmail/message.rs
git commit -m "$(cat <<'EOF'
feat(gmail): extract_plain_text from MIME payload

Рекурсивно обходить payload.parts, перевага text/plain над text/html.
Для html fallback: strip тегів regex + decode_html_entities + collapse
whitespace. base64url decode як у Gmail API.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Add `list_inbox_ids_at` HTTP helper to `gmail/mod.rs` (TDD)

**Files:**

- Modify: `app/src-tauri/src/gmail/mod.rs`

- [ ] **Step 1: Append failing tests for `list_inbox_ids_at`**

Inside the existing `#[cfg(test)] mod tests` block of `app/src-tauri/src/gmail/mod.rs`, append:

```rust
    #[tokio::test]
    async fn list_inbox_ids_returns_ids_on_200() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("labelIds".into(), "INBOX".into()),
                mockito::Matcher::UrlEncoded("maxResults".into(), "100".into()),
                mockito::Matcher::UrlEncoded("fields".into(), "messages/id".into()),
            ]))
            .match_header("authorization", "Bearer AT-1")
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"a"},{"id":"b"},{"id":"c"}]}"#)
            .create_async()
            .await;

        let ids = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn list_inbox_ids_returns_empty_when_no_messages_field() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .with_status(200)
            .with_body(r#"{"resultSizeEstimate":0}"#)
            .create_async()
            .await;

        let ids = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn list_inbox_ids_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .with_status(401)
            .with_body("nope")
            .create_async()
            .await;

        let err = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn list_inbox_ids_maps_5xx_to_http() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .with_status(503)
            .with_body("boom")
            .create_async()
            .await;

        let err = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
    }
```

- [ ] **Step 2: Run — expect compile failure**

Run: `cd app/src-tauri && cargo test --lib gmail`
Expected: COMPILE ERROR — `list_inbox_ids_at` not found.

- [ ] **Step 3: Implement helper**

Add after `fetch_inbox_count_at` in `app/src-tauri/src/gmail/mod.rs`:

```rust
pub const GMAIL_MESSAGES_LIST_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/messages";

pub(crate) async fn list_inbox_ids_at(
    endpoint: &str,
    access_token: &str,
) -> Result<Vec<String>, GmailError> {
    let resp = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .query(&[
            ("labelIds", "INBOX"),
            ("maxResults", "100"),
            ("fields", "messages/id"),
        ])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| GmailError::Parse(e.to_string()))?;
    let arr = match v.get("messages").and_then(|m| m.as_array()) {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };
    let mut ids = Vec::with_capacity(arr.len());
    for m in arr {
        let id = m
            .get("id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| GmailError::Parse("message without id".into()))?;
        ids.push(id.to_string());
    }
    Ok(ids)
}
```

- [ ] **Step 4: Run gmail tests — expect pass**

Run: `cd app/src-tauri && cargo test --lib gmail`
Expected: усі тести PASS (11 нових із Task 4 + 8 inbox_count + 3 error + 4 нових list).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/gmail/mod.rs
git commit -m "$(cat <<'EOF'
feat(gmail): list_inbox_ids_at helper for messages.list

Тонкий wrapper над GET messages?labelIds=INBOX&maxResults=100&fields=messages/id.
Повертає Vec<String> з id-ами або порожній вектор для resultSizeEstimate=0.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Add `get_message_at` HTTP helper (TDD)

**Files:**

- Modify: `app/src-tauri/src/gmail/mod.rs`

- [ ] **Step 1: Append failing tests**

Add to the same `#[cfg(test)] mod tests` block:

```rust
    #[tokio::test]
    async fn get_message_returns_parsed_gmail_message() {
        use base64::Engine;
        let body_data = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"hello body");
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages/m1")
            .match_query(mockito::Matcher::UrlEncoded("format".into(), "full".into()))
            .match_header("authorization", "Bearer AT-1")
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "id": "m1",
                    "payload": {
                        "mimeType": "text/plain",
                        "headers": [
                            {"name": "From",    "value": "alice@example.com"},
                            {"name": "Subject", "value": "Greetings"},
                            {"name": "Date",    "value": "Mon, 15 May 2026 10:00:00 +0300"}
                        ],
                        "body": {"data": body_data}
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let msg = get_message_at(&format!("{}/messages", server.url()), "AT-1", "m1")
            .await
            .unwrap();
        assert_eq!(msg.id, "m1");
        assert_eq!(msg.from, "alice@example.com");
        assert_eq!(msg.subject, "Greetings");
        assert_eq!(msg.date, "Mon, 15 May 2026 10:00:00 +0300");
        assert_eq!(msg.body, "hello body");
    }

    #[tokio::test]
    async fn get_message_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages/m1")
            .with_status(401)
            .with_body("nope")
            .create_async()
            .await;

        let err = get_message_at(&format!("{}/messages", server.url()), "AT-1", "m1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn get_message_truncates_long_body() {
        use base64::Engine;
        let long = "x".repeat(11_000);
        let data = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(long.as_bytes());
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages/m1")
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "id": "m1",
                    "payload": {
                        "mimeType": "text/plain",
                        "headers": [],
                        "body": {"data": data}
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let msg = get_message_at(&format!("{}/messages", server.url()), "AT-1", "m1")
            .await
            .unwrap();
        assert_eq!(msg.body.chars().count(), 10_000);
    }
```

- [ ] **Step 2: Run — expect compile failure**

Run: `cd app/src-tauri && cargo test --lib gmail`
Expected: COMPILE ERROR — `get_message_at` not found.

- [ ] **Step 3: Implement helper**

Add after `list_inbox_ids_at` in `app/src-tauri/src/gmail/mod.rs`:

```rust
use crate::gmail::message::{extract_header, extract_plain_text, GmailMessage};

pub(crate) async fn get_message_at(
    base_endpoint: &str,
    access_token: &str,
    id: &str,
) -> Result<GmailMessage, GmailError> {
    let resp = reqwest::Client::new()
        .get(format!("{base_endpoint}/{id}"))
        .bearer_auth(access_token)
        .query(&[("format", "full")])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| GmailError::Parse(e.to_string()))?;
    let payload = v
        .get("payload")
        .ok_or_else(|| GmailError::Parse("message has no payload".into()))?;
    let empty_headers: Vec<serde_json::Value> = Vec::new();
    let headers = payload
        .get("headers")
        .and_then(|h| h.as_array())
        .unwrap_or(&empty_headers);

    let body_text = extract_plain_text(payload);
    let body_truncated: String = body_text.chars().take(10_000).collect();

    Ok(GmailMessage {
        id: id.to_string(),
        from: extract_header(headers, "From"),
        subject: extract_header(headers, "Subject"),
        date: extract_header(headers, "Date"),
        body: body_truncated,
    })
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `cd app/src-tauri && cargo test --lib gmail`
Expected: усі PASS (попередні + 3 нові).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/gmail/mod.rs
git commit -m "$(cat <<'EOF'
feat(gmail): get_message_at fetches and parses a single message

GET messages/<id>?format=full → витягає headers (From/Subject/Date) і
тіло через extract_plain_text. Truncates body to 10K chars.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Wire `gmail_random_message` Tauri command

**Files:**

- Modify: `app/src-tauri/src/gmail/mod.rs`
- Modify: `app/src-tauri/src/lib.rs`

- [ ] **Step 1: Add command to `gmail/mod.rs`**

Append at the end of `app/src-tauri/src/gmail/mod.rs` (after the existing `gmail_inbox_count` function, before the `#[cfg(test)] mod tests` block):

```rust
#[tauri::command]
pub async fn gmail_random_message(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<GmailMessage, GmailError> {
    let token = auth::acquire_access_token(&app, &state).await?;
    let ids = list_inbox_ids_at(GMAIL_MESSAGES_LIST_URL, &token).await?;
    if ids.is_empty() {
        return Err(GmailError::Empty);
    }
    let i = rand::random::<u64>() as usize % ids.len();
    get_message_at(GMAIL_MESSAGES_LIST_URL, &token, &ids[i]).await
}
```

(`rand::random::<u64>()` — без проблем on Send-bound; `rand::random::<usize>()` працює так само, але `u64` явно безпечніший cross-platform.)

- [ ] **Step 2: Register the command**

Modify `app/src-tauri/src/lib.rs`. Add `gmail::gmail_random_message` to `generate_handler!`:

```rust
        .invoke_handler(tauri::generate_handler![
            auth::auth_start_login,
            auth::auth_get_access_token,
            auth::auth_is_authenticated,
            auth::auth_current_email,
            auth::auth_logout,
            gmail::gmail_inbox_count,
            gmail::gmail_random_message,
        ])
```

- [ ] **Step 3: Build + test**

Run: `cd app/src-tauri && cargo build && cargo test --lib`
Expected: SUCCESS, усі тести PASS.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/gmail/mod.rs app/src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(gmail): wire gmail_random_message into Tauri handlers

Тепер фронт може кликати invoke('gmail_random_message') — повертає
GmailMessage або GmailError::Empty/ReauthRequired/Http/Parse/Network.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Extend `auth-store.js` with random message state (TDD)

**Files:**

- Modify: `app/src/services/auth-store.js`
- Modify: `app/src/services/auth-store.test.js`

- [ ] **Step 1: Append failing tests to `auth-store.test.js`**

Append at the bottom of `app/src/services/auth-store.test.js`:

```js
describe('useAuthStore random message', () => {
  const sampleMessage = { id: 'm1', from: 'a@e', subject: 's', date: 'd', body: 'b' }

  it('initialize loads currentMessage after successful auth', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value).toEqual(sampleMessage)
    expect(store.messageErrorKind.value).toBe(null)
    expect(store.isMessageLoading.value).toBe(false)
  })

  it('login also loads currentMessage', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_start_login') return Promise.resolve({ email: 'u@e' })
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.login()
    expect(store.currentMessage.value).toEqual(sampleMessage)
  })

  it('loadRandomMessage replaces currentMessage', async () => {
    let call = 0
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') {
        call += 1
        return Promise.resolve({ ...sampleMessage, id: `m${call}` })
      }
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value.id).toBe('m1')
    await store.loadRandomMessage()
    expect(store.currentMessage.value.id).toBe('m2')
  })

  it('captures Empty kind from Gmail', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(0)
      if (cmd === 'gmail_random_message')
        return Promise.reject(Object.assign(new Error('Empty'), { kind: 'Empty' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value).toBe(null)
    expect(store.messageErrorKind.value).toBe('Empty')
  })

  it('ReauthRequired from random message forces logout state', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message')
        return Promise.reject(Object.assign(new Error('ReauthRequired'), { kind: 'ReauthRequired' }))
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.isAuthenticated.value).toBe(false)
    expect(store.email.value).toBe(null)
    expect(store.currentMessage.value).toBe(null)
  })

  it('logout clears currentMessage and messageErrorKind', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      if (cmd === 'auth_logout') return Promise.resolve()
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(store.currentMessage.value).toEqual(sampleMessage)
    await store.logout()
    expect(store.currentMessage.value).toBe(null)
    expect(store.messageErrorKind.value).toBe(null)
    expect(store.isMessageLoading.value).toBe(false)
  })

  it('does not call gmail_random_message when not authenticated', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(false)
      return Promise.resolve(null)
    })
    const store = useAuthStore()
    await store.initialize()
    expect(invokeMock).not.toHaveBeenCalledWith('gmail_random_message')
    expect(store.currentMessage.value).toBe(null)
  })
})
```

- [ ] **Step 2: Run tests — expect failures**

Run: `cd app && bun run test auth-store`
Expected: 7 нових FAIL (`store.currentMessage` undefined).

- [ ] **Step 3: Update `auth-store.js`**

Replace the contents of `app/src/services/auth-store.js` with:

```js
import { invoke } from '@tauri-apps/api/core'
import { readonly, ref } from 'vue'

const _email = ref(null)
const _isAuthenticated = ref(false)
const _isLoading = ref(false)
const _errorKind = ref(null)
const _inboxCount = ref(null)
const _inboxErrorKind = ref(null)
const _currentMessage = ref(null)
const _messageErrorKind = ref(null)
const _isMessageLoading = ref(false)

/**
 * @returns {Promise<string>} access token
 */
function getAccessToken() {
  return invoke('auth_get_access_token')
}

/**
 * @returns {object} auth store
 */
export function useAuthStore() {
  /**
   *
   */
  async function refreshInboxCount() {
    if (!_isAuthenticated.value) return
    try {
      _inboxCount.value = await invoke('gmail_inbox_count')
      _inboxErrorKind.value = null
    } catch (error) {
      const kind = error && typeof error === 'object' && error.kind ? error.kind : 'Unknown'
      _inboxCount.value = null
      _inboxErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    }
  }

  /**
   *
   */
  async function loadRandomMessage() {
    if (!_isAuthenticated.value) return
    _isMessageLoading.value = true
    _messageErrorKind.value = null
    try {
      _currentMessage.value = await invoke('gmail_random_message')
    } catch (error) {
      const kind = error && typeof error === 'object' && error.kind ? error.kind : 'Unknown'
      _currentMessage.value = null
      _messageErrorKind.value = kind
      if (kind === 'ReauthRequired') {
        _email.value = null
        _isAuthenticated.value = false
      }
    } finally {
      _isMessageLoading.value = false
    }
  }

  /**
   *
   */
  async function initialize() {
    const ok = await invoke('auth_is_authenticated')
    _isAuthenticated.value = ok
    if (ok) {
      _email.value = await invoke('auth_current_email')
      await refreshInboxCount()
      await loadRandomMessage()
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
      await refreshInboxCount()
      await loadRandomMessage()
    } catch (error) {
      _errorKind.value = error && typeof error === 'object' && error.kind ? error.kind : 'Unknown'
    } finally {
      _isLoading.value = false
    }
  }

  /**
   *
   */
  async function logout() {
    await invoke('auth_logout')
    _email.value = null
    _isAuthenticated.value = false
    _errorKind.value = null
    _inboxCount.value = null
    _inboxErrorKind.value = null
    _currentMessage.value = null
    _messageErrorKind.value = null
    _isMessageLoading.value = false
  }

  return {
    email: readonly(_email),
    isAuthenticated: readonly(_isAuthenticated),
    isLoading: readonly(_isLoading),
    errorKind: readonly(_errorKind),
    inboxCount: readonly(_inboxCount),
    inboxErrorKind: readonly(_inboxErrorKind),
    currentMessage: readonly(_currentMessage),
    messageErrorKind: readonly(_messageErrorKind),
    isMessageLoading: readonly(_isMessageLoading),
    initialize,
    login,
    getAccessToken,
    logout,
    refreshInboxCount,
    loadRandomMessage
  }
}

/**
 *
 */
export function _resetForTest() {
  _email.value = null
  _isAuthenticated.value = false
  _isLoading.value = false
  _errorKind.value = null
  _inboxCount.value = null
  _inboxErrorKind.value = null
  _currentMessage.value = null
  _messageErrorKind.value = null
  _isMessageLoading.value = false
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `bun --cwd app run test`
Expected: усі PASS (попередні + 7 нових).

- [ ] **Step 5: Commit**

```bash
git add app/src/services/auth-store.js app/src/services/auth-store.test.js
git commit -m "$(cat <<'EOF'
feat(auth-store): track random INBOX message via gmail_random_message

Додає currentMessage/messageErrorKind/isMessageLoading + loadRandomMessage.
Auto-fetch після initialize/login. ReauthRequired від Gmail знімає логін.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Add `Empty` kind to i18n (TDD)

**Files:**

- Modify: `app/src/i18n/auth-errors.js`
- Modify: `app/src/i18n/auth-errors.test.js`

- [ ] **Step 1: Append failing test**

Append inside `describe('errorMessage Gmail kinds', () => { ... })` block of `app/src/i18n/auth-errors.test.js` (right before its closing `})`):

```js
  it('returns Ukrainian text for Empty kind', () => {
    expect(errorMessage('Empty')).toBe('Скринька порожня.')
  })
```

- [ ] **Step 2: Run — expect failure**

Run: `bun --cwd app run test auth-errors`
Expected: FAIL (returns "Невідома помилка.").

- [ ] **Step 3: Add the message**

Edit `app/src/i18n/auth-errors.js`. Insert after the `Parse:` line in the `messages` object:

```js
  Http: 'Gmail повернув помилку. Спробуйте пізніше.',
  Parse: 'Несподівана відповідь від Gmail.',
  Empty: 'Скринька порожня.',
  Unknown: 'Невідома помилка.'
```

- [ ] **Step 4: Run — expect pass**

Run: `bun --cwd app run test auth-errors`
Expected: усі PASS.

- [ ] **Step 5: Commit**

```bash
git add app/src/i18n/auth-errors.js app/src/i18n/auth-errors.test.js
git commit -m "$(cat <<'EOF'
feat(i18n): add Ukrainian message for Gmail Empty kind

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Render random message card on `Login.vue` (TDD)

**Files:**

- Modify: `app/src/views/Login.vue`
- Modify: `app/src/views/Login.test.js`

- [ ] **Step 1: Append failing tests**

Append at the bottom of `app/src/views/Login.test.js`:

```js
describe('Login.vue random message', () => {
  const sampleMessage = {
    id: 'm1',
    from: 'alice@example.com',
    subject: 'Greetings',
    date: 'Mon, 15 May 2026 10:00:00 +0300',
    body: 'hello body'
  }

  it('renders the message card after initialize', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('alice@example.com')
    expect(w.text()).toContain('Greetings')
    expect(w.text()).toContain('Mon, 15 May 2026 10:00:00 +0300')
    expect(w.text()).toContain('hello body')
  })

  it('shows "Скринька порожня." when Gmail returns Empty', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(0)
      if (cmd === 'gmail_random_message')
        return Promise.reject(Object.assign(new Error('Empty'), { kind: 'Empty' }))
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    expect(w.text()).toContain('Скринька порожня.')
  })

  it('clicking "Показати інший" re-invokes gmail_random_message', async () => {
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'auth_is_authenticated') return Promise.resolve(true)
      if (cmd === 'auth_current_email') return Promise.resolve('u@e')
      if (cmd === 'gmail_inbox_count') return Promise.resolve(5)
      if (cmd === 'gmail_random_message') return Promise.resolve(sampleMessage)
      return Promise.resolve(null)
    })
    const w = mount(Login)
    await flushPromises()
    invokeMock.mockClear()
    invokeMock.mockImplementation((cmd) => {
      if (cmd === 'gmail_random_message')
        return Promise.resolve({ ...sampleMessage, id: 'm2', subject: 'Next one' })
      return Promise.resolve(null)
    })
    const btn = w.findAll('button').find((b) => b.text() === 'Показати інший')
    await btn.trigger('click')
    await flushPromises()
    expect(invokeMock).toHaveBeenCalledWith('gmail_random_message')
    expect(w.text()).toContain('Next one')
  })
})
```

- [ ] **Step 2: Run — expect failures**

Run: `bun --cwd app run test Login`
Expected: 3 нові FAIL (елементи не існують).

- [ ] **Step 3: Update `Login.vue`**

Open `app/src/views/Login.vue`. Inside the `<div v-if="auth.isAuthenticated.value" class="signed-in">` block, after the inbox-count `<p>` lines (the `<p v-else class="inbox-count muted">Листів у скриньці: …</p>` line) and **before** the `<button type="button" @click="auth.logout()">Вийти</button>`, insert:

```vue
      <section v-if="auth.currentMessage.value" class="message">
        <header class="message-head">
          <p><strong>Від:</strong> {{ auth.currentMessage.value.from }}</p>
          <p><strong>Тема:</strong> {{ auth.currentMessage.value.subject }}</p>
          <p><strong>Дата:</strong> {{ auth.currentMessage.value.date }}</p>
        </header>
        <pre class="message-body">{{ auth.currentMessage.value.body }}</pre>
      </section>
      <p v-else-if="auth.isMessageLoading.value" class="muted">Завантаження…</p>
      <p v-else-if="auth.messageErrorKind.value" class="error">
        {{ errorMessage(auth.messageErrorKind.value) }}
      </p>
      <button
        type="button"
        :disabled="auth.isMessageLoading.value"
        @click="auth.loadRandomMessage()"
      >
        Показати інший
      </button>
```

Append CSS rules to the `<style scoped>` block (before the closing `</style>`):

```css
.message {
  max-width: 60ch;
  text-align: left;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 0.8em 1em;
  background: rgba(0, 0, 0, 0.02);
}

.message-head p {
  margin: 0.1em 0;
}

.message-body {
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0.6em 0 0;
  font-family: inherit;
}

.muted {
  opacity: 0.6;
}
```

- [ ] **Step 4: Run — expect pass**

Run: `bun --cwd app run test`
Expected: усі PASS.

- [ ] **Step 5: Commit**

```bash
git add app/src/views/Login.vue app/src/views/Login.test.js
git commit -m "$(cat <<'EOF'
feat(login): render random INBOX message + «Показати інший» button

Картка під лічильником: Від/Тема/Дата + тіло у <pre> для збереження
переносів. Кнопка тригерить loadRandomMessage(). На Empty показуємо
«Скринька порожня.».

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Update `docs/ci4/03-components.md`

**Files:**

- Modify: `docs/ci4/03-components.md`

- [ ] **Step 1: Update Auth Store description**

In `docs/ci4/03-components.md`, find the «Компонент Auth Store MLMaiL (implemented)» section. Replace the existing `**лише UI-стан** (...)` paragraph with:

```
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
```

- [ ] **Step 2: Update Auth Component description**

Replace the UI-гілки list (the `- авторизовано → ...` bullet) inside «Компонент Auth Component MLMaiL (implemented)» with:

```
- не авторизовано → кнопка "Увійти через Google" (заблокована поки триває
  логін, текст змінюється на "Зачекайте…");
- авторизовано → "Ви увійшли як {email}", рядок "Листів у скриньці: N",
  картка випадкового листа (Від/Тема/Дата + тіло у `<pre>`) і кнопка
  "Показати інший", яка обирає інший випадковий лист; внизу кнопка "Вийти";
- помилка останньої спроби логіну — український рядок з Auth Errors i18n MLMaiL.
```

- [ ] **Step 3: Update Auth Errors i18n description**

Replace the «Девʼять ключів (...)» list in the Auth Errors i18n section with:

```
Десять ключів (`Cancelled`, `Network`, `OAuth`, `Storage`, `ReauthRequired`,
`Platform`, `Http`, `Parse`, `Empty`, `Unknown`) — `errorMessage(kind)`
повертає українську строку або `"Невідома помилка."` для невідомих kind.
`Http`/`Parse`/`Empty` — для помилок Gmail-шару.
```

- [ ] **Step 4: Update Gmail Module description**

Inside «Компонент Gmail Module MLMaiL (implemented)»:

- Append a second bullet after `gmail_inbox_count` bullet:

```
- `gmail_random_message() -> Result<GmailMessage, GmailError>` — обирає
  випадковий id серед перших 100 листів INBOX (`messages.list?maxResults=100`),
  тягне `messages.get?format=full`, парсить headers (From/Subject/Date) і body
  (через `extract_plain_text`). На порожню скриньку повертає `GmailError::Empty`.
```

- Append a row to the subcomponents table:

```
| `message.rs` | `GmailMessage` DTO (`id, from, subject, date, body`), `extract_header` (case-insensitive header lookup), `extract_plain_text` (рекурсивний обхід `payload.parts` з пріоритетом `text/plain` і fallback на `text/html` зі стрипом тегів через `regex` + `html-escape`) |
```

- Update the test count line — replace «11 unit-тестів» with «27 unit-тестів» (11 попередніх + 8 message + 4 list + 3 get + 1 empty error).

- [ ] **Step 5: Commit**

```bash
git add docs/ci4/03-components.md
git commit -m "$(cat <<'EOF'
docs(ci4): components — Auth Store / Gmail Module / i18n після random message

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Update `docs/ci4/04-code.md`

**Files:**

- Modify: `docs/ci4/04-code.md`

- [ ] **Step 1: Add `message.rs` to the gmail directory tree**

In the «Каталог [app/src-tauri/src/gmail/]» section, replace the existing tree:

```text
gmail/
├── mod.rs   — Tauri command gmail_inbox_count + fetch_inbox_count_at + parse_messages_total
└── error.rs — GmailError + From-конверсії з reqwest::Error і AuthError
```

with:

```text
gmail/
├── mod.rs     — Tauri commands gmail_inbox_count / gmail_random_message; HTTP helpers fetch_inbox_count_at / list_inbox_ids_at / get_message_at; parse_messages_total
├── message.rs — GmailMessage DTO + extract_header + extract_plain_text
└── error.rs   — GmailError + From-конверсії з reqwest::Error і AuthError
```

- [ ] **Step 2: Append description of `gmail_random_message`**

After the existing paragraph that describes `gmail_inbox_count`, append:

```
Команда `gmail_random_message(app, state) -> Result<GmailMessage, GmailError>` —
`list_inbox_ids_at` → випадковий id (через `rand::random`) → `get_message_at` →
`GmailMessage { id, from, subject, date, body }`. Body truncate до 10 000
символів. Порожній INBOX → `GmailError::Empty`. Status mapping ідентичний
`fetch_inbox_count_at`.

`extract_plain_text(payload)` рекурсивно обходить `payload.parts`, обираючи
`text/plain` (base64url-decoded). Fallback — `text/html` зі стрипом тегів
(`regex` `<[^>]+>`) і decode HTML entities (`html-escape`).
```

- [ ] **Step 3: Update lib.rs example to include the new command**

In the `lib.rs` rust block (the one inside «Файл [app/src-tauri/src/lib.rs]»), add `gmail::gmail_random_message,` line after `gmail::gmail_inbox_count,`:

```rust
            gmail::gmail_inbox_count,
            gmail::gmail_random_message,
```

Also add `pub mod gmail;` already exists — нічого міняти у `pub mod` секції.

- [ ] **Step 4: Update auth-store description**

Find the «Файл [app/src/services/auth-store.js]» section. Replace its body with:

```
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
```

- [ ] **Step 5: Update auth-errors description**

Replace the body of «Файл [app/src/i18n/auth-errors.js]» with:

```
Auth Errors i18n MLMaiL — словник `kind` → українська строка. Десять ключів
(`Cancelled`, `Network`, `OAuth`, `Storage`, `ReauthRequired`, `Platform`,
`Http`, `Parse`, `Empty`, `Unknown`), fallback `"Невідома помилка."`. Тести —
[auth-errors.test.js](../../app/src/i18n/auth-errors.test.js).
```

- [ ] **Step 6: Commit**

```bash
git add docs/ci4/04-code.md
git commit -m "$(cat <<'EOF'
docs(ci4): code-level — додати gmail/message.rs і команду gmail_random_message

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 13: Update `decisions.md` + ADR-inbox note

**Files:**

- Modify: `docs/ci4/decisions.md`
- Create: `docs/adr/_inbox/<timestamp>-random-message.md`

- [ ] **Step 1: Append decision to `decisions.md`**

In `docs/ci4/decisions.md`, after the «Рішення: Кількість листів у скриньці...» block (i.e. before «## Рішення, що очікують ADR для MLMaiL»), append:

```markdown
### Рішення: Випадковий лист на стартовому екрані — sample з перших 100

Закодовано у [app/src-tauri/src/gmail/](../../app/src-tauri/src/gmail/) і
[app/src/services/auth-store.js](../../app/src/services/auth-store.js).

Команда `gmail_random_message` обирає випадковий id серед перших 100 листів
INBOX (`messages.list?maxResults=100&fields=messages/id`). «Чесний» рандом
по всьому INBOX потребує курсор-пагінації (Gmail API не підтримує offset),
що для скриньки в 5K листів = 50+ послідовних викликів — overkill для UX
«покажи якийсь лист».

Plain-text body extraction: пріоритет `text/plain` part, fallback на
`text/html` зі стрипом тегів через `regex` + `html-escape`. HTML-рендер у
sandboxed iframe — окрема ітерація.

Вплив на C4-модель MLMaiL:

- [03-components.md](03-components.md) — Gmail Module MLMaiL отримав другу
  команду; Auth Store — `currentMessage`/`messageErrorKind`/`isMessageLoading`
  і метод `loadRandomMessage`; Auth Errors i18n — ключ `Empty`.
- [04-code.md](04-code.md) — нова секція для `gmail/message.rs`, оновлено
  список handler-ів у `lib.rs`.

ADR ще не оформлений; кандидат — `docs/adr/ADR-0008-random-message.md`.
Чернетка інбоксу — у `docs/adr/_inbox/`.
```

- [ ] **Step 2: Generate timestamp and create ADR note**

Run: `date +%Y%m%d-%H%M%S` and capture the value as `TS`.

Then create `docs/adr/_inbox/<TS>-random-message.md` with content:

```markdown
---
session: brainstorm
captured: 2026-05-15
---

## Random INBOX message: sample-space and body extraction

**Контекст:** Стартовий екран MLMaiL після логіну показує plain-text один випадковий лист з INBOX і кнопку «Показати інший».

**Рішення/Процедура/Факт:** Sample = перші 100 листів INBOX (`messages.list?maxResults=100&fields=messages/id`). Випадковий id обирається `rand::random::<u64>() as usize % ids.len()`. Лист береться `messages.get?id=<id>&format=full`. Body extraction: `text/plain` part першим, fallback `text/html` зі стрипом тегів (`regex` + `html-escape`). Body truncate до 10K символів. Порожній INBOX → `GmailError::Empty` → «Скринька порожня.» у UI.

**Обґрунтування:** Один HTTP-виклик для пошуку id (мінімальний JSON через `fields=messages/id`), потім ще один для тіла. Швидко, без додаткової квоти. Plain-text — без XSS-ризиків і без потреби в DOMPurify/iframe.

**Розглянуті альтернативи:**

- «Чесний» рандом серед усього INBOX через pageToken-пагінацію — 50+ послідовних викликів для звичайної скриньки. Відкинуто.
- `q=before:/after:` з рандомною датою — складно, ефекти rounding. Відкинуто.
- HTML-рендер тіла листа — потребує DOMPurify + sandboxed iframe; винесено в окрему ітерацію.

**Зачіпає:** `app/src-tauri/src/gmail/mod.rs`, новий `app/src-tauri/src/gmail/message.rs`, `app/src-tauri/src/gmail/error.rs` (+`Empty`), `app/src/services/auth-store.js`, `app/src/views/Login.vue`, `app/src/i18n/auth-errors.js`, `docs/ci4/*`.
```

- [ ] **Step 3: Commit**

```bash
git add docs/ci4/decisions.md docs/adr/_inbox/
git commit -m "$(cat <<'EOF'
docs: ADR-inbox + decisions для random message рішення

Фіксує вибір sample-space (100 останніх) і plain-text body extraction
у C4-моделі та ADR-inbox-чернетці.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 14: Final verification

- [ ] **Step 1: Run full Rust test suite**

Run: `cd app/src-tauri && cargo test --lib`
Expected: усі PASS (попередні 45 + новий 11 message + 4 list + 3 get + 1 empty error = 64).

- [ ] **Step 2: Run full Vue test suite**

Run: `bun --cwd app run test`
Expected: усі PASS.

- [ ] **Step 3: Run repo-wide lint and address spell-check additions**

Run: `bun run lint`

If `cspell` complains about Ukrainian words like «парс», «парсить», «обрати», «cброс» (or any new term we introduced), add them to `.cspell.json` `words` array. Re-run `bun run lint` until clean exit.

Expected after fixes: `Exit code 0`.

- [ ] **Step 4: Rust build sanity**

Run: `cd app/src-tauri && cargo build`
Expected: SUCCESS.

- [ ] **Step 5: Smoke checklist (manual; не виконує план)**

Виконує користувач:

1. `bun --cwd app run tauri dev` на macOS;
2. логін через Google → побачити «Ви увійшли як …», «Листів у скриньці: N», далі картку випадкового листа (Від/Тема/Дата + тіло);
3. натиснути «Показати інший» → лист змінюється на інший;
4. вимкнути мережу → натиснути «Показати інший» → побачити «Не вдалося з'єднатися з Google. Перевірте мережу.» замість картки;
5. «Вийти» → повертає до «Увійти через Google»;
6. (опційно) тест на акаунт із порожнім INBOX → побачити «Скринька порожня.».

- [ ] **Step 6: Commit final spell-check additions (if any)**

If `.cspell.json` was updated in Step 3, commit it separately:

```bash
git add .cspell.json
git commit -m "$(cat <<'EOF'
chore(cspell): add Ukrainian terms introduced by random message docs

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

**Spec coverage:**

| Spec requirement | Covered by |
| --- | --- |
| Випадковий лист серед 100 останніх INBOX | Task 5 (`list_inbox_ids_at` з maxResults=100) + Task 7 (`% ids.len()`) |
| Два Gmail-виклики (list + get) | Task 5, Task 6 |
| `text/plain` first, `text/html` fallback зі стрипом | Task 4 (`extract_plain_text`) |
| Body truncate до 10K символів | Task 6 (тест + код) |
| Порожній INBOX → `Empty` | Task 2 (variant) + Task 7 (повернення Empty) + Task 9 (i18n) |
| Auto-fetch після `initialize`/`login` | Task 8 (тести + код) |
| Кнопка «Показати інший» | Task 10 (тест + UI + click→loadRandomMessage) |
| Скидання у `logout`/`_resetForTest` | Task 8 (тест + код) |
| ReauthRequired знімає логін | Task 8 (тест) |
| Plain-text only, без HTML-рендеру | Task 10 (`<pre>` + `white-space: pre-wrap`) |
| 401 → ReauthRequired, 5xx → Http | Task 5, Task 6 (tests) |
| New dependencies (regex, html-escape) | Task 1 |
| Docs 03-components.md | Task 11 |
| Docs 04-code.md | Task 12 |
| Docs decisions.md + ADR-inbox | Task 13 |
| Empty kind у i18n | Task 9 |
| Поза scope (HTML рендер, mail reader, кеш) | не реалізуємо (свідомо) |

Прогалин не знайдено.

**Placeholder scan:** Жодних TBD/TODO/«ще щось». Усі тестові тіла, шаблони, текст помилок — повні. Усі команди явні.

**Type consistency:**

- `GmailMessage { id, from, subject, date, body }` — однаково в Rust і у тестах JS.
- Метод називається `loadRandomMessage` (не `refreshMessage` чи `nextMessage`) скрізь.
- Ref `currentMessage` (не `randomMessage` чи `message`) скрізь.
- Команда `gmail_random_message` (не `gmail_get_random_message`) скрізь.
- `GmailError::Empty` (не `EmptyInbox` чи `NoMessages`) скрізь.
- `extract_plain_text` / `extract_header` — стабільні імена.
- HTTP helpers — `list_inbox_ids_at` / `get_message_at` — стабільні.

Перевірив — все консистентно.
