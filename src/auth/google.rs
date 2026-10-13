use topcoat::router::request::uri;
use topcoat::{
    Result,
    context::{Cx, app_context},
    cookie::{Cookie, Cookies, cookies},
    router::route,
};

use crate::auth::session as sessions;
use crate::db::Db;
use crate::models::User;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

fn extract_query_param(cx: &Cx, key: &str) -> String {
    let query = uri(cx).query().unwrap_or("");
    form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default()
}

fn generate_state() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..crate::auth::session::CSRF_TOKEN_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..crate::auth::session::CSRF_ALPHABET);
            if idx < crate::auth::session::CSRF_DIGIT_END {
                (b'0' + idx) as char
            } else if idx < crate::auth::session::CSRF_LOWER_END {
                (b'a' + idx - crate::auth::session::CSRF_DIGIT_END) as char
            } else {
                (b'A' + idx - crate::auth::session::CSRF_LOWER_END) as char
            }
        })
        .collect()
}

fn hardened_state_cookie(state: String) -> Cookie<'static> {
    let mut c = Cookie::new(crate::auth::session::OAUTH_STATE_COOKIE, state);
    c.set_path("/");
    c.set_http_only(true);
    c.set_same_site(topcoat::cookie::SameSite::Lax);
    c.set_max_age(topcoat::cookie::time::Duration::minutes(
        crate::auth::session::PENDING_TTL_MINUTES,
    ));
    if std::env::var("OAUTH_SECURE_COOKIES").as_deref() == Ok("1") {
        c.set_secure(true);
    }
    c
}

/// Google userinfo `id` is a JSON string (numeric, exceeds i64 range).
/// Parse it as text; never coerce to integer.
fn google_sub(value: &serde_json::Value) -> String {
    if let Some(s) = value.get("id").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    value
        .get("sub")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

#[route(GET "/auth/google")]
pub async fn google_login(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let client_id = crate::config::env_or("GOOGLE_CLIENT_ID", "");
    let redirect_uri = {
        let v = crate::config::env_or("GOOGLE_REDIRECT_URI", "");
        if v.is_empty() {
            crate::config::site_url() + "/auth/google/callback"
        } else {
            v
        }
    };
    if client_id.is_empty() || crate::config::env_or("GOOGLE_CLIENT_SECRET", "").is_empty() {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_error_url(
                crate::auth::session::login_error::OAUTH_NOT_CONFIGURED,
            ),
        ));
    }
    let state = generate_state();

    cookies(cx).add(hardened_state_cookie(state.clone()));

    let url = format!(
        "{GOOGLE_AUTH_URL}?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=openid+email+profile&state={state}&access_type=offline"
    );
    Ok(topcoat::router::error::see_other(&url))
}

#[route(GET "/auth/google/callback")]
pub async fn google_callback(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let code = extract_query_param(cx, "code");
    let state = extract_query_param(cx, "state");

    let expected_state = cookies(cx)
        .get(crate::auth::session::OAUTH_STATE_COOKIE)
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    {
        let mut c = Cookie::new(crate::auth::session::OAUTH_STATE_COOKIE, "");
        c.set_path("/");
        cookies(cx).remove(c);
    }

    if code.is_empty() || state.is_empty() || state != expected_state {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_error_url(crate::auth::session::login_error::OAUTH_STATE),
        ));
    }

    let client_id = crate::config::env_or("GOOGLE_CLIENT_ID", "");
    let client_secret = crate::config::env_or("GOOGLE_CLIENT_SECRET", "");
    let redirect_uri = {
        let v = crate::config::env_or("GOOGLE_REDIRECT_URI", "");
        if v.is_empty() {
            crate::config::site_url() + "/auth/google/callback"
        } else {
            v
        }
    };

    if client_id.is_empty() || client_secret.is_empty() {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_error_url(
                crate::auth::session::login_error::OAUTH_NOT_CONFIGURED,
            ),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            crate::config::http_timeout_secs(),
        ))
        .user_agent(crate::config::http_user_agent())
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = match client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                crate::auth::session::login_error_url(
                    crate::auth::session::login_error::OAUTH_NETWORK,
                ),
            ));
        }
    };

    let token_resp: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                crate::auth::session::login_error_url(
                    crate::auth::session::login_error::OAUTH_TOKEN,
                ),
            ));
        }
    };

    let access_token = match token_resp["access_token"].as_str() {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Ok(topcoat::router::error::see_other(
                crate::auth::session::login_error_url(
                    crate::auth::session::login_error::OAUTH_ACCESS_DENIED,
                ),
            ));
        }
    };

    let user_resp: serde_json::Value = match client
        .get(GOOGLE_USER_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
    {
        Ok(r) => match r.json().await {
            Ok(v) => v,
            Err(_) => {
                return Ok(topcoat::router::error::see_other(
                    crate::auth::session::login_error_url(
                        crate::auth::session::login_error::OAUTH_USER_INFO,
                    ),
                ));
            }
        },
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                crate::auth::session::login_error_url(
                    crate::auth::session::login_error::OAUTH_NETWORK,
                ),
            ));
        }
    };

    let google_id = google_sub(&user_resp);
    if google_id.is_empty() {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_error_url(
                crate::auth::session::login_error::OAUTH_USER_INFO,
            ),
        ));
    }
    let email = user_resp["email"].as_str().unwrap_or("").to_string();
    let display_name = user_resp["name"].as_str().unwrap_or("").to_string();
    let avatar_url = user_resp["picture"].as_str().map(String::from);
    let avatar_url: Option<String> =
        avatar_url.and_then(|u| crate::services::markdown::safe_http_url(&u));

    let username = if !email.is_empty() {
        email.split('@').next().unwrap_or("user").to_string()
    } else {
        format!("google_{google_id}")
    };

    let db: &Db = app_context(cx);

    let status = crate::auth::access::initial_status(db).await;

    sqlx::query(
        r#"
        INSERT INTO users (google_id, email, username, display_name, avatar_url, role_id, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW()::text, NOW()::text)
        ON CONFLICT (google_id) DO UPDATE SET
            email = EXCLUDED.email,
            display_name = EXCLUDED.display_name,
            avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()::text
        "#,
    )
    .bind(&google_id)
    .bind(&email)
    .bind(&username)
    .bind(&display_name)
    .bind(&avatar_url)
    .bind(crate::auth::access::DEFAULT_ROLE)
    .bind(&status)
    .execute(db)
    .await
    .map_err(|e| eprintln!("[oauth] user upsert failed: {e}"))
    .ok();

    let user: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE google_id = $1")
        .bind(&google_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if let Some(u) = user {
        use crate::auth::access::LoginDecision;
        match crate::auth::access::login_decision(&u) {
            LoginDecision::Allow => {
                sessions::prune_expired_sessions(db).await;
                if sessions::create_session(db, cx, &u.id).await.is_err() {
                    return Ok(topcoat::router::error::see_other(
                        crate::auth::session::login_error_url(
                            crate::auth::session::login_error::SESSION,
                        ),
                    ));
                }
            }
            LoginDecision::Pending => {
                return Ok(topcoat::router::error::see_other(
                    crate::auth::session::login_error_url(
                        crate::auth::session::login_error::PENDING_APPROVAL,
                    ),
                ));
            }
            LoginDecision::Suspended => {
                return Ok(topcoat::router::error::see_other(
                    crate::auth::session::login_error_url(
                        crate::auth::session::login_error::SUSPENDED,
                    ),
                ));
            }
        }
    }

    Ok(topcoat::router::error::see_other("/"))
}
