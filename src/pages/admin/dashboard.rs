use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::auth::session as sessions;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub csrf_token: String,
    pub email: String,
    pub password: String,
}

#[route(POST "/admin/login")]
pub async fn admin_login(
    cx: &Cx,
    Form(input): Form<LoginForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !sessions::same_origin_ok(cx) {
        return Ok(topcoat::router::error::see_other("/admin/login?error=1"));
    }
    if !sessions::verify_csrf_seed_logged(db, cx, &input.csrf_token, "admin.login").await {
        return Ok(topcoat::router::error::see_other("/admin/login?error=1"));
    }
    if !sessions::rate_limit_ok(db, cx, &input.email).await {
        return Ok(topcoat::router::error::see_other("/admin/login?error=1"));
    }
    sessions::record_login_attempt(db, cx, &input.email).await;

    // Admin console = password hash + active status + any admin permission.
    let user = match sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE email = $1 AND password_hash IS NOT NULL",
    )
    .bind(&input.email)
    .fetch_optional(db)
    .await
    {
        Ok(u) => u,
        Err(e) => {
            eprintln!("[admin_login] db error: {e}");
            None
        }
    };

    if let Some(u) = user {
        use crate::auth::access::LoginDecision;
        use argon2::Argon2;
        use argon2::password_hash::{PasswordHash, PasswordVerifier};
        if crate::auth::access::login_decision(&u) != LoginDecision::Allow {
            return Ok(topcoat::router::error::see_other("/admin/login?error=1"));
        }
        if let Ok(hash) = PasswordHash::new(u.password_hash.as_deref().unwrap_or("")) {
            let ok = Argon2::default()
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok();
            if !ok {
                return Ok(topcoat::router::error::see_other("/admin/login?error=1"));
            }
            let access = crate::auth::access::load_access_for_user(db, &u).await;
            let allowed = access
                .as_ref()
                .map(crate::auth::access::can_access_admin)
                .unwrap_or(false);
            if !allowed {
                crate::auth::access::audit(
                    db,
                    cx,
                    &u.id,
                    "admin.login.denied",
                    "user",
                    &u.id,
                    "no admin permission",
                )
                .await;
                return Ok(topcoat::router::error::see_other("/admin/login?error=1"));
            }
            sessions::prune_expired_sessions(db).await;
            if u.totp_enabled == 1 && !u.totp_secret.is_empty() {
                sessions::clear_login_attempts(db, cx, &input.email).await;
                if sessions::create_pending_session(db, cx, &u.id)
                    .await
                    .is_ok()
                {
                    crate::auth::access::audit(
                        db,
                        cx,
                        &u.id,
                        "admin.login.2fa_required",
                        "user",
                        &u.id,
                        "",
                    )
                    .await;
                    return Ok(topcoat::router::error::see_other("/admin/2fa"));
                }
            } else if sessions::create_session(db, cx, &u.id).await.is_ok() {
                sessions::clear_login_attempts(db, cx, &input.email).await;
                crate::auth::access::audit(db, cx, &u.id, "admin.login", "user", &u.id, "").await;
                return Ok(topcoat::router::error::see_other("/admin"));
            }
        }
    }
    crate::auth::access::audit(db, cx, "", "admin.login.failed", "user", &input.email, "").await;
    Ok(topcoat::router::error::see_other("/admin/login?error=1"))
}

#[route(GET "/admin/logout")]
pub async fn admin_logout(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    sessions::destroy_session(db, cx).await;
    Ok(topcoat::router::error::see_other("/admin/login"))
}

#[page("/admin/login")]
pub async fn admin_login_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_admin_area(__adb, cx)
            .await
            .is_some()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin").into());
    }

    let csrf_token = sessions::issue_csrf_seed(cx);

    let (site_title_val, logo_url_val) = {
        let db: &crate::db::Db = topcoat::context::app_context(cx);
        let t = crate::services::site_settings::get_setting(db, "site_title").await;
        let l = crate::services::site_settings::get_setting(db, "logo_url").await;
        (
            if t.is_empty() {
                "Rust Korea".to_string()
            } else {
                t
            },
            l,
        )
    };

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"관리자 로그인 - Rust Korea"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body>
            <div class="wrap">
                <div class="logo">
                    <div class="logo-icon">
                        if !logo_url_val.is_empty() {
                            <img src=(logo_url_val.clone()) alt="logo" style="width:32px; height:32px; border-radius:6px; object-fit:contain;">
                        } else {
                            <img src="/logo.svg" alt="Rust logo" width="24" height="24" style="object-fit:contain;">
                        }
                    </div>
                    <h1>(site_title_val)</h1>
                    <p>"관리자 패널"</p>
                </div>

                <div class="card">
                    <h2>"로그인"</h2>
                    <p class="sub">"이메일과 비밀번호를 입력하세요"</p>

                    <div class="error" id="error-msg">"이메일 또는 비밀번호가 올바르지 않습니다."</div>

                    <form action="/admin/login" method="POST">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div class="field">
                            <label for="email">"이메일"</label>
                            <input type="email" id="email" name="email" required="true" placeholder="admin@rustkorea.kr" autofocus="true">
                        </div>
                        <div class="field">
                            <label for="password">"비밀번호"</label>
                            <input type="password" id="password" name="password" required="true" placeholder="비밀번호 입력">
                        </div>
                        <button type="submit" class="btn btn-block">"로그인"</button>
                    </form>
                </div>

                <div class="site-link">
                    <a href="/">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                        "사이트로 돌아가기"
                    </a>
                </div>
            </div>
            <script>
                "if(window.location.search.includes('error=1'))document.getElementById('error-msg').style.display='block'"
            </script>
        </body>
        </html>
    })
}

#[page("/admin")]
pub async fn admin_dashboard(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_admin_area(__adb, cx)
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);

    let series_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM manual_series")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let chapter_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM manual_chapters")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let video_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM youtube_videos")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let comment_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let total_views: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(view_count), 0) FROM posts")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let bookmark_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let report_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM reports WHERE status = 'pending'")
            .fetch_one(db)
            .await
            .unwrap_or(0);
    let _notif_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications")
        .fetch_one(db)
        .await
        .unwrap_or(0);

    let recent_posts = sqlx::query_as::<_, crate::models::PostListItem>(
        "SELECT p.*, COALESCE(u.username, p.author_id) AS username FROM posts p
         LEFT JOIN users u ON u.id = p.author_id
         ORDER BY p.created_at DESC LIMIT 5",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let popular_posts = sqlx::query_as::<_, crate::models::Post>(
        "SELECT * FROM posts ORDER BY view_count DESC LIMIT 5",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let _recent_series = sqlx::query_as::<_, crate::models::ManualSeries>(
        "SELECT * FROM manual_series ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let recent_users = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"대시보드 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"대시보드"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"Rust Korea Hub 관리 현황"</p>
                </div>

                <div style="display:grid; grid-template-columns:repeat(4,1fr); gap:16px; margin-bottom:24px;">
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(96,165,250,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--blue)" stroke-width="2" stroke-linecap="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(post_count)</div><div style="font-size:12px; color:var(--muted2);">"게시글"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(251,191,36,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--amber)" stroke-width="2" stroke-linecap="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(user_count)</div><div style="font-size:12px; color:var(--muted2);">"사용자"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(167,139,250,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--purple)" stroke-width="2" stroke-linecap="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(total_views)</div><div style="font-size:12px; color:var(--muted2);">"총 조회수"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(222,91,48,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#DE5B30" stroke-width="2" stroke-linecap="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(series_count)</div><div style="font-size:12px; color:var(--muted2);">"매뉴얼 시리즈"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(96,165,250,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--blue)" stroke-width="2" stroke-linecap="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(chapter_count)</div><div style="font-size:12px; color:var(--muted2);">"매뉴얼 챕터"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(167,139,250,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--purple)" stroke-width="2" stroke-linecap="round"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(comment_count)</div><div style="font-size:12px; color:var(--muted2);">"댓글"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(248,113,113,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--red)" stroke-width="2" stroke-linecap="round"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(video_count)</div><div style="font-size:12px; color:var(--muted2);">"영상"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(52,211,153,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--green)" stroke-width="2" stroke-linecap="round"><path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(bookmark_count)</div><div style="font-size:12px; color:var(--muted2);">"북마크"</div></div>
                    </div>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px; display:flex; align-items:center; gap:16px;">
                        <div style="width:44px; height:44px; border-radius:10px; background:rgba(251,146,60,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#fb923c" stroke-width="2" stroke-linecap="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                        </div>
                        <div><div style="font-size:1.5rem; font-weight:800; color:var(--text); letter-spacing:-0.02em;">(report_count)</div><div style="font-size:12px; color:var(--muted2);">"미처리 신고"</div></div>
                    </div>
                </div>

                <div style="display:grid; grid-template-columns:1fr 1fr 1fr; gap:16px; margin-bottom:16px;">
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                        <div style="padding:16px 20px; border-bottom:1px solid var(--border); display:flex; align-items:center; justify-content:space-between;">
                            <h2 style="font-size:14px; font-weight:700; color:var(--text2);">"최근 가입 사용자"</h2>
                            <a href="/admin/users" style="font-size:12px; color:#DE5B30; font-weight:600;">"전체 보기"</a>
                        </div>
                        <div>
                            if recent_users.is_empty() {
                                <p style="padding:40px; text-align:center; font-size:13px; color:var(--dim);">"등록된 사용자가 없습니다"</p>
                            } else {
                                for u in &recent_users {
                                    <div style="display:flex; align-items:center; justify-content:space-between; padding:12px 20px; border-bottom:1px solid var(--line);">
                                        <div style="display:flex; align-items:center; gap:10px; min-width:0;">
                                            <div style="width:32px; height:32px; border-radius:8px; background:rgba(222,91,48,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0; font-size:13px; font-weight:700; color:#DE5B30;">(u.username.chars().next().unwrap_or('?').to_string())</div>
                                            <div style="min-width:0;">
                                                <div style="font-size:13px; font-weight:600; color:var(--text3); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">(u.username.clone())</div>
                                                <div style="font-size:11px; color:var(--dim); margin-top:2px;">(u.email.clone().unwrap_or_default())</div>
                                            </div>
                                        </div>
                                        <span style="font-size:11px; color:var(--faint); flex-shrink:0;">(u.created_at.get(..10).unwrap_or("").to_string())</span>
                                    </div>
                                }
                            }
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                        <div style="padding:16px 20px; border-bottom:1px solid var(--border); display:flex; align-items:center; justify-content:space-between;">
                            <h2 style="font-size:14px; font-weight:700; color:var(--text2);">"최근 게시글"</h2>
                            <a href="/admin/posts" style="font-size:12px; color:#DE5B30; font-weight:600;">"전체 보기"</a>
                        </div>
                        <div>
                            if recent_posts.is_empty() {
                                <p style="padding:40px; text-align:center; font-size:13px; color:var(--dim);">"등록된 게시글이 없습니다"</p>
                            } else {
                                for p in &recent_posts {
                                    <a href=(format!("/community/{}", p.id)) style="display:flex; align-items:center; justify-content:space-between; padding:12px 20px; border-bottom:1px solid var(--line); transition:background 0.15s;">
                                        <div style="min-width:0;">
                                            <div style="font-size:13px; font-weight:600; color:var(--text3);">(p.title.clone())</div>
                                            <div style="font-size:11px; color:var(--dim); margin-top:2px;">(p.username.clone())</div>
                                        </div>
                                        <span style="font-size:11px; color:var(--faint); flex-shrink:0;">(p.created_at.get(..10).unwrap_or("").to_string())</span>
                                    </a>
                                }
                            }
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                        <div style="padding:16px 20px; border-bottom:1px solid var(--border); display:flex; align-items:center; justify-content:space-between;">
                            <h2 style="font-size:14px; font-weight:700; color:var(--text2);">"인기 게시글"</h2>
                            <a href="/admin/posts" style="font-size:12px; color:#DE5B30; font-weight:600;">"전체 보기"</a>
                        </div>
                        <div>
                            if popular_posts.is_empty() {
                                <p style="padding:40px; text-align:center; font-size:13px; color:var(--dim);">"등록된 게시글이 없습니다"</p>
                            } else {
                                for p in &popular_posts {
                                    <a href=(format!("/community/{}", p.id)) style="display:flex; align-items:center; justify-content:space-between; padding:12px 20px; border-bottom:1px solid var(--line); transition:background 0.15s;">
                                        <div style="min-width:0;">
                                            <div style="font-size:13px; font-weight:600; color:var(--text3);">(p.title.clone())</div>
                                            <div style="font-size:11px; color:var(--dim); margin-top:2px;">(format!("{} \u{c9c8} \u{bcf4}\u{ae30}", p.view_count))</div>
                                        </div>
                                        <span style="font-size:11px; color:var(--faint); flex-shrink:0;">(format!("{}\u{c9c8}", p.view_count))</span>
                                    </a>
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        </body>
        </html>
    })
}
