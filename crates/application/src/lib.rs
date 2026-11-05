//! Application layer: the use cases of the system. Each service orchestrates domain entities
//! through repository and service *ports* that the infrastructure layer implements.

pub mod auth;
pub mod error;
pub mod ports;
pub mod todo;
pub mod users;
pub mod weather;

#[cfg(test)]
mod fakes;

pub use error::{AppError, AppResult};
