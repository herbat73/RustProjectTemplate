use std::sync::Arc;

use application::{
    auth::AuthService, todo::TodoService, users::UserService, weather::WeatherService,
};

use crate::controllers::ws_controller::ChatHub;

/// The use cases (and chat hub) shared by all handlers. Built once by [`crate::bootstrap`].
#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub users: Arc<UserService>,
    pub todos: Arc<TodoService>,
    pub weather: Arc<WeatherService>,
    pub chat: ChatHub,
}
