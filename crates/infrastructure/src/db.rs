use std::time::Duration;

use sqlx::{
    AnyPool,
    any::{AnyPoolOptions, install_default_drivers},
};

/// The flavor of SQL (Postgres, SQLite) is inferred from the scheme of the database URL.
pub type DbPool = AnyPool;

pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    install_default_drivers();
    AnyPoolOptions::new()
        .max_connections(100)
        .idle_timeout(Some(Duration::from_millis(1000)))
        .connect(database_url)
        .await
}

/// Connects to the database in `DATABASE_URL`; panics if it is missing or unreachable.
pub async fn connect_from_env() -> DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|error| panic!("Error getting DATABASE_URL: {error}"));
    let pool = connect(&database_url)
        .await
        .unwrap_or_else(|error| panic!("Could not create pool: {error}"));
    println!("Created SQL pool from DB url");
    pool
}
