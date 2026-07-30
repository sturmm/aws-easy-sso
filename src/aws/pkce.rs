use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::Digest;

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

impl Pkce {
    pub fn new() -> Self {
        let mut verifier_bytes = [0u8; 32];
        getrandom::fill(&mut verifier_bytes).expect("Failed to get random bytes");
        let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

        let mut hasher = sha2::Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge_bytes = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(challenge_bytes);

        Self {
            verifier,
            challenge,
        }
    }
}

impl Default for Pkce {
    fn default() -> Self {
        Self::new()
    }
}
