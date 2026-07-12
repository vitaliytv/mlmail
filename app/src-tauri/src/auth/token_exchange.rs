use crate::auth::error::AuthError;
use serde::Deserialize;

pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum FlowKind {
    Desktop,
    Android,
}

pub async fn exchange_code(
    client_id: &str,
    client_secret: Option<&str>,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
    flow: FlowKind,
) -> Result<TokenResponse, AuthError> {
    exchange_code_at(
        GOOGLE_TOKEN_URL,
        client_id,
        client_secret,
        code,
        code_verifier,
        redirect_uri,
        flow,
    )
    .await
}

pub async fn exchange_refresh(
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    exchange_refresh_at(GOOGLE_TOKEN_URL, client_id, client_secret, refresh_token).await
}

pub(crate) async fn exchange_code_at(
    endpoint: &str,
    client_id: &str,
    client_secret: Option<&str>,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
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
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }

    let resp = reqwest::Client::new()
        .post(endpoint)
        .form(&form)
        .send()
        .await?;

    let status = resp.status();
    if status.is_success() {
        return resp
            .json::<TokenResponse>()
            .await
            .map_err(|e| AuthError::OAuth(e.to_string()));
    }
    let body = resp.text().await.unwrap_or_default();
    eprintln!("Google token endpoint returned {status}: {body}");
    Err(classify_http_error(status, &body))
}

pub(crate) async fn exchange_refresh_at(
    endpoint: &str,
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    let mut form: Vec<(&str, &str)> = vec![
        ("client_id", client_id),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }

    let resp = reqwest::Client::new()
        .post(endpoint)
        .form(&form)
        .send()
        .await?;

    let status = resp.status();
    if status.is_success() {
        return resp
            .json::<TokenResponse>()
            .await
            .map_err(|e| AuthError::OAuth(e.to_string()));
    }
    let body = resp.text().await.unwrap_or_default();
    eprintln!("Google token endpoint (refresh) returned {status}: {body}");
    if status == reqwest::StatusCode::BAD_REQUEST && body.contains("invalid_grant") {
        return Err(AuthError::ReauthRequired);
    }
    Err(classify_http_error(status, &body))
}

fn classify_http_error(status: reqwest::StatusCode, body: &str) -> AuthError {
    if status.is_server_error() {
        return AuthError::Network(format!("HTTP {status}: {body}"));
    }
    // 4xx — Google returns a JSON object like {"error":"invalid_grant","error_description":"..."}
    let oauth_error = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .map(|v| {
            let kind = v
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("oauth_error");
            let desc = v
                .get("error_description")
                .and_then(|e| e.as_str())
                .unwrap_or("");
            if desc.is_empty() {
                kind.to_string()
            } else {
                format!("{kind}: {desc}")
            }
        })
        .unwrap_or_else(|| format!("HTTP {status}: {body}"));
    AuthError::OAuth(oauth_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::error::AuthError;

    #[tokio::test]
    async fn exchange_code_parses_full_success_response() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token":"AT",
                "expires_in":3600,
                "refresh_token":"RT",
                "id_token":"ID"
            }"#,
            )
            .create_async()
            .await;

        let r = exchange_code_at(
            &format!("{}/token", server.url()),
            "cid",
            Some("secret"),
            "code",
            "verifier",
            "http://127.0.0.1:1234/callback",
            FlowKind::Desktop,
        )
        .await
        .unwrap();

        assert_eq!(r.access_token, "AT");
        assert_eq!(r.expires_in, 3600);
        assert_eq!(r.refresh_token.as_deref(), Some("RT"));
        assert_eq!(r.id_token.as_deref(), Some("ID"));
        m.assert_async().await;
    }

    #[tokio::test]
    async fn exchange_code_desktop_sends_pkce_verifier_and_client_secret() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("code_verifier".into(), "the-verifier".into()),
                mockito::Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
                mockito::Matcher::UrlEncoded("client_id".into(), "cid".into()),
                mockito::Matcher::UrlEncoded("client_secret".into(), "the-secret".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"AT","expires_in":1}"#)
            .create_async()
            .await;

        let _ = exchange_code_at(
            &format!("{}/token", server.url()),
            "cid",
            Some("the-secret"),
            "the-code",
            "the-verifier",
            "http://127.0.0.1:0/callback",
            FlowKind::Desktop,
        )
        .await
        .unwrap();

        m.assert_async().await;
    }

    #[tokio::test]
    async fn exchange_code_android_omits_pkce_and_client_secret() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
                mockito::Matcher::UrlEncoded("code".into(), "android-code".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"AT","expires_in":1}"#)
            .create_async()
            .await;

        let _ = exchange_code_at(
            &format!("{}/token", server.url()),
            "cid",
            None,
            "android-code",
            "",
            "",
            FlowKind::Android,
        )
        .await
        .unwrap();

        m.assert_async().await;
    }

    #[tokio::test]
    async fn exchange_code_maps_5xx_to_network_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(503)
            .with_body("upstream failure")
            .create_async()
            .await;

        let err = exchange_code_at(
            &format!("{}/token", server.url()),
            "cid",
            Some("s"),
            "code",
            "v",
            "http://r",
            FlowKind::Desktop,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, AuthError::Network(_)));
    }

    #[tokio::test]
    async fn exchange_refresh_returns_reauth_required_on_invalid_grant() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(400)
            .with_body(r#"{"error":"invalid_grant"}"#)
            .create_async()
            .await;

        let err = exchange_refresh_at(&format!("{}/token", server.url()), "cid", None, "rt")
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::ReauthRequired));
    }

    #[tokio::test]
    async fn exchange_refresh_parses_new_tokens() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(200)
            .with_body(r#"{"access_token":"NEW","expires_in":3600}"#)
            .create_async()
            .await;

        let r = exchange_refresh_at(&format!("{}/token", server.url()), "cid", None, "rt")
            .await
            .unwrap();
        assert_eq!(r.access_token, "NEW");
        assert_eq!(r.refresh_token, None);
    }

    #[tokio::test]
    async fn exchange_refresh_returns_rotated_refresh_token_when_present() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(200)
            .with_body(r#"{"access_token":"NEW","expires_in":3600,"refresh_token":"ROT"}"#)
            .create_async()
            .await;

        let r = exchange_refresh_at(&format!("{}/token", server.url()), "cid", None, "rt")
            .await
            .unwrap();
        assert_eq!(r.refresh_token.as_deref(), Some("ROT"));
    }
}
