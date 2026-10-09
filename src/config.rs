//! Central runtime configuration. Every environment-dependent value lives
//! here (read from env with sane defaults) instead of being hardcoded
//! across page handlers.

use std::env;

pub fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.into())
}

/// Canonical public origin, e.g. `https://rustkorea.kr`.
/// Used for sitemap, feed, robots.txt and absolute links.
pub fn site_url() -> String {
    let url = env_or("SITE_URL", "https://rustkorea.kr");
    url.trim_end_matches('/').to_string()
}

/// Database URL (PostgreSQL).
pub fn database_url() -> String {
    env_or(
        "DATABASE_URL",
        "postgres://postgres:postgres@127.0.0.1:5435/rustkorea",
    )
}

/// Default YouTube channel synced by the background worker.
pub fn youtube_channel_id() -> String {
    env_or("YOUTUBE_CHANNEL_ID", "UCeVVCp2CRrcmUQ8SuvCdSyg")
}

/// Display name used in worker logs.
pub fn youtube_channel_name() -> String {
    env_or("YOUTUBE_CHANNEL_NAME", "Rust Korea")
}

/// Crate version (used for service-worker cache busting).
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Android package name for Trusted Web Activity asset links.
/// Empty = assetlinks endpoint returns 404 (no association advertised).
pub fn twa_package_name() -> String {
    env_or("TWA_PACKAGE_NAME", "")
}

/// Comma-separated SHA-256 cert fingerprints for `assetlinks.json`.
pub fn twa_fingerprints() -> Vec<String> {
    env_or("TWA_SHA256_FINGERPRINTS", "")
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Contact domain used for generated author addresses in the RSS feed.
pub fn mail_domain() -> String {
    env_or("MAIL_DOMAIN", "rustkorea.kr")
}

/// Public contact address shown in legal pages and used as a fallback
/// identifier where an email is required.
pub fn contact_email() -> String {
    env_or("ADMIN_EMAIL", "admin@rustkorea.kr")
}
