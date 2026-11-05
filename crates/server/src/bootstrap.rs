//! Composition root: the only place that knows both the use cases and the concrete
//! infrastructure, and wires them together.

use std::sync::Arc;

use application::{
    auth::AuthService, todo::TodoService, users::UserService, weather::WeatherService,
};
use infrastructure::{
    db::{self, DbPool},
    mail::SmtpResetMailer,
    persistence::{SqlxTodoRepository, SqlxUserRepository},
    reset_keys::InMemoryResetKeyStore,
    security::{BcryptPasswordHasher, JwtConfig, JwtTokenService},
    weather::RandomForecastProvider,
};

use crate::{controllers::ws_controller::ChatHub, state::AppState};

/// Connects to the database in `DATABASE_URL` and builds the application from the environment.
pub async fn state_from_env() -> AppState {
    state_with_pool(db::connect_from_env().await)
}

/// Builds the application on top of an existing connection pool. The remaining configuration
/// (token secrets, password salt, mail) is read from the environment.
pub fn state_with_pool(pool: DbPool) -> AppState {
    let users = Arc::new(SqlxUserRepository::new(pool.clone()));
    let todos = Arc::new(SqlxTodoRepository::new(pool));

    let auth = AuthService::new(
        users.clone(),
        Arc::new(BcryptPasswordHasher::from_env()),
        Arc::new(JwtTokenService::new(JwtConfig::from_env())),
        Arc::new(InMemoryResetKeyStore::new()),
        Arc::new(SmtpResetMailer::from_env()),
    );

    AppState {
        auth: Arc::new(auth),
        users: Arc::new(UserService::new(users.clone())),
        todos: Arc::new(TodoService::new(users, todos)),
        weather: Arc::new(WeatherService::new(Arc::new(RandomForecastProvider))),
        chat: ChatHub::new(),
    }
}
