use std::sync::OnceLock;

const KEY_DESKTOP: &str = "MLMAIL_GOOGLE_DESKTOP_CLIENT_ID";
const KEY_DESKTOP_SECRET: &str = "MLMAIL_GOOGLE_DESKTOP_CLIENT_SECRET";
const KEY_ANDROID: &str = "MLMAIL_GOOGLE_ANDROID_CLIENT_ID";
const KEY_ANDROID_WEB: &str = "MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID";
const PLACEHOLDER: &str = "REPLACE_ME";

// Built-in Google OAuth Desktop client_secret. Google treats the Desktop-client
// secret as non-confidential (it ships in every native binary anyway and PKCE is
// the real protection), so it is embedded here as a default. An env / .env.secret
// value still overrides it. trufflehog is told to skip this file (.trufflehog-exclude).
const DESKTOP_SECRET_BUILTIN: &str = "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"; // cspell:disable-line

static DESKTOP: OnceLock<String> = OnceLock::new();
static DESKTOP_SECRET: OnceLock<String> = OnceLock::new();
static ANDROID: OnceLock<String> = OnceLock::new();
static ANDROID_WEB: OnceLock<String> = OnceLock::new();

/// Loads `.env` once from CWD (best-effort) and resolves the three Google
/// OAuth client IDs from process env. Missing vars fall back to a sentinel
/// `REPLACE_ME_<key>` so the binary still links; callers check via
/// `is_real_client_id` before relying on the value.
fn resolve(key: &str) -> String {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        // Public IDs (tracked in git).
        let _ = dotenvy::from_filename(".env");
        // Secrets (gitignored). Loaded second; existing keys (from process env
        // or .env above) are not overwritten — dotenvy::from_filename only
        // sets vars that aren't already in the environment.
        let _ = dotenvy::from_filename(".env.secret");
    });
    std::env::var(key).unwrap_or_else(|_| format!("{PLACEHOLDER}_{key}"))
}

pub fn desktop_client_id() -> &'static str {
    DESKTOP.get_or_init(|| resolve(KEY_DESKTOP)).as_str()
}

pub fn desktop_client_secret() -> &'static str {
    DESKTOP_SECRET
        .get_or_init(|| {
            let from_env = resolve(KEY_DESKTOP_SECRET);
            if is_real_client_id(&from_env) {
                from_env
            } else {
                DESKTOP_SECRET_BUILTIN.to_string()
            }
        })
        .as_str()
}

pub fn android_client_id() -> &'static str {
    ANDROID.get_or_init(|| resolve(KEY_ANDROID)).as_str()
}

pub fn android_web_client_id() -> &'static str {
    ANDROID_WEB.get_or_init(|| resolve(KEY_ANDROID_WEB)).as_str()
}

pub fn is_real_client_id(value: &str) -> bool {
    !value.trim().is_empty() && !value.starts_with(PLACEHOLDER)
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

    #[test]
    fn empty_or_blank_value_is_not_considered_real() {
        assert!(!is_real_client_id(""));
        assert!(!is_real_client_id("   "));
    }
}
