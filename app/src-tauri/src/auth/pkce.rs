use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
}

pub fn generate() -> PkcePair {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
    PkcePair { verifier, challenge }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Digest, Sha256};

    fn is_url_safe(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }

    #[test]
    fn verifier_length_is_in_pkce_spec_range() {
        let pair = generate();
        assert!(pair.verifier.len() >= 43);
        assert!(pair.verifier.len() <= 128);
    }

    #[test]
    fn verifier_is_url_safe() {
        let pair = generate();
        assert!(is_url_safe(&pair.verifier));
    }

    #[test]
    fn challenge_is_url_safe() {
        let pair = generate();
        assert!(is_url_safe(&pair.challenge));
    }

    #[test]
    fn challenge_matches_sha256_of_verifier() {
        let pair = generate();
        let mut hasher = Sha256::new();
        hasher.update(pair.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(pair.challenge, expected);
    }

    #[test]
    fn two_calls_produce_different_pairs() {
        let a = generate();
        let b = generate();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
    }
}
