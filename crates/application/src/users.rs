use std::sync::Arc;

use domain::user::{User, UserRepository};

use crate::{
    error::{AppError, AppResult},
    ports::AccessClaims,
};

pub struct UserService {
    users: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    pub async fn get(&self, user_uuid: &str) -> AppResult<User> {
        self.users
            .find_by_uuid(user_uuid)
            .await?
            .ok_or(AppError::UserDoesNotExist)
    }

    /// Admin only.
    pub async fn list_all(&self, caller: &AccessClaims) -> AppResult<Vec<User>> {
        require_admin(caller)?;
        Ok(self.users.list().await?)
    }

    /// Admin only.
    pub async fn delete(&self, caller: &AccessClaims, user_uuid: &str) -> AppResult<()> {
        require_admin(caller)?;
        self.users.delete_by_uuid(user_uuid).await?;
        Ok(())
    }
}

fn require_admin(caller: &AccessClaims) -> AppResult<()> {
    if caller.is_admin {
        Ok(())
    } else {
        Err(AppError::AccessDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{FakeUsers, user};

    fn service() -> UserService {
        UserService::new(Arc::new(FakeUsers::with(vec![
            user(1, "admin-uuid", "admin", true),
            user(2, "bob-uuid", "bob", false),
        ])))
    }

    fn claims(uuid: &str, is_admin: bool) -> AccessClaims {
        AccessClaims {
            user_uuid: uuid.into(),
            is_admin,
        }
    }

    #[tokio::test]
    async fn listing_and_deleting_require_admin() {
        let users = service();
        let admin = claims("admin-uuid", true);
        let bob = claims("bob-uuid", false);

        assert_eq!(
            users.list_all(&bob).await.unwrap_err(),
            AppError::AccessDenied
        );
        assert_eq!(
            users.delete(&bob, "admin-uuid").await.unwrap_err(),
            AppError::AccessDenied
        );

        assert_eq!(users.list_all(&admin).await.unwrap().len(), 2);
        users.delete(&admin, "bob-uuid").await.unwrap();
        assert_eq!(users.list_all(&admin).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_reports_unknown_users() {
        let users = service();
        assert_eq!(users.get("bob-uuid").await.unwrap().username, "bob");
        assert_eq!(
            users.get("ghost").await.unwrap_err(),
            AppError::UserDoesNotExist
        );
    }
}
