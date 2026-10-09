use topcoat::{Result, context::Cx, cookie::Cookies};

use crate::db::Db;
use crate::models::User;

pub const SESSION_LIFETIME_SECS: i64 = 30 * 24 * 3600;
const LOGIN_WINDOW_SECS: i64 = 300;
const LOGIN_MAX_ATTEMPTS: i64 = 10;

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn generate_csrf_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            match idx {
                0..10 => (b'0' + idx) as char,
                10..36 => (b'a' + idx - 10) as char,
                _ => (b'A' + idx - 36) as char,
            }
        })
        .collect()
}

/// Create a server-side session for `user_id` and issue the client token.
/// Always generates a fresh token (protects against session fixation).
pub async fn create_session(db: &Db, cx: &Cx, user_id: &str) -> Result<()> {
    let session = topcoat::session::start(cx).await.map_err(|e| {
        eprintln!("[session] start failed: {e}");
        e
    })?;
    let token_hash = hex_encode(&session.token_hash[..]);
    let csrf_token = generate_csrf_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(SESSION_LIFETIME_SECS);
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, csrf_token, expires_at, pending_2fa)
         VALUES ($1, $2, $3, $4, 0)
         ON CONFLICT (token_hash) DO UPDATE SET user_id = EXCLUDED.user_id, csrf_token = EXCLUDED.csrf_token, expires_at = EXCLUDED.expires_at, pending_2fa = 0",
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(&csrf_token)
    .bind(expires_at)
    .execute(db)
    .await
    .map_err(|e| {
        eprintln!("[session] insert failed: {e}");
        topcoat::Error::from(e)
    })?;
    sqlx::query("UPDATE users SET last_login_at = NOW()::text WHERE id = $1")
        .bind(user_id)
        .execute(db)
        .await
        .ok();
    Ok(())
}

/// Resolve the current request's user via the server-side session table.
/// Expired, unknown, suspended, or pending sessions return `None`.
pub async fn current_user(db: &Db, cx: &Cx) -> Option<User> {
    let hash = topcoat::session::token_hash(cx).await.ok()??;
    let token_hash = hex_encode(&hash[..]);
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u JOIN sessions s ON s.user_id = u.id
         WHERE s.token_hash = $1 AND s.expires_at > NOW() AND s.pending_2fa = 0",
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    let user = user?;
    if user.status != "active" {
        return None;
    }
    Some(user)
}

/// Create a pre-2FA session: authenticated password, TOTP still pending.
/// Such sessions are invisible to [`current_user`] until completed.
pub async fn create_pending_session(db: &Db, cx: &Cx, user_id: &str) -> Result<()> {
    let session = topcoat::session::start(cx).await.map_err(|e| {
        eprintln!("[session] start failed: {e}");
        e
    })?;
    let token_hash = hex_encode(&session.token_hash[..]);
    let csrf_token = generate_csrf_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, csrf_token, expires_at, pending_2fa)
         VALUES ($1, $2, $3, $4, 1)
         ON CONFLICT (token_hash) DO UPDATE SET user_id = EXCLUDED.user_id, csrf_token = EXCLUDED.csrf_token, expires_at = EXCLUDED.expires_at, pending_2fa = 1",
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(&csrf_token)
    .bind(expires_at)
    .execute(db)
    .await
    .map_err(|e| {
        eprintln!("[session] insert failed: {e}");
        topcoat::Error::from(e)
    })?;
    Ok(())
}

/// Resolve the user behind a pre-2FA session, if any.
pub async fn pending_user(db: &Db, cx: &Cx) -> Option<User> {
    let hash = topcoat::session::token_hash(cx).await.ok()??;
    let token_hash = hex_encode(&hash[..]);
    sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u JOIN sessions s ON s.user_id = u.id
         WHERE s.token_hash = $1 AND s.expires_at > NOW() AND s.pending_2fa = 1",
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
}

/// Promote a pre-2FA session to a full session after TOTP verification.
pub async fn complete_2fa(db: &Db, cx: &Cx) -> bool {
    let Ok(Some(hash)) = topcoat::session::token_hash(cx).await else {
        return false;
    };
    let token_hash = hex_encode(&hash[..]);
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(SESSION_LIFETIME_SECS);
    sqlx::query(
        "UPDATE sessions SET pending_2fa = 0, expires_at = $1
         WHERE token_hash = $2 AND pending_2fa = 1",
    )
    .bind(expires_at)
    .bind(&token_hash)
    .execute(db)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0)
        == 1
}

/// Destroy the current session (client token + server row).
pub async fn destroy_session(db: &Db, cx: &Cx) {
    if let Ok(Some(hash)) = topcoat::session::stop(cx).await {
        let token_hash = hex_encode(&hash[..]);
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(db)
            .await
            .ok();
    }
}

/// Remove all sessions belonging to a user (password change, admin revoke).
pub async fn destroy_user_sessions(db: &Db, user_id: &str) {
    if let Err(e) = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(db)
        .await
    {
        eprintln!("[session] destroy user sessions failed: {e}");
    }
}

/// Delete expired session rows. Called opportunistically on login.
pub async fn prune_expired_sessions(db: &Db) {
    sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()::text")
        .execute(db)
        .await
        .ok();
}

/// Best-effort client IP for rate limiting (X-Forwarded-For aware).
pub fn client_ip(cx: &Cx) -> String {
    let headers = topcoat::router::request::headers(cx);
    if let Some(fwd) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = fwd.split(',').next().map(str::trim) {
            if !first.is_empty() {
                return first.to_string();
            }
        }
    }
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// Sliding-window login rate limit: at most LOGIN_MAX_ATTEMPTS attempts per
/// IP (or identifier) in LOGIN_WINDOW_SECS.
pub async fn rate_limit_ok(db: &Db, cx: &Cx, identifier: &str) -> bool {
    let ip = client_ip(cx);
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM login_attempts
         WHERE attempted_at > NOW() - make_interval(secs => $1)
           AND (ip = $2 OR identifier = $3)",
    )
    .bind(LOGIN_WINDOW_SECS as f64)
    .bind(&ip)
    .bind(identifier)
    .fetch_one(db)
    .await
    .unwrap_or((0,));
    count.0 < LOGIN_MAX_ATTEMPTS
}

pub async fn record_login_attempt(db: &Db, cx: &Cx, identifier: &str) {
    let ip = client_ip(cx);
    sqlx::query("INSERT INTO login_attempts (ip, identifier) VALUES ($1, $2)")
        .bind(&ip)
        .bind(identifier)
        .execute(db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM login_attempts WHERE attempted_at < NOW() - make_interval(secs => 3600)",
    )
    .execute(db)
    .await
    .ok();
}

/// Clear recorded attempts after a successful authentication so legitimate
/// users are not stuck behind their own failed tries.
pub async fn clear_login_attempts(db: &Db, cx: &Cx, identifier: &str) {
    let ip = client_ip(cx);
    sqlx::query("DELETE FROM login_attempts WHERE ip = $1 AND identifier = $2")
        .bind(&ip)
        .bind(identifier)
        .execute(db)
        .await
        .ok();
}

/// CSRF mitigation for state-changing POSTs: require a same-origin
/// Origin/Referer when the browser sends one. SameSite=Lax session cookies
/// (framework default) are the primary defense; this is defense in depth.
pub fn same_origin_ok(cx: &Cx) -> bool {
    let headers = topcoat::router::request::headers(cx);
    let origin = headers.get("origin").and_then(|v| v.to_str().ok());
    let referer = headers.get("referer").and_then(|v| v.to_str().ok());
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let check = |url: &str| -> bool {
        let after_scheme = url.split("://").nth(1).unwrap_or(url);
        let origin_host = after_scheme.split('/').next().unwrap_or("");
        origin_host.eq_ignore_ascii_case(host) || host.is_empty()
    };
    match (origin, referer) {
        (Some(o), _) => check(o),
        (None, Some(r)) => check(r),
        (None, None) => true,
    }
}

/// Escape LIKE wildcards so user input can't broaden a search pattern.
pub fn escape_like(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Minimum password policy: 8+ chars (admin bootstrap requires 10+).
pub fn password_ok(password: &str) -> bool {
    password.len() >= 8 && password.len() <= 128
}

/// Minimal form carrying only a CSRF token (for button-style POSTs).
#[derive(serde::Deserialize)]
pub struct CsrfForm {
    pub csrf_token: String,
}

/// This session's CSRF token (empty when anonymous).
pub async fn csrf_token(db: &Db, cx: &Cx) -> String {
    let Ok(Some(hash)) = topcoat::session::token_hash(cx).await else {
        return String::new();
    };
    let token_hash = hex_encode(&hash[..]);
    sqlx::query_scalar::<_, String>("SELECT csrf_token FROM sessions WHERE token_hash = $1")
        .bind(&token_hash)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

/// Verify a submitted CSRF token against the session row.
/// Fails closed on any error (including anonymous callers).
pub async fn verify_csrf(db: &Db, cx: &Cx, submitted: &str) -> bool {
    if submitted.is_empty() {
        return false;
    }
    let expected = csrf_token(db, cx).await;
    !expected.is_empty() && expected == submitted
}

/// [`verify_csrf`] with audit logging on failure. `scope` names the
/// protected operation (e.g. `"admin.users.create"`).
pub async fn verify_csrf_logged(db: &Db, cx: &Cx, submitted: &str, scope: &str) -> bool {
    if verify_csrf(db, cx, submitted).await {
        return true;
    }
    crate::auth::access::audit(db, cx, "", "csrf.blocked", "", "", scope).await;
    false
}

/// Issue a double-submit CSRF seed for anonymous forms (login/register):
/// sets an HttpOnly cookie and returns the value to embed in the form.
pub fn issue_csrf_seed(cx: &Cx) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let seed: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            match idx {
                0..10 => (b'0' + idx) as char,
                10..36 => (b'a' + idx - 10) as char,
                _ => (b'A' + idx - 36) as char,
            }
        })
        .collect();
    let mut cookie = topcoat::cookie::Cookie::new("csrf_seed", seed.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(topcoat::cookie::SameSite::Lax);
    cookie.set_max_age(topcoat::cookie::time::Duration::minutes(10));
    if std::env::var("OAUTH_SECURE_COOKIES").as_deref() == Ok("1") {
        cookie.set_secure(true);
    }
    topcoat::cookie::cookies(cx).add(cookie);
    seed
}

/// Verify a double-submit seed (anonymous forms).
pub fn verify_csrf_seed(cx: &Cx, submitted: &str) -> bool {
    if submitted.is_empty() {
        return false;
    }
    let expected = topcoat::cookie::cookies(cx)
        .get("csrf_seed")
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    !expected.is_empty() && expected == submitted
}

/// [`verify_csrf_seed`] with audit logging on failure.
pub async fn verify_csrf_seed_logged(db: &Db, cx: &Cx, submitted: &str, scope: &str) -> bool {
    if verify_csrf_seed(cx, submitted) {
        return true;
    }
    crate::auth::access::audit(db, cx, "", "csrf.blocked", "", "", scope).await;
    false
}
