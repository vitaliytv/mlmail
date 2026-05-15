use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
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
}
