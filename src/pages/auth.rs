use std::future::Future;
use topcoat::router::request::uri;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, route},
    view::{View, view},
};

use crate::auth::session as sessions;
use crate::db::Db;
use crate::models::User;

async fn get_setting(db: &Db, key: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM site_settings WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub csrf_token: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
    pub csrf_token: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

async fn get_current_user_id(db: &Db, cx: &Cx) -> Option<String> {
    sessions::current_user(db, cx).await.map(|u| u.id)
}

fn requested_redirect(cx: &Cx) -> Option<String> {
    let query = uri(cx).query()?;
    form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == "redirect")
        .map(|(_, v)| v.into_owned())
        .filter(|r| r.starts_with('/') && !r.starts_with("//"))
}

fn safe_redirect_target(cx: &Cx) -> String {
    requested_redirect(cx).unwrap_or_else(|| "/".to_string())
}

fn encode_redirect(value: &str) -> String {
    form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[route(POST "/auth/login")]
pub async fn login_submit(
    cx: &Cx,
    Form(input): Form<LoginForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    let target = safe_redirect_target(cx);
    let fail_target = format!("/auth/login?error=1&redirect={}", encode_redirect(&target));

    if !sessions::same_origin_ok(cx) {
        return Ok(topcoat::router::error::see_other(&fail_target));
    }
    if !sessions::verify_csrf_seed_logged(db, cx, &input.csrf_token, "auth.login").await {
        return Ok(topcoat::router::error::see_other(&fail_target));
    }
    if !sessions::rate_limit_ok(db, cx, &input.email).await {
        return Ok(topcoat::router::error::see_other(&fail_target));
    }
    sessions::record_login_attempt(db, cx, &input.email).await;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND password_hash IS NOT NULL",
    )
    .bind(&input.email)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    if let Some(u) = user {
        use crate::auth::access::LoginDecision;
        use argon2::Argon2;
        use argon2::password_hash::{PasswordHash, PasswordVerifier};

        if let Ok(hash) = PasswordHash::new(u.password_hash.as_deref().unwrap_or("")) {
            let argon2 = Argon2::default();
            if argon2
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok()
            {
                match crate::auth::access::login_decision(&u) {
                    LoginDecision::Allow => {
                        sessions::prune_expired_sessions(db).await;
                        if sessions::create_session(db, cx, &u.id).await.is_ok() {
                            sessions::clear_login_attempts(db, cx, &input.email).await;
                            return Ok(topcoat::router::error::see_other(&target));
                        }
                    }
                    decision => {
                        let code = if decision == LoginDecision::Pending {
                            "5"
                        } else {
                            "6"
                        };
                        return Ok(topcoat::router::error::see_other(format!(
                            "/auth/login?error={code}&redirect={}",
                            encode_redirect(&target)
                        )));
                    }
                }
            }
        }
    }

    Ok(topcoat::router::error::see_other(&fail_target))
}

#[route(POST "/auth/register")]
pub async fn register_submit(
    cx: &Cx,
    Form(input): Form<RegisterForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !sessions::same_origin_ok(cx) {
        return Ok(topcoat::router::error::see_other("/auth/register?error=1"));
    }
    if !sessions::verify_csrf_seed_logged(db, cx, &input.csrf_token, "auth.register").await {
        return Ok(topcoat::router::error::see_other("/auth/register?error=1"));
    }
    if !sessions::rate_limit_ok(db, cx, &input.email).await {
        return Ok(topcoat::router::error::see_other("/auth/register?error=4"));
    }
    sessions::record_login_attempt(db, cx, &input.email).await;

    if input.password != input.password_confirm {
        return Ok(topcoat::router::error::see_other("/auth/register?error=1"));
    }

    if !sessions::password_ok(&input.password) {
        return Ok(topcoat::router::error::see_other("/auth/register?error=2"));
    }

    if input.username.trim().is_empty() || input.username.len() > 32 {
        return Ok(topcoat::router::error::see_other("/auth/register?error=1"));
    }

    let existing: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&input.email)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if existing.is_some() {
        return Ok(topcoat::router::error::see_other("/auth/register?error=3"));
    }

    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};

    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(input.password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();
    if password_hash.is_empty() {
        return Ok(topcoat::router::error::see_other("/auth/register?error=1"));
    }

    let status = crate::auth::access::initial_status(db).await;

    let inserted = sqlx::query(
        "INSERT INTO users (email, username, password_hash, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, NOW()::text, NOW()::text)",
    )
    .bind(&input.email)
    .bind(&input.username)
    .bind(&password_hash)
    .bind(&status)
    .execute(db)
    .await;

    if inserted.map(|r| r.rows_affected()).unwrap_or(0) != 1 {
        return Ok(topcoat::router::error::see_other("/auth/register?error=3"));
    }

    if status == "pending" {
        return Ok(topcoat::router::error::see_other("/auth/login?error=5"));
    }

    let user: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&input.email)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if let Some(u) = user {
        // Only log in as the account we just created (guard against races).
        if u.password_hash.as_deref() == Some(password_hash.as_str()) {
            sessions::prune_expired_sessions(db).await;
            let _ = sessions::create_session(db, cx, &u.id).await;
        }
    }

    Ok(topcoat::router::error::see_other("/"))
}

#[route(GET "/auth/logout")]
pub async fn logout(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    sessions::destroy_session(db, cx).await;
    Ok(topcoat::router::error::see_other("/"))
}

#[page("/auth/login")]
pub async fn login_page(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = sessions::issue_csrf_seed(cx);
    if get_current_user_id(db, cx).await.is_some() {
        return Err(topcoat::router::error::redirect("/").into());
    }

    let redirect_param = requested_redirect(cx).unwrap_or_default();
    let login_action = if redirect_param.is_empty() {
        "/auth/login".to_string()
    } else {
        format!("/auth/login?redirect={}", encode_redirect(&redirect_param))
    };
    let login_error = uri(cx)
        .query()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "error")
                .map(|(_, v)| v.into_owned())
        })
        .unwrap_or_default();

    let github_enabled = get_setting(db, "github_oauth_enabled").await != "false";
    let google_enabled = get_setting(db, "google_oauth_enabled").await == "true";
    let has_oauth = github_enabled || google_enabled;

    Ok(view! {
        <div style="min-height:calc(100vh - 64px); display:flex; align-items:center; justify-content:center; padding:40px 24px; background:#f8f9fa;">
            <div style="width:100%; max-width:400px;">
                <div style="text-align:center; margin-bottom:32px;">
                    <a href="/" style="display:inline-flex; align-items:center; gap:8px; font-weight:800; font-size:20px; color:#16181d; letter-spacing:-0.02em; margin-bottom:8px;">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#2563EB" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                        </svg>
                        "Rust Korea"
                    </a>
                    <p style="font-size:14px; color:#6b7280;">"계정에 로그인하세요"</p>
                </div>

                <div style="background:#fff; border:1px solid #e5e7eb; border-radius:8px; padding:32px; box-shadow:0 1px 3px rgba(0,0,0,0.05);">
                    if has_oauth {
                        <div style="display:flex; flex-direction:column; gap:10px; margin-bottom:24px;">
                            if github_enabled {
                                <a href="/auth/github" style="display:flex; align-items:center; justify-content:center; gap:10px; width:100%; background:#24292f; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none; transition:background 0.15s;" onmouseover="this.style.background='#1b1f23'" onmouseout="this.style.background='#24292f'">
                                    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                                    "GitHub로 로그인"
                                </a>
                            }
                            if google_enabled {
                                <a href="/auth/google" style="display:flex; align-items:center; justify-content:center; gap:10px; width:100%; background:#fff; color:#374151; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none; border:1px solid #d1d5db; transition:border-color 0.15s;" onmouseover="this.style.borderColor='#9ca3af'" onmouseout="this.style.borderColor='#d1d5db'">
                                    <svg width="18" height="18" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>
                                    "Google로 로그인"
                                </a>
                            }
                        </div>

                        <div style="display:flex; align-items:center; gap:12px; margin-bottom:24px;">
                            <div style="flex:1; height:1px; background:#e5e7eb;"></div>
                            <span style="font-size:12px; color:#9ca3af;">"또는"</span>
                            <div style="flex:1; height:1px; background:#e5e7eb;"></div>
                        </div>
                    }

                    <form action=(login_action) method="POST" style="display:flex; flex-direction:column; gap:20px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        if login_error == "5" {
                            <div style="background:rgba(251,191,36,0.1); border:1px solid rgba(251,191,36,0.3); border-radius:6px; padding:10px 12px; font-size:13px; color:#b45309;">"가입 승인 대기 중입니다. 관리자 승인 후 로그인할 수 있습니다."</div>
                        }
                        if login_error == "6" {
                            <div style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.3); border-radius:6px; padding:10px 12px; font-size:13px; color:#b91c1c;">"정지된 계정입니다. 관리자에게 문의하세요."</div>
                        }
                        <div>
                            <label style="display:block; font-size:14px; font-weight:500; color:#374151; margin-bottom:6px;">"이메일"</label>
                            <input type="email" name="email" required="true" placeholder="email@example.com" style="width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none; transition:border-color 0.15s; box-sizing:border-box;" onfocus="this.style.borderColor='#2563EB'; this.style.boxShadow='0 0 0 3px rgba(37,99,235,0.1)'" onblur="this.style.borderColor='#d1d5db'; this.style.boxShadow='none'">
                        </div>
                        <div>
                            <label style="display:block; font-size:14px; font-weight:500; color:#374151; margin-bottom:6px;">"비밀번호"</label>
                            <input type="password" name="password" required="true" placeholder="8자 이상" style="width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none; transition:border-color 0.15s; box-sizing:border-box;" onfocus="this.style.borderColor='#2563EB'; this.style.boxShadow='0 0 0 3px rgba(37,99,235,0.1)'" onblur="this.style.borderColor='#d1d5db'; this.style.boxShadow='none'">
                        </div>
                        <button type="submit" style="width:100%; background:#2563EB; color:#fff; padding:10px 20px; border-radius:8px; font-size:16px; font-weight:600; border:none; cursor:pointer; transition:background 0.15s; box-shadow:0 1px 3px rgba(0,0,0,0.1);" onmouseover="this.style.background='#1D4ED8'" onmouseout="this.style.background='#2563EB'">"로그인"</button>
                    </form>

                    <div style="margin-top:24px; text-align:center; font-size:14px; color:#6b7280;">
                        <span>"계정이 없으신가요? "</span>
                        <a href="/auth/register" style="color:#2563EB; font-weight:500;">"회원가입"</a>
                    </div>
                </div>
            </div>
        </div>
    })
}

#[page("/auth/register")]
pub async fn register_page(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = sessions::issue_csrf_seed(cx);
    if get_current_user_id(db, cx).await.is_some() {
        return Err(topcoat::router::error::redirect("/").into());
    }

    let github_enabled = get_setting(db, "github_oauth_enabled").await != "false";
    let google_enabled = get_setting(db, "google_oauth_enabled").await == "true";
    let has_oauth = github_enabled || google_enabled;

    Ok(view! {
        <div style="min-height:calc(100vh - 64px); display:flex; align-items:center; justify-content:center; padding:40px 24px; background:#f8f9fa;">
            <div style="width:100%; max-width:400px;">
                <div style="text-align:center; margin-bottom:32px;">
                    <a href="/" style="display:inline-flex; align-items:center; gap:8px; font-weight:800; font-size:20px; color:#16181d; letter-spacing:-0.02em; margin-bottom:8px;">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#2563EB" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                        </svg>
                        "Rust Korea"
                    </a>
                    <p style="font-size:14px; color:#6b7280;">"새 계정을 만드세요"</p>
                </div>

                <div style="background:#fff; border:1px solid #e5e7eb; border-radius:8px; padding:32px; box-shadow:0 1px 3px rgba(0,0,0,0.05);">
                    if has_oauth {
                        <div style="display:flex; flex-direction:column; gap:10px; margin-bottom:24px;">
                            if github_enabled {
                                <a href="/auth/github" style="display:flex; align-items:center; justify-content:center; gap:10px; width:100%; background:#24292f; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none; transition:background 0.15s;" onmouseover="this.style.background='#1b1f23'" onmouseout="this.style.background='#24292f'">
                                    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                                    "GitHub로 회원가입"
                                </a>
                            }
                            if google_enabled {
                                <a href="/auth/google" style="display:flex; align-items:center; justify-content:center; gap:10px; width:100%; background:#fff; color:#374151; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none; border:1px solid #d1d5db; transition:border-color 0.15s;" onmouseover="this.style.borderColor='#9ca3af'" onmouseout="this.style.borderColor='#d1d5db'">
                                    <svg width="18" height="18" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>
                                    "Google로 회원가입"
                                </a>
                            }
                        </div>

                        <div style="display:flex; align-items:center; gap:12px; margin-bottom:24px;">
                            <div style="flex:1; height:1px; background:#e5e7eb;"></div>
                            <span style="font-size:12px; color:#9ca3af;">"또는"</span>
                            <div style="flex:1; height:1px; background:#e5e7eb;"></div>
                        </div>
                    }

                    <form action="/auth/register" method="POST" style="display:flex; flex-direction:column; gap:20px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div>
                            <label style="display:block; font-size:14px; font-weight:500; color:#374151; margin-bottom:6px;">"이메일"</label>
                            <input type="email" name="email" required="true" placeholder="email@example.com" style="width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none; transition:border-color 0.15s; box-sizing:border-box;" onfocus="this.style.borderColor='#2563EB'; this.style.boxShadow='0 0 0 3px rgba(37,99,235,0.1)'" onblur="this.style.borderColor='#d1d5db'; this.style.boxShadow='none'">
                        </div>
                        <div>
                            <label style="display:block; font-size:14px; font-weight:500; color:#374151; margin-bottom:6px;">"사용자 이름"</label>
                            <input type="text" name="username" required="true" placeholder="홍길동" style="width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none; transition:border-color 0.15s; box-sizing:border-box;" onfocus="this.style.borderColor='#2563EB'; this.style.boxShadow='0 0 0 3px rgba(37,99,235,0.1)'" onblur="this.style.borderColor='#d1d5db'; this.style.boxShadow='none'">
                        </div>
                        <div>
                            <label style="display:block; font-size:14px; font-weight:500; color:#374151; margin-bottom:6px;">"비밀번호"</label>
                            <input type="password" name="password" required="true" placeholder="8자 이상" style="width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none; transition:border-color 0.15s; box-sizing:border-box;" onfocus="this.style.borderColor='#2563EB'; this.style.boxShadow='0 0 0 3px rgba(37,99,235,0.1)'" onblur="this.style.borderColor='#d1d5db'; this.style.boxShadow='none'">
                        </div>
                        <div>
                            <label style="display:block; font-size:14px; font-weight:500; color:#374151; margin-bottom:6px;">"비밀번호 확인"</label>
                            <input type="password" name="password_confirm" required="true" placeholder="비밀번호를 다시 입력하세요" style="width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none; transition:border-color 0.15s; box-sizing:border-box;" onfocus="this.style.borderColor='#2563EB'; this.style.boxShadow='0 0 0 3px rgba(37,99,235,0.1)'" onblur="this.style.borderColor='#d1d5db'; this.style.boxShadow='none'">
                        </div>
                        <button type="submit" style="width:100%; background:#2563EB; color:#fff; padding:10px 20px; border-radius:8px; font-size:16px; font-weight:600; border:none; cursor:pointer; transition:background 0.15s; box-shadow:0 1px 3px rgba(0,0,0,0.1);" onmouseover="this.style.background='#1D4ED8'" onmouseout="this.style.background='#2563EB'">"회원가입"</button>
                    </form>

                    <div style="margin-top:24px; text-align:center; font-size:14px; color:#6b7280;">
                        <span>"이미 계정이 있으신가요? "</span>
                        <a href="/auth/login" style="color:#2563EB; font-weight:500;">"로그인"</a>
                    </div>
                </div>
            </div>
        </div>
    })
}
