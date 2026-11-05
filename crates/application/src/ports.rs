//! Interfaces to everything the use cases need that is not business logic itself.
//! Implemented by the infrastructure layer.

use async_trait::async_trait;
use domain::weather::WeatherForecast;

use crate::error::AppResult;

/// Identity carried by a verified access token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessClaims {
    pub user_uuid: String,
    pub is_admin: bool,
}

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, plain: &str) -> AppResult<String>;
    fn verify(&self, plain: &str, hash: &str) -> bool;
}

/// Issues and verifies the two kinds of tokens:
/// - *requester* tokens: long-lived, obtained by logging in, only good for requesting access tokens;
/// - *access* tokens: short-lived, carry the access level and authorize API calls.
pub trait TokenService: Send + Sync {
    fn issue_requester_token(&self, user_uuid: &str) -> AppResult<String>;
    fn issue_access_token(&self, user_uuid: &str, is_admin: bool) -> AppResult<String>;
    /// Returns the uuid of the user the requester token was issued to.
    fn verify_requester_token(&self, token: &str) -> AppResult<String>;
    fn verify_access_token(&self, token: &str) -> AppResult<AccessClaims>;
}

/// Short-lived keys that prove ownership of an email address when resetting a password.
#[async_trait]
pub trait ResetKeyStore: Send + Sync {
    /// Creates and remembers a new key for the email address.
    async fn issue(&self, email: &str) -> String;
    /// Whether the key exists, belongs to the email address and has not expired.
    async fn is_valid(&self, key: &str, email: &str) -> bool;
    async fn revoke(&self, key: &str);
}

#[async_trait]
pub trait ResetMailer: Send + Sync {
    async fn send_reset_link(&self, email: &str, reset_key: &str) -> AppResult<()>;
}

#[async_trait]
pub trait ForecastProvider: Send + Sync {
    /// Forecasts for each of the next `days` days, starting tomorrow.
    async fn next_days(&self, days: u32) -> AppResult<Vec<WeatherForecast>>;
}
