// Component functions use PascalCase by framework convention.
#![allow(non_snake_case)]

mod admin_ui;
mod app;
mod auth;
mod components;
mod config;
mod db;
mod models;
mod pages;
mod security;
mod services;
mod util;

use topcoat::{
    asset::RouterBuilderAssetExt,
    cookie::RouterBuilderCookieExt,
    router::{Router, RouterBuilderDiscoverExt},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config::database_url());

    let pool = db::connect(&database_url)
        .await
        .expect("failed to connect to database");

    db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    // Seed sample data
    services::seed::seed_database(&pool).await;

    // Start YouTube background worker (aborted on shutdown below).
    let worker_pool = pool.clone();
    let worker = tokio::spawn(async move {
        services::youtube_worker::start_youtube_worker(worker_pool).await;
    });

    let router = Router::builder()
        .app_context(pool.clone())
        .cookies()
        .sessions(
            topcoat::session::SessionConfig::builder()
                .token_store(crate::auth::cookie_store::EnvCookieStore)
                .build(),
        )
        .assets(topcoat::asset::AssetBundle::load().expect("asset bundle not found"))
        .layer(security::SecurityHeaders::new())
        .runtime()
        .discover()
        .build();

    let res = topcoat::start(router).await;
    worker.abort();
    pool.close().await;
    res.unwrap();
}
