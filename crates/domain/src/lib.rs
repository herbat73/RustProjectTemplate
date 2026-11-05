//! Domain layer: entities, business rules and the repository interfaces (ports) the
//! outer layers implement. It has no knowledge of HTTP, SQL, JWT or any other detail.

pub mod error;
pub mod todo;
pub mod user;
pub mod weather;
