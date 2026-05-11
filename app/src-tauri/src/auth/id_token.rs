use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub fn extract_email(id_token: &str) -> Option<String> {
    let payload_b64 = id_token.split('.').nth(1)?;
    let payload = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    json.get("email")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

    fn build_jwt(payload_json: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(b"fake-signature");
        format!("{header}.{payload}.{signature}")
    }

    #[test]
    fn returns_email_from_valid_jwt() {
        let jwt = build_jwt(r#"{"email":"vitaliytv@nitralabs.com","sub":"123"}"#);
        assert_eq!(extract_email(&jwt), Some("vitaliytv@nitralabs.com".into()));
    }

    #[test]
    fn returns_none_when_jwt_has_no_email_field() {
        let jwt = build_jwt(r#"{"sub":"123"}"#);
        assert_eq!(extract_email(&jwt), None);
    }

    #[test]
    fn returns_none_when_payload_is_not_json() {
        let jwt = build_jwt("not-json-at-all");
        assert_eq!(extract_email(&jwt), None);
    }

    #[test]
    fn returns_none_when_token_is_not_jwt_shape() {
        assert_eq!(extract_email("totally-bogus"), None);
    }

    #[test]
    fn returns_none_when_payload_is_not_base64() {
        assert_eq!(extract_email("header.NOT_BASE_64!!!.sig"), None);
    }
}
