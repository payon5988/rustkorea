//! Outbound fetches of admin-supplied URLs (blog import) without SSRF.
//!
//! Every hop's host is resolved here and all of its addresses must be
//! public. The request is then pinned to the checked address, so a second
//! DNS answer cannot point it inward. Redirects are followed manually (each
//! hop re-checked) and the body is read in chunks up to a byte cap.

use std::net::{IpAddr, SocketAddr};

pub const MAX_REDIRECTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchError {
    /// Not an absolute http(s) URL.
    Url,
    /// Resolves to a loopback/private/link-local/reserved address.
    Blocked,
    /// DNS, connect, non-2xx, or too many redirects.
    Network,
    /// Body exceeds the cap.
    TooLarge,
}

impl FetchError {
    /// `?error=` code for redirects.
    pub fn code(self) -> &'static str {
        match self {
            FetchError::Url => "url",
            FetchError::Blocked => "blocked",
            FetchError::Network => "fetch",
            FetchError::TooLarge => "size",
        }
    }
}

/// Globally routable unicast address (IPv4-mapped IPv6 judged as IPv4).
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_multicast()
                || o[0] == 0
                || (o[0] == 100 && (o[1] & 0xc0) == 64) // 100.64.0.0/10 CGNAT
                || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24
                || (o[0] == 198 && (o[1] & 0xfe) == 18) // 198.18.0.0/15
                || o[0] >= 240) // reserved
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            !(v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (s[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
                || (s[0] & 0xffc0) == 0xfe80 // link local fe80::/10
                || (s[0] == 0x2001 && s[1] == 0x0db8)) // documentation
        }
    }
}

/// Resolve `url`'s host; every address must be public. Returns one to pin.
async fn resolve_public(url: &reqwest::Url) -> Result<SocketAddr, FetchError> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(FetchError::Url);
    }
    let host = url.host_str().ok_or(FetchError::Url)?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let port = url.port_or_known_default().ok_or(FetchError::Url)?;
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| FetchError::Network)?
        .collect();
    match addrs.first() {
        None => Err(FetchError::Network),
        Some(_) if addrs.iter().any(|a| !is_public_ip(a.ip())) => Err(FetchError::Blocked),
        Some(first) => Ok(*first),
    }
}

/// GET a public URL as text, refusing internal targets and bodies over
/// `max_bytes`.
pub async fn fetch_public_text(raw_url: &str, max_bytes: usize) -> Result<String, FetchError> {
    let mut url = reqwest::Url::parse(raw_url).map_err(|_| FetchError::Url)?;
    for _ in 0..=MAX_REDIRECTS {
        let addr = resolve_public(&url).await?;
        let host = url.host_str().ok_or(FetchError::Url)?.to_string();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                crate::config::http_timeout_secs(),
            ))
            .user_agent(crate::config::http_user_agent())
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .resolve(&host, addr)
            .build()
            .map_err(|_| FetchError::Network)?;
        let mut resp = client
            .get(url.clone())
            .send()
            .await
            .map_err(|_| FetchError::Network)?;
        if resp.status().is_redirection() {
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or(FetchError::Network)?;
            url = url.join(location).map_err(|_| FetchError::Url)?;
            continue;
        }
        if !resp.status().is_success() {
            return Err(FetchError::Network);
        }
        if resp.content_length().is_some_and(|n| n > max_bytes as u64) {
            return Err(FetchError::TooLarge);
        }
        let mut body = Vec::new();
        while let Some(chunk) = resp.chunk().await.map_err(|_| FetchError::Network)? {
            if body.len() + chunk.len() > max_bytes {
                return Err(FetchError::TooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        return Ok(String::from_utf8_lossy(&body).into_owned());
    }
    Err(FetchError::Network)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_addresses() {
        for ip in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
            assert!(is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.17.0.1",
            "192.168.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "::1",
            "fd00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
        ] {
            assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
        }
    }

    #[tokio::test]
    async fn refuses_internal_and_non_http_targets() {
        for (url, want) in [
            ("http://127.0.0.1:5432/", FetchError::Blocked),
            ("http://localhost/", FetchError::Blocked),
            (
                "http://169.254.169.254/latest/meta-data/",
                FetchError::Blocked,
            ),
            ("http://[::1]/", FetchError::Blocked),
            ("http://[::ffff:127.0.0.1]/", FetchError::Blocked),
            ("http://0x7f000001/", FetchError::Blocked),
            ("ftp://example.com/", FetchError::Url),
            ("file:///etc/passwd", FetchError::Url),
            ("not a url", FetchError::Url),
        ] {
            assert_eq!(fetch_public_text(url, 1024).await, Err(want), "{url}");
        }
    }

    /// Needs internet; run with `RUN_NETWORK_TESTS=1 cargo test outbound`.
    #[tokio::test]
    async fn fetches_public_page_and_caps_size() {
        if std::env::var("RUN_NETWORK_TESTS").as_deref() != Ok("1") {
            return;
        }
        let html = fetch_public_text("https://example.com/", 100_000)
            .await
            .unwrap();
        assert!(html.contains("Example Domain"));
        // github.com answers http with 301 -> https; the hop is re-checked.
        let html = fetch_public_text("http://github.com/", 2_000_000)
            .await
            .unwrap();
        assert!(html.contains("GitHub"));
        assert_eq!(
            fetch_public_text("https://example.com/", 10).await,
            Err(FetchError::TooLarge)
        );
    }
}
