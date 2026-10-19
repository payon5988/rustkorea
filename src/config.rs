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

/// Local upload directory (product images). Served at `/uploads/*`.
pub fn upload_dir() -> String {
    env_or("UPLOAD_DIR", "data/uploads")
}

/// YouTube sync interval (seconds).
pub fn youtube_poll_secs() -> u64 {
    env_or("YOUTUBE_POLL_SECS", "900").parse().unwrap_or(900)
}

/// Outbound HTTP timeout (seconds) for OAuth / feed fetches.
pub fn http_timeout_secs() -> u64 {
    env_or("HTTP_TIMEOUT_SECS", "10").parse().unwrap_or(10)
}

/// Shared User-Agent for outbound HTTP (includes crate version).
pub fn http_user_agent() -> String {
    format!("rustkorea-hub/{}", app_version())
}

/// Reverse proxies allowed to set `X-Forwarded-For` (comma-separated IPs or
/// CIDRs). Defaults to loopback + private ranges: the app port is only
/// published to 127.0.0.1 / the compose network, never to the internet.
/// Invalid entries are skipped with a warning.
pub fn trusted_proxies() -> Vec<String> {
    env_or(
        "TRUSTED_PROXIES",
        "127.0.0.0/8,::1/128,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16",
    )
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .filter(|s| {
        let ok = valid_ip_or_cidr(s);
        if !ok {
            eprintln!("[config] TRUSTED_PROXIES: invalid entry skipped: {s}");
        }
        ok
    })
    .map(String::from)
    .collect()
}

fn valid_ip_or_cidr(s: &str) -> bool {
    let (addr, len) = match s.split_once('/') {
        Some((a, l)) => (a, Some(l)),
        None => (s, None),
    };
    let Ok(ip) = addr.parse::<std::net::IpAddr>() else {
        return false;
    };
    let max = if ip.is_ipv4() { 32 } else { 128 };
    len.is_none_or(|l| l.parse::<u8>().is_ok_and(|n| n <= max))
}

/// Public contact address shown in legal pages and used as a fallback
/// identifier where an email is required.
pub fn contact_email() -> String {
    env_or("ADMIN_EMAIL", "admin@rustkorea.kr")
}

#[cfg(test)]
mod tests {
    use super::valid_ip_or_cidr;

    #[test]
    fn proxy_entries_validated() {
        for ok in ["127.0.0.1", "10.0.0.0/8", "::1/128", "172.16.0.0/12"] {
            assert!(valid_ip_or_cidr(ok), "{ok}");
        }
        for bad in ["", "localhost", "10.0.0.0/33", "::1/129", "1.2.3.4/x"] {
            assert!(!valid_ip_or_cidr(bad), "{bad}");
        }
    }
}
