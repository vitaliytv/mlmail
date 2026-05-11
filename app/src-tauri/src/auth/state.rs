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

    pub fn reset(&mut self) {
        self.email = None;
        self.access_token = None;
        self.access_token_expires_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn fresh_when_expiry_well_in_future() {
        let s = AuthState {
            email: Some("u@example.com".into()),
            access_token: Some("tok".into()),
            access_token_expires_at: Some(Instant::now() + Duration::from_secs(3600)),
        };
        assert!(s.is_access_token_fresh());
    }

    #[test]
    fn stale_when_expiry_within_30_seconds() {
        let s = AuthState {
            email: None,
            access_token: Some("tok".into()),
            access_token_expires_at: Some(Instant::now() + Duration::from_secs(10)),
        };
        assert!(!s.is_access_token_fresh());
    }

    #[test]
    fn stale_when_no_token_present() {
        let s = AuthState {
            email: Some("u@example.com".into()),
            access_token: None,
            access_token_expires_at: Some(Instant::now() + Duration::from_secs(3600)),
        };
        assert!(!s.is_access_token_fresh());
    }

    #[test]
    fn stale_when_no_expiry_set() {
        let s = AuthState {
            email: None,
            access_token: Some("tok".into()),
            access_token_expires_at: None,
        };
        assert!(!s.is_access_token_fresh());
    }

    #[test]
    fn default_is_fully_empty_and_stale() {
        let s = AuthState::default();
        assert!(s.email.is_none());
        assert!(s.access_token.is_none());
        assert!(s.access_token_expires_at.is_none());
        assert!(!s.is_access_token_fresh());
    }
}
