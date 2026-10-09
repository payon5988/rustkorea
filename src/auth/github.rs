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

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

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
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            match idx {
                0..10 => (b'0' + idx) as char,
                10..36 => (b'a' + idx - 10) as char,
                36..62 => (b'A' + idx - 36) as char,
                _ => 'a',
            }
        })
        .collect()
}

fn hardened_state_cookie(state: String) -> Cookie<'static> {
    let mut c = Cookie::new("oauth_state", state);
    c.set_path("/");
    c.set_http_only(true);
    c.set_same_site(topcoat::cookie::SameSite::Lax);
    c.set_max_age(topcoat::cookie::time::Duration::minutes(10));
    if std::env::var("OAUTH_SECURE_COOKIES").as_deref() == Ok("1") {
        c.set_secure(true);
    }
    c
}

#[route(GET "/auth/github")]
pub async fn github_login(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let client_id = crate::config::env_or("GITHUB_CLIENT_ID", "");
    let redirect_uri = crate::config::env_or(
        "GITHUB_REDIRECT_URI",
        "http://localhost:3000/auth/github/callback",
    );
    let state = generate_state();

    cookies(cx).add(hardened_state_cookie(state.clone()));

    let url = format!(
        "{GITHUB_AUTH_URL}?client_id={client_id}&redirect_uri={redirect_uri}&scope=read:user&state={state}"
    );
    Ok(topcoat::router::error::see_other(&url))
}

#[route(GET "/auth/github/callback")]
pub async fn github_callback(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let code = extract_query_param(cx, "code");
    let state = extract_query_param(cx, "state");

    let expected_state = cookies(cx)
        .get("oauth_state")
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    {
        let mut c = Cookie::new("oauth_state", "");
        c.set_path("/");
        cookies(cx).remove(c);
    }

    if code.is_empty() || state.is_empty() || state != expected_state {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?error=oauth_state",
        ));
    }

    let client_id = crate::config::env_or("GITHUB_CLIENT_ID", "");
    let client_secret = crate::config::env_or("GITHUB_CLIENT_SECRET", "");

    if client_id.is_empty() || client_secret.is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?error=oauth_not_configured",
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("rustkorea-hub")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = match client
        .post(GITHUB_TOKEN_URL)
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                "/auth/login?error=oauth_network",
            ));
        }
    };

    let token_resp: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                "/auth/login?error=oauth_token",
            ));
        }
    };

    let access_token = match token_resp["access_token"].as_str() {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/auth/login?error=oauth_access_denied",
            ));
        }
    };

    let user_resp: serde_json::Value = match client
        .get(GITHUB_USER_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "rustkorea-hub")
        .send()
        .await
    {
        Ok(r) => match r.json().await {
            Ok(v) => v,
            Err(_) => {
                return Ok(topcoat::router::error::see_other(
                    "/auth/login?error=oauth_user_info",
                ));
            }
        },
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                "/auth/login?error=oauth_network",
            ));
        }
    };

    let github_id = user_resp["id"].as_i64().unwrap_or(0);
    if github_id == 0 {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?error=oauth_user_info",
        ));
    }
    let username = user_resp["login"].as_str().unwrap_or("unknown").to_string();
    let avatar_url = user_resp["avatar_url"].as_str().map(String::from);
    let avatar_url: Option<String> =
        avatar_url.and_then(|u| crate::services::markdown::safe_http_url(&u));

    let db: &Db = app_context(cx);

    let status = crate::auth::access::initial_status(db).await;

    sqlx::query(
        r#"
        INSERT INTO users (github_id, username, avatar_url, role_id, status, created_at, updated_at)
        VALUES ($1, $2, $3, 'member', $4, NOW()::text, NOW()::text)
        ON CONFLICT (github_id) DO UPDATE SET
            username = EXCLUDED.username,
            avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()::text
        "#,
    )
    .bind(github_id)
    .bind(&username)
    .bind(&avatar_url)
    .bind(&status)
    .execute(db)
    .await
    .map_err(|e| eprintln!("[oauth] user upsert failed: {e}"))
    .ok();

    let user: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE github_id = $1")
        .bind(github_id)
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
                        "/auth/login?error=session",
                    ));
                }
            }
            LoginDecision::Pending => {
                return Ok(topcoat::router::error::see_other("/auth/login?error=5"));
            }
            LoginDecision::Suspended => {
                return Ok(topcoat::router::error::see_other("/auth/login?error=6"));
            }
        }
    }

    Ok(topcoat::router::error::see_other("/"))
}
