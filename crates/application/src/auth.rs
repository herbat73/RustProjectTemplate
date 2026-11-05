use std::{str::FromStr, sync::Arc};

use domain::{
    error::RepositoryError,
    user::{NewUser, User, UserRepository},
};
use email_address::EmailAddress;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    ports::{AccessClaims, PasswordHasher, ResetKeyStore, ResetMailer, TokenService},
};

/// A user together with the requester token issued to them.
#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    pub requester_token: String,
}

/// Registration, login, token exchange and password reset.
pub struct AuthService {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    reset_keys: Arc<dyn ResetKeyStore>,
    mailer: Arc<dyn ResetMailer>,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
        reset_keys: Arc<dyn ResetKeyStore>,
        mailer: Arc<dyn ResetMailer>,
    ) -> Self {
        Self {
            users,
            hasher,
            tokens,
            reset_keys,
            mailer,
        }
    }

    /// Logs in with a username (or email) and password.
    pub async fn login(&self, username_or_email: &str, password: &str) -> AppResult<Session> {
        if username_or_email.is_empty() || password.is_empty() {
            return Err(AppError::WrongCredentials);
        }
        let user = self
            .users
            .find_by_username_or_email(username_or_email)
            .await?
            .ok_or(AppError::UserDoesNotExist)?;
        if !self.hasher.verify(password, &user.password_hash) {
            return Err(AppError::WrongCredentials);
        }
        let requester_token = self.tokens.issue_requester_token(&user.uuid)?;
        Ok(Session {
            user,
            requester_token,
        })
    }

    pub async fn register(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> AppResult<Session> {
        if username.is_empty() || password.is_empty() || email.is_empty() {
            return Err(AppError::MissingFields);
        }
        if !EmailAddress::is_valid(email) {
            return Err(AppError::InvalidEmail);
        }
        let new_user = NewUser {
            uuid: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: self.hasher.hash(password)?,
            email: email.to_string(),
        };
        let user = self.users.insert(new_user).await.map_err(|error| {
            if error == RepositoryError::Conflict {
                AppError::UserAlreadyExists
            } else {
                AppError::from(error)
            }
        })?;
        let requester_token = self.tokens.issue_requester_token(&user.uuid)?;
        Ok(Session {
            user,
            requester_token,
        })
    }

    /// Checks a requester token and returns the uuid of its user.
    pub fn verify_requester_token(&self, token: &str) -> AppResult<String> {
        self.tokens.verify_requester_token(token)
    }

    pub fn verify_access_token(&self, token: &str) -> AppResult<AccessClaims> {
        self.tokens.verify_access_token(token)
    }

    /// Exchanges the identity of a verified requester token for a short-lived access token that
    /// carries the user's current access level.
    pub async fn issue_access_token(&self, requester_user_uuid: &str) -> AppResult<String> {
        let user = match self.users.find_by_uuid(requester_user_uuid).await {
            Ok(Some(user)) => user,
            _ => return Err(AppError::AccessDenied),
        };
        self.tokens.issue_access_token(&user.uuid, user.is_admin)
    }

    /// Resolves the user behind a requester token (used to authenticate the chat socket).
    pub async fn user_for_requester_token(&self, token: &str) -> AppResult<User> {
        let uuid = self.tokens.verify_requester_token(token)?;
        self.users
            .find_by_uuid(&uuid)
            .await?
            .ok_or(AppError::UserDoesNotExist)
    }

    /// Emails a password reset link to the user registered with the address.
    pub async fn request_password_reset(&self, email: &str) -> AppResult<()> {
        let email = EmailAddress::from_str(email).map_err(|_| AppError::InvalidEmail)?;
        let email = email.to_string();
        self.users
            .find_by_username_or_email(&email)
            .await?
            .ok_or(AppError::UserDoesNotExist)?;
        let key = self.reset_keys.issue(&email).await;
        self.mailer.send_reset_link(&email, &key).await
    }

    /// Sets a new password using the key from the reset email.
    pub async fn reset_password(
        &self,
        reset_key: &str,
        email: &str,
        new_password: &str,
    ) -> AppResult<()> {
        let user = self
            .users
            .find_by_username_or_email(email)
            .await?
            .ok_or(AppError::UserDoesNotExist)?;
        if !self.reset_keys.is_valid(reset_key, email).await {
            return Err(AppError::ResetLinkInvalid);
        }
        let hash = self.hasher.hash(new_password)?;
        self.users.update_password_hash(user.id, &hash).await?;
        self.reset_keys.revoke(reset_key).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{FakeHasher, FakeMailer, FakeResetKeys, FakeTokens, FakeUsers, user};

    struct Fixture {
        auth: AuthService,
        users: Arc<FakeUsers>,
        mailer: Arc<FakeMailer>,
    }

    fn fixture() -> Fixture {
        let users = Arc::new(FakeUsers::with(vec![user(1, "alice-uuid", "alice", true)]));
        let mailer = Arc::new(FakeMailer::default());
        let auth = AuthService::new(
            users.clone(),
            Arc::new(FakeHasher),
            Arc::new(FakeTokens),
            Arc::new(FakeResetKeys::default()),
            mailer.clone(),
        );
        Fixture {
            auth,
            users,
            mailer,
        }
    }

    #[tokio::test]
    async fn register_validates_and_rejects_duplicates() {
        let f = fixture();
        assert_eq!(
            f.auth.register("", "pw", "a@b.com").await.unwrap_err(),
            AppError::MissingFields
        );
        assert_eq!(
            f.auth
                .register("bob", "pw", "not-an-email")
                .await
                .unwrap_err(),
            AppError::InvalidEmail
        );

        let session = f
            .auth
            .register("bob", "pw", "bob@example.com")
            .await
            .unwrap();
        assert_eq!(session.user.username, "bob");
        assert!(!session.user.is_admin);
        assert_eq!(
            session.requester_token,
            format!("requester:{}", session.user.uuid)
        );

        assert_eq!(
            f.auth
                .register("bob", "pw", "other@example.com")
                .await
                .unwrap_err(),
            AppError::UserAlreadyExists
        );
    }

    #[tokio::test]
    async fn login_checks_credentials() {
        let f = fixture();
        assert_eq!(
            f.auth.login("", "secret").await.unwrap_err(),
            AppError::WrongCredentials
        );
        assert_eq!(
            f.auth.login("nobody", "secret").await.unwrap_err(),
            AppError::UserDoesNotExist
        );
        assert_eq!(
            f.auth.login("alice", "wrong").await.unwrap_err(),
            AppError::WrongCredentials
        );
        let session = f.auth.login("alice", "secret").await.unwrap();
        assert_eq!(session.requester_token, "requester:alice-uuid");
    }

    #[tokio::test]
    async fn access_token_carries_the_current_access_level() {
        let f = fixture();
        assert_eq!(
            f.auth.issue_access_token("alice-uuid").await.unwrap(),
            "access:alice-uuid:true"
        );
        assert_eq!(
            f.auth.issue_access_token("ghost").await.unwrap_err(),
            AppError::AccessDenied
        );
    }

    #[tokio::test]
    async fn password_reset_round_trip() {
        let f = fixture();
        assert_eq!(
            f.auth.request_password_reset("nope").await.unwrap_err(),
            AppError::InvalidEmail
        );
        assert_eq!(
            f.auth
                .request_password_reset("ghost@example.com")
                .await
                .unwrap_err(),
            AppError::UserDoesNotExist
        );

        f.auth
            .request_password_reset("alice@example.com")
            .await
            .unwrap();
        let (email, key) = f.mailer.sent.lock().unwrap()[0].clone();
        assert_eq!(email, "alice@example.com");

        assert_eq!(
            f.auth
                .reset_password("wrong-key", &email, "new")
                .await
                .unwrap_err(),
            AppError::ResetLinkInvalid
        );
        f.auth.reset_password(&key, &email, "new").await.unwrap();
        // the key is single use
        assert_eq!(
            f.auth
                .reset_password(&key, &email, "again")
                .await
                .unwrap_err(),
            AppError::ResetLinkInvalid
        );
        assert!(f.auth.login("alice", "new").await.is_ok());
        assert_eq!(
            f.users
                .find_by_uuid("alice-uuid")
                .await
                .unwrap()
                .unwrap()
                .password_hash,
            "hashed:new"
        );
    }
}
