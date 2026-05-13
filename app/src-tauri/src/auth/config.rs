use std::sync::OnceLock;

const KEY_DESKTOP: &str = "MLMAIL_GOOGLE_DESKTOP_CLIENT_ID";
const KEY_ANDROID: &str = "MLMAIL_GOOGLE_ANDROID_CLIENT_ID";
const KEY_ANDROID_WEB: &str = "MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID";
const PLACEHOLDER: &str = "REPLACE_ME";

static DESKTOP: OnceLock<String> = OnceLock::new();
static ANDROID: OnceLock<String> = OnceLock::new();
static ANDROID_WEB: OnceLock<String> = OnceLock::new();

/// Loads `.env` once from CWD (best-effort) and resolves the three Google
/// OAuth client IDs from process env. Missing vars fall back to a sentinel
/// `REPLACE_ME_<key>` so the binary still links; callers check via
/// `is_real_client_id` before relying on the value.
fn resolve(key: &str) -> String {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        let _ = dotenvy::dotenv();
    });
    std::env::var(key).unwrap_or_else(|_| format!("{PLACEHOLDER}_{key}"))
}

pub fn desktop_client_id() -> &'static str {
    DESKTOP.get_or_init(|| resolve(KEY_DESKTOP)).as_str()
}

pub fn android_client_id() -> &'static str {
    ANDROID.get_or_init(|| resolve(KEY_ANDROID)).as_str()
}

pub fn android_web_client_id() -> &'static str {
    ANDROID_WEB.get_or_init(|| resolve(KEY_ANDROID_WEB)).as_str()
}

pub fn is_real_client_id(value: &str) -> bool {
    !value.starts_with(PLACEHOLDER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_value_is_not_considered_real() {
        assert!(!is_real_client_id("REPLACE_ME_MLMAIL_GOOGLE_DESKTOP_CLIENT_ID"));
    }

    #[test]
    fn real_looking_client_id_is_considered_real() {
        assert!(is_real_client_id("123-abc.apps.googleusercontent.com"));
    }
}
