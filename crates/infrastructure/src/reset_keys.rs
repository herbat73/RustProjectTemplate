use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use application::ports::ResetKeyStore;
use async_trait::async_trait;
use rand::{RngExt, distr::Alphanumeric};

const DEFAULT_LIFETIME: Duration = Duration::from_secs(3600 * 24);

struct Entry {
    email: String,
    issued_at: SystemTime,
}

/// Keeps reset keys in memory, so they are lost when the server restarts.
pub struct InMemoryResetKeyStore {
    keys: Mutex<HashMap<String, Entry>>,
    lifetime: Duration,
}

impl InMemoryResetKeyStore {
    pub fn new() -> Self {
        Self::with_lifetime(DEFAULT_LIFETIME)
    }

    pub fn with_lifetime(lifetime: Duration) -> Self {
        Self {
            keys: Mutex::new(HashMap::new()),
            lifetime,
        }
    }
}

impl Default for InMemoryResetKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ResetKeyStore for InMemoryResetKeyStore {
    async fn issue(&self, email: &str) -> String {
        let key: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        self.keys.lock().unwrap().insert(
            key.clone(),
            Entry {
                email: email.to_string(),
                issued_at: SystemTime::now(),
            },
        );
        key
    }

    async fn is_valid(&self, key: &str, email: &str) -> bool {
        self.keys.lock().unwrap().get(key).is_some_and(|entry| {
            entry.email == email
                && SystemTime::now()
                    .duration_since(entry.issued_at)
                    .is_ok_and(|age| age <= self.lifetime)
        })
    }

    async fn revoke(&self, key: &str) {
        self.keys.lock().unwrap().remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn keys_are_bound_to_the_email_and_single_use() {
        let store = InMemoryResetKeyStore::new();
        let key = store.issue("a@example.com").await;
        assert_eq!(key.len(), 64);
        assert!(store.is_valid(&key, "a@example.com").await);
        assert!(!store.is_valid(&key, "b@example.com").await);
        assert!(!store.is_valid("unknown", "a@example.com").await);
        store.revoke(&key).await;
        assert!(!store.is_valid(&key, "a@example.com").await);
    }

    #[tokio::test]
    async fn keys_expire() {
        let store = InMemoryResetKeyStore::with_lifetime(Duration::ZERO);
        let key = store.issue("a@example.com").await;
        tokio::time::sleep(Duration::from_millis(5)).await;
        assert!(!store.is_valid(&key, "a@example.com").await);
    }
}
