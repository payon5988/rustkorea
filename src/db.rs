use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub type Db = PgPool;

/// Connection-pool tuning (single place; not scattered across call sites).
pub const DB_MAX_CONNECTIONS: u32 = 20;
pub const DB_MIN_CONNECTIONS: u32 = 2;
pub const DB_ACQUIRE_TIMEOUT_SECS: u64 = 10;
pub const DB_IDLE_TIMEOUT_SECS: u64 = 300;
pub const DB_MAX_LIFETIME_SECS: u64 = 1800;

pub async fn connect(database_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .min_connections(DB_MIN_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(DB_ACQUIRE_TIMEOUT_SECS))
        .idle_timeout(Duration::from_secs(DB_IDLE_TIMEOUT_SECS))
        .max_lifetime(Duration::from_secs(DB_MAX_LIFETIME_SECS))
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &Db) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
