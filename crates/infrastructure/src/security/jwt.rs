use application::{
    error::{AppError, AppResult},
    ports::{AccessClaims, TokenService},
};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    /// Lifetime of access tokens in seconds; keep it very short.
    pub access_token_lifetime_secs: u64,
    /// Lifetime of requester tokens in seconds: how long until the user must log in again.
    pub requester_token_lifetime_secs: u64,
    /// Used as the `iss` and `com` claims.
    pub company_name: String,
    /// Used as the `aud` claim.
    pub company_domain: String,
}

impl JwtConfig {
    /// Panics if a variable is missing, as there is no sensible way to run without them.
    pub fn from_env() -> Self {
        Self {
            secret: config::required("AUTH_TOKEN_SECRET"),
            access_token_lifetime_secs: config::required_u64("AUTH_TOKEN_EXPIRE"),
            requester_token_lifetime_secs: config::required_u64("AUTH_REQUEST_TOKEN_EXPIRE"),
            company_name: config::required("COMPANY_NAME"),
            company_domain: config::required("COMPANY_DOMAIN"),
        }
    }
}

/// Claims of an access token (carries the access level).
#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    aud: String,
    iss: String,
    com: String,
    sub: String,
    exp: u64,
    acc: bool,
}

/// Claims of a requester token. Unknown fields are denied so an access token (which carries
/// `acc`) is never accepted as a requester token.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequesterTokenClaims {
    aud: String,
    iss: String,
    com: String,
    sub: String,
    exp: u64,
}

pub struct JwtTokenService {
    config: JwtConfig,
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtTokenService {
    pub fn new(config: JwtConfig) -> Self {
        Self {
            encoding: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding: DecodingKey::from_secret(config.secret.as_bytes()),
            config,
        }
    }

    fn validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 5;
        validation.set_audience(&[&self.config.company_domain]);
        validation.set_issuer(&[&self.config.company_name]);
        validation
    }

    fn encode<T: Serialize>(&self, claims: &T) -> AppResult<String> {
        encode(&Header::default(), claims, &self.encoding).map_err(|error| {
            println!("Error creating token: {error}");
            AppError::TokenCreation
        })
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, token: &str) -> AppResult<T> {
        decode::<T>(token, &self.decoding, &self.validation())
            .map(|data| data.claims)
            .map_err(|_| AppError::InvalidToken)
    }
}

impl TokenService for JwtTokenService {
    fn issue_requester_token(&self, user_uuid: &str) -> AppResult<String> {
        self.encode(&RequesterTokenClaims {
            sub: user_uuid.to_string(),
            aud: self.config.company_domain.clone(),
            iss: self.config.company_name.clone(),
            com: self.config.company_name.clone(),
            exp: get_current_timestamp() + self.config.requester_token_lifetime_secs,
        })
    }

    fn issue_access_token(&self, user_uuid: &str, is_admin: bool) -> AppResult<String> {
        self.encode(&AccessTokenClaims {
            sub: user_uuid.to_string(),
            aud: self.config.company_domain.clone(),
            iss: self.config.company_name.clone(),
            com: self.config.company_name.clone(),
            exp: get_current_timestamp() + self.config.access_token_lifetime_secs,
            acc: is_admin,
        })
    }

    fn verify_requester_token(&self, token: &str) -> AppResult<String> {
        Ok(self.decode::<RequesterTokenClaims>(token)?.sub)
    }

    fn verify_access_token(&self, token: &str) -> AppResult<AccessClaims> {
        let claims = self.decode::<AccessTokenClaims>(token)?;
        Ok(AccessClaims {
            user_uuid: claims.sub,
            is_admin: claims.acc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> JwtTokenService {
        JwtTokenService::new(JwtConfig {
            secret: "test-secret-test-secret-test-secret".into(),
            access_token_lifetime_secs: 60,
            requester_token_lifetime_secs: 3600,
            company_name: "TestCo".into(),
            company_domain: "test.example.com".into(),
        })
    }

    #[test]
    fn tokens_round_trip() {
        let tokens = service();
        let requester = tokens.issue_requester_token("u-1").unwrap();
        assert_eq!(tokens.verify_requester_token(&requester).unwrap(), "u-1");
        let access = tokens.issue_access_token("u-1", true).unwrap();
        assert_eq!(
            tokens.verify_access_token(&access).unwrap(),
            AccessClaims {
                user_uuid: "u-1".into(),
                is_admin: true
            }
        );
    }

    #[test]
    fn the_two_token_kinds_are_not_interchangeable() {
        let tokens = service();
        let requester = tokens.issue_requester_token("u-1").unwrap();
        let access = tokens.issue_access_token("u-1", false).unwrap();
        assert_eq!(
            tokens.verify_access_token(&requester),
            Err(AppError::InvalidToken)
        );
        assert_eq!(
            tokens.verify_requester_token(&access),
            Err(AppError::InvalidToken)
        );
    }

    #[test]
    fn foreign_tokens_are_rejected() {
        let tokens = service();
        let mut other = service();
        other.config.company_name = "Other".into();
        let token = other.issue_access_token("u-1", true).unwrap();
        assert_eq!(
            tokens.verify_access_token(&token),
            Err(AppError::InvalidToken)
        );
        assert_eq!(
            tokens.verify_access_token("garbage"),
            Err(AppError::InvalidToken)
        );
    }
}
