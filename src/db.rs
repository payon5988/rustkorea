use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub type Db = PgPool;

pub async fn connect(database_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &Db) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
