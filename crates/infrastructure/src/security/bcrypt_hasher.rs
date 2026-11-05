use application::{
    error::{AppError, AppResult},
    ports::PasswordHasher,
};
use bcrypt::{DEFAULT_COST, hash_with_salt};

use crate::config;

/// bcrypt with a fixed, application-wide salt (the first 16 bytes of `PASSWORD_SALT`).
pub struct BcryptPasswordHasher {
    salt: [u8; 16],
}

impl BcryptPasswordHasher {
    pub fn new(salt: [u8; 16]) -> Self {
        Self { salt }
    }

    /// Panics if `PASSWORD_SALT` is missing or shorter than 16 bytes.
    pub fn from_env() -> Self {
        let value = config::required("PASSWORD_SALT");
        let bytes = value.as_bytes();
        assert!(bytes.len() >= 16, "PASSWORD_SALT must be at least 16 bytes");
        let mut salt = [0; 16];
        salt.copy_from_slice(&bytes[..16]);
        Self::new(salt)
    }
}

impl PasswordHasher for BcryptPasswordHasher {
    fn hash(&self, plain: &str) -> AppResult<String> {
        hash_with_salt(plain, DEFAULT_COST, self.salt)
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::Internal(format!("password hashing failed: {error}")))
    }

    fn verify(&self, plain: &str, hash: &str) -> bool {
        bcrypt::verify(plain, hash).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_verify_only_for_the_right_password() {
        let hasher = BcryptPasswordHasher::new(*b"0123456789abcdef");
        let hash = hasher.hash("pw12345!").unwrap();
        assert!(hasher.verify("pw12345!", &hash));
        assert!(!hasher.verify("other", &hash));
        assert!(!hasher.verify("pw12345!", "not a bcrypt hash"));
    }
}
