use std::future::Future;
use topcoat::router::request::uri;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::User;

const INPUT_STYLE: &str = "width:100%; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;";
const LABEL_STYLE: &str =
    "display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;";

#[derive(serde::Deserialize)]
pub struct DeleteForm {
    pub csrf_token: String,
    pub user_id: String,
}

#[derive(serde::Deserialize)]
pub struct CreateUserForm {
    pub csrf_token: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub role_id: String,
}

#[derive(serde::Deserialize)]
pub struct EditUserForm {
    pub csrf_token: String,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub role_id: String,
    pub status: String,
}

#[derive(serde::Deserialize)]
pub struct ResetPasswordForm {
    pub csrf_token: String,
    pub user_id: String,
    pub new_password: String,
}

fn extract_param(cx: &Cx, key: &str) -> String {
    let query = uri(cx).query().unwrap_or("");
    form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default()
}

#[page("/admin/users")]
pub async fn admin_users_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "users.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    let roles =
        sqlx::query_as::<_, crate::models::Role>("SELECT * FROM roles ORDER BY position ASC")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    let pending_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE status = 'pending'")
            .fetch_one(db)
            .await
            .unwrap_or(0);

    let query_str = uri(cx).query().unwrap_or("");

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"사용자 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/users".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700; color:#f9fafb; letter-spacing:-0.01em;">"사용자 관리"</h1>
                        <p style="font-size:13px; color:#6b7280; margin-top:2px;">"전체 사용자 목록 및 관리"</p>
                    </div>
                    <a href="/admin/users/new" style="background:#DE5B30; color:#fff; padding:8px 16px; border-radius:8px; font-size:13px; font-weight:600; text-decoration:none; transition:background 0.15s;" onmouseover="this.style.background='#c94e28'" onmouseout="this.style.background='#DE5B30'">"+ 사용자 추가"</a>
                </div>

                if query_str.contains("created=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#34d399;">"사용자가 생성되었습니다."</div>
                }
                if query_str.contains("updated=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#34d399;">"사용자 정보가 수정되었습니다."</div>
                }
                if query_str.contains("password_reset=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#34d399;">"비밀번호가 초기화되었습니다."</div>
                }
                if pending_count > 0 {
                    <div style="background:rgba(251,191,36,0.1); border:1px solid rgba(251,191,36,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#fbbf24;">(format!("승인 대기 중인 사용자가 {pending_count}명 있습니다."))</div>
                }

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden;">
                    <div style="padding:16px 20px; border-bottom:1px solid #2a2d3a;">
                        <span style="font-size:14px; font-weight:700; color:#e5e7eb;">"전체 "(users.len())"명"</span>
                    </div>

                    if users.is_empty() {
                        <div style="padding:60px; text-align:center;">
                            <p style="font-size:14px; color:#4b5563;">"등록된 사용자가 없습니다"</p>
                        </div>
                    } else {
                        <table style="width:100%; border-collapse:collapse; font-size:13px;">
                            <thead>
                                <tr>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"사용자"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"이메일"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"역할"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"상태"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"인증"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"가입일"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"관리"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for u in &users {
                                    <tr>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                            <div style="display:flex; align-items:center; gap:10px;">
                                                <div style="width:32px; height:32px; border-radius:8px; background:rgba(222,91,48,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0; font-size:13px; font-weight:700; color:#DE5B30;">(u.username.chars().next().unwrap_or('?').to_string())</div>
                                                <div>
                                                    <div style="font-weight:600; color:#d1d5db;">(u.username.clone())</div>
                                                    <div style="font-size:11px; color:#4b5563; font-family:monospace;">(format!("ID {}", u.id))</div>
                                                </div>
                                            </div>
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#9ca3af;">
                                            (u.email.clone().unwrap_or_else(|| "-".into()))
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                            if roles.iter().any(|r| r.id == u.role_id && r.name == "admin") {
                                                <span style="background:rgba(239,68,68,0.1); color:#f87171; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"관리자"</span>
                                            } else if roles.iter().any(|r| r.id == u.role_id && r.name == "editor") {
                                                <span style="background:rgba(96,165,250,0.1); color:#60a5fa; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"에디터"</span>
                                            } else if roles.iter().any(|r| r.id == u.role_id && r.name == "member") {
                                                <span style="background:rgba(107,114,128,0.1); color:#9ca3af; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"일반"</span>
                                            } else {
                                                <span style="background:rgba(167,139,250,0.1); color:#a78bfa; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"커스텀"</span>
                                            }
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                            if u.status == "active" {
                                                <span style="background:rgba(52,211,153,0.1); color:#34d399; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"활성"</span>
                                            } else if u.status == "pending" {
                                                <span style="background:rgba(251,191,36,0.1); color:#fbbf24; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"승인대기"</span>
                                            } else {
                                                <span style="background:rgba(239,68,68,0.1); color:#f87171; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"정지"</span>
                                            }
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                            if u.password_hash.is_some() {
                                                <span style="background:rgba(52,211,153,0.1); color:#34d399; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"이메일"</span>
                                            } else if u.github_id.is_some() {
                                                <span style="background:rgba(167,139,250,0.1); color:#a78bfa; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"GitHub"</span>
                                            } else if u.google_id.is_some() {
                                                <span style="background:rgba(96,165,250,0.1); color:#60a5fa; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"Google"</span>
                                            } else {
                                                <span style="background:rgba(251,191,36,0.1); color:#fbbf24; padding:3px 8px; border-radius:4px; font-size:11px; font-weight:600;">"게스트"</span>
                                            }
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#6b7280; font-size:12px;">
                                            (u.created_at.get(..10).unwrap_or("").to_string())
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                            <div style="display:flex; gap:6px;">
                                                <a href=(format!("/admin/users/edit?user_id={}", u.id)) style="background:rgba(96,165,250,0.1); color:#60a5fa; padding:4px 10px; border-radius:6px; font-size:11px; text-decoration:none; font-weight:600;">"편집"</a>
                                                <a href=(format!("/admin/users/reset-password?user_id={}", u.id)) style="background:rgba(251,191,36,0.1); color:#fbbf24; padding:4px 10px; border-radius:6px; font-size:11px; text-decoration:none; font-weight:600;">"비밀번호"</a>
                                                if u.status == "pending" {
                                                    <form action="/admin/users/status" method="POST" style="display:inline;">
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                        <input type="hidden" name="user_id" value=(u.id.clone())>
                                                        <input type="hidden" name="status" value="active">
                                                        <button type="submit" style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.3); color:#34d399; padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;">"승인"</button>
                                                    </form>
                                                }
                                                if u.status == "active" && u.username != "guest" {
                                                    <form action="/admin/users/status" method="POST" style="display:inline;" onsubmit="return confirm('이 사용자를 정지하시겠습니까? 즉시 로그아웃됩니다.')">
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                        <input type="hidden" name="user_id" value=(u.id.clone())>
                                                        <input type="hidden" name="status" value="suspended">
                                                        <button type="submit" style="background:none; border:1px solid rgba(251,191,36,0.3); color:#fbbf24; padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;">"정지"</button>
                                                    </form>
                                                }
                                                if u.status == "suspended" {
                                                    <form action="/admin/users/status" method="POST" style="display:inline;">
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                        <input type="hidden" name="user_id" value=(u.id.clone())>
                                                        <input type="hidden" name="status" value="active">
                                                        <button type="submit" style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.3); color:#34d399; padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;">"해제"</button>
                                                    </form>
                                                }
                                                if u.username != "guest" && u.username != "admin" {
                                                    <form action="/admin/users/delete" method="POST" style="display:inline;" onsubmit="return confirm('정말 이 사용자를 삭제하시겠습니까?')">
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                        <input type="hidden" name="user_id" value=(u.id.clone())>
                                                        <button type="submit" style="background:none; border:1px solid rgba(239,68,68,0.3); color:#f87171; padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;">"삭제"</button>
                                                    </form>
                                                }
                                            </div>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    }
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/users/delete")]
pub async fn admin_delete_user(
    cx: &Cx,
    Form(input): Form<DeleteForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.users.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::load_access(db, cx).await {
        Some(a) if a.can("users.manage") => a,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(&input.user_id)
        .fetch_optional(db)
        .await
        .unwrap_or(None);

    if let Some(u) = user {
        if u.username != "guest" && u.username != "admin" && u.id != me.user.id {
            let super_left: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM users u JOIN roles r ON r.id = u.role_id
                 WHERE r.is_superuser = 1 AND u.id != $1",
            )
            .bind(&u.id)
            .fetch_one(db)
            .await
            .unwrap_or(0);
            let target_super: bool = sqlx::query_scalar(
                "SELECT COALESCE((SELECT is_superuser = 1 FROM roles WHERE id = $1), false)",
            )
            .bind(&u.role_id)
            .fetch_one(db)
            .await
            .unwrap_or(false);
            if !target_super || super_left > 0 {
                let mut tx = match db.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        eprintln!("[admin] delete user tx begin failed: {e}");
                        return Ok(topcoat::router::error::see_other("/admin/users?error=1"));
                    }
                };
                let ok = sqlx::query("DELETE FROM comments WHERE author_id = $1")
                    .bind(&u.id)
                    .execute(&mut *tx)
                    .await
                    .is_ok()
                    && sqlx::query("DELETE FROM posts WHERE author_id = $1")
                        .bind(&u.id)
                        .execute(&mut *tx)
                        .await
                        .is_ok()
                    && sqlx::query("DELETE FROM users WHERE id = $1")
                        .bind(&u.id)
                        .execute(&mut *tx)
                        .await
                        .map(|r| r.rows_affected() == 1)
                        .unwrap_or(false);
                if ok && tx.commit().await.is_ok() {
                    crate::auth::access::audit(
                        db,
                        cx,
                        &me.user.id,
                        "admin.user.delete",
                        "user",
                        &input.user_id,
                        "",
                    )
                    .await;
                } else {
                    eprintln!("[admin] delete user tx failed: {}", u.id);
                    return Ok(topcoat::router::error::see_other("/admin/users?error=1"));
                }
            }
        }
    }

    Ok(topcoat::router::error::see_other("/admin/users"))
}

#[page("/admin/users/new")]
pub async fn admin_user_new_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "users.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let all_roles =
        sqlx::query_as::<_, crate::models::Role>("SELECT * FROM roles ORDER BY position ASC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"사용자 추가 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/users".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:#f9fafb; letter-spacing:-0.01em;">"사용자 추가"</h1>
                    <p style="font-size:13px; color:#6b7280; margin-top:2px;">"새로운 사용자를 생성합니다."</p>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; max-width:560px;">
                    <form action="/admin/users/create" method="POST">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="margin-bottom:16px;">
                            <label style=(LABEL_STYLE)>"이메일"</label>
                            <input type="email" name="email" required="true" placeholder="user@example.com" style=(INPUT_STYLE)>
                        </div>
                        <div style="margin-bottom:16px;">
                            <label style=(LABEL_STYLE)>"사용자명"</label>
                            <input type="text" name="username" required="true" placeholder="username" style=(INPUT_STYLE)>
                        </div>
                        <div style="margin-bottom:16px;">
                            <label style=(LABEL_STYLE)>"비밀번호"</label>
                            <input type="password" name="password" required="true" placeholder="비밀번호를 입력하세요" style=(INPUT_STYLE)>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(LABEL_STYLE)>"역할"</label>
                            <select name="role_id" required="true" style=(INPUT_STYLE)>
                                for r in &all_roles {
                                    <option value=(r.id.clone())>(crate::auth::access::role_label(&r.name))</option>
                                }
                            </select>
                        </div>
                        <div style="display:flex; gap:10px;">
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"사용자 추가"</button>
                            <a href="/admin/users" style="background:none; border:1px solid #2a2d3a; color:#9ca3af; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none;">"취소"</a>
                        </div>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/users/create")]
pub async fn admin_create_user(
    cx: &Cx,
    Form(input): Form<CreateUserForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "users.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.users.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    if !crate::auth::session::password_ok(&input.password) {
        return Ok(topcoat::router::error::see_other(
            "/admin/users/new?error=password",
        ));
    }
    let role_ok: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM roles WHERE id = $1")
        .bind(&input.role_id)
        .fetch_one(db)
        .await
        .unwrap_or(false);
    if !role_ok {
        return Ok(topcoat::router::error::see_other(
            "/admin/users/new?error=role",
        ));
    }

    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();

    let done = sqlx::query(
        "INSERT INTO users (email, username, password_hash, role_id, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 'active', NOW()::text, NOW()::text)"
    )
    .bind(&input.email)
    .bind(&input.username)
    .bind(&hash)
    .bind(&input.role_id)
    .execute(db)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);

    if done != 1 {
        return Ok(topcoat::router::error::see_other(
            "/admin/users/new?error=exists",
        ));
    }
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.user.create",
        "user",
        &input.email,
        &input.role_id,
    )
    .await;

    Ok(topcoat::router::error::see_other("/admin/users?created=1"))
}

#[page("/admin/users/edit")]
pub async fn admin_user_edit_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "users.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let user_id = extract_param(cx, "user_id");
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_optional(db)
        .await
        .unwrap_or(None);

    let Some(u) = user else {
        return Err(topcoat::router::error::redirect("/admin/users").into());
    };

    let id_display = u.id.clone();
    let email_display = u.email.clone().unwrap_or_default();
    let username_display = u.username.clone();
    let current_role_id = u.role_id.clone();
    let current_status = u.status.clone();
    let all_roles =
        sqlx::query_as::<_, crate::models::Role>("SELECT * FROM roles ORDER BY position ASC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"사용자 편집 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/users".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:#f9fafb; letter-spacing:-0.01em;">"사용자 편집"</h1>
                    <p style="font-size:13px; color:#6b7280; margin-top:2px;">(format!("{} 사용자의 정보를 수정합니다.", username_display))</p>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; max-width:560px;">
                    <form action="/admin/users/update" method="POST">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="hidden" name="user_id" value=(id_display.clone())>
                        <div style="margin-bottom:16px;">
                            <label style=(LABEL_STYLE)>"이메일"</label>
                            <input type="email" name="email" required="true" value=(email_display) style=(INPUT_STYLE)>
                        </div>
                        <div style="margin-bottom:16px;">
                            <label style=(LABEL_STYLE)>"사용자명"</label>
                            <input type="text" name="username" required="true" value=(username_display) style=(INPUT_STYLE)>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(LABEL_STYLE)>"역할"</label>
                            <select name="role_id" required="true" style=(INPUT_STYLE)>
                                for r in &all_roles {
                                    if r.id == current_role_id {
                                        <option value=(r.id.clone()) selected="true">(crate::auth::access::role_label(&r.name))</option>
                                    } else {
                                        <option value=(r.id.clone())>(crate::auth::access::role_label(&r.name))</option>
                                    }
                                }
                            </select>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(LABEL_STYLE)>"상태"</label>
                            <select name="status" required="true" style=(INPUT_STYLE)>
                                if current_status == "active" {
                                    <option value="active" selected="true">"활성"</option>
                                } else {
                                    <option value="active">"활성"</option>
                                }
                                if current_status == "pending" {
                                    <option value="pending" selected="true">"승인대기"</option>
                                } else {
                                    <option value="pending">"승인대기"</option>
                                }
                                if current_status == "suspended" {
                                    <option value="suspended" selected="true">"정지"</option>
                                } else {
                                    <option value="suspended">"정지"</option>
                                }
                            </select>
                        </div>
                        <div style="display:flex; gap:10px;">
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"저장"</button>
                            <a href="/admin/users" style="background:none; border:1px solid #2a2d3a; color:#9ca3af; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none;">"취소"</a>
                        </div>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/users/update")]
pub async fn admin_update_user(
    cx: &Cx,
    Form(input): Form<EditUserForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.users.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::load_access(db, cx).await {
        Some(a) if a.is_admin() => a,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    // Validate role + status against known values.
    let role_ok: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM roles WHERE id = $1")
        .bind(&input.role_id)
        .fetch_one(db)
        .await
        .unwrap_or(false);
    if !role_ok || !["active", "suspended", "pending"].contains(&input.status.as_str()) {
        return Ok(topcoat::router::error::see_other(
            "/admin/users?error=invalid",
        ));
    }

    let target: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(&input.user_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    let Some(target) = target else {
        return Ok(topcoat::router::error::see_other(
            "/admin/users?error=invalid",
        ));
    };

    // Never demote yourself or the last superuser out of the superuser role.
    let target_super: bool = sqlx::query_scalar(
        "SELECT COALESCE((SELECT is_superuser = 1 FROM roles WHERE id = $1), false)",
    )
    .bind(&target.role_id)
    .fetch_one(db)
    .await
    .unwrap_or(false);
    let new_super: bool = sqlx::query_scalar(
        "SELECT COALESCE((SELECT is_superuser = 1 FROM roles WHERE id = $1), false)",
    )
    .bind(&input.role_id)
    .fetch_one(db)
    .await
    .unwrap_or(false);
    if target_super && !new_super {
        let supers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users u JOIN roles r ON r.id = u.role_id WHERE r.is_superuser = 1",
        )
        .fetch_one(db)
        .await
        .unwrap_or(0);
        if supers <= 1 || target.id == me.user.id {
            return Ok(topcoat::router::error::see_other(
                "/admin/users?error=selflock",
            ));
        }
    }

    sqlx::query(
        "UPDATE users SET email = $1, username = $2, role_id = $3, status = $4, updated_at = NOW()::text WHERE id = $5"
    )
    .bind(&input.email)
    .bind(&input.username)
    .bind(&input.role_id)
    .bind(&input.status)
    .bind(&input.user_id)
    .execute(db)
    .await
    .ok();

    crate::auth::access::audit(
        db,
        cx,
        &me.user.id,
        "admin.user.update",
        "user",
        &input.user_id,
        &format!("role={} status={}", input.role_id, input.status),
    )
    .await;
    // Suspended/pending accounts lose their sessions immediately.
    crate::auth::access::audit(
        db,
        cx,
        &me.user.id,
        "admin.user.status",
        "user",
        &input.user_id,
        &input.status,
    )
    .await;
    if input.status != "active" {
        crate::auth::session::destroy_user_sessions(db, &input.user_id).await;
    }

    Ok(topcoat::router::error::see_other("/admin/users?updated=1"))
}

#[derive(serde::Deserialize)]
pub struct StatusForm {
    pub csrf_token: String,
    pub user_id: String,
    pub status: String,
}

#[route(POST "/admin/users/status")]
pub async fn admin_set_status(
    cx: &Cx,
    Form(input): Form<StatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.users.status")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::load_access(db, cx).await {
        Some(a) if a.is_admin() => a,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    if !["active", "suspended", "pending"].contains(&input.status.as_str()) {
        return Ok(topcoat::router::error::see_other(
            "/admin/users?error=invalid",
        ));
    }
    if input.user_id == me.user.id && input.status != "active" {
        return Ok(topcoat::router::error::see_other(
            "/admin/users?error=selflock",
        ));
    }

    sqlx::query("UPDATE users SET status = $1, updated_at = NOW()::text WHERE id = $2")
        .bind(&input.status)
        .bind(&input.user_id)
        .execute(db)
        .await
        .ok();
    if input.status != "active" {
        crate::auth::session::destroy_user_sessions(db, &input.user_id).await;
    }

    Ok(topcoat::router::error::see_other("/admin/users?updated=1"))
}

#[page("/admin/users/reset-password")]
pub async fn admin_user_reset_password_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "users.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let user_id = extract_param(cx, "user_id");
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_optional(db)
        .await
        .unwrap_or(None);

    let Some(u) = user else {
        return Err(topcoat::router::error::redirect("/admin/users").into());
    };

    let id_display = u.id.clone();
    let username_display = u.username.clone();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"비밀번호 초기화 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/users".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:#f9fafb; letter-spacing:-0.01em;">"비밀번호 초기화"</h1>
                    <p style="font-size:13px; color:#6b7280; margin-top:2px;">(format!("{} 사용자의 비밀번호를 변경합니다.", username_display))</p>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; max-width:560px;">
                    <form action="/admin/users/reset-password" method="POST">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="hidden" name="user_id" value=(id_display.clone())>
                        <div style="margin-bottom:20px;">
                            <label style=(LABEL_STYLE)>"새 비밀번호"</label>
                            <input type="password" name="new_password" required="true" placeholder="새 비밀번호를 입력하세요" style=(INPUT_STYLE)>
                        </div>
                        <div style="display:flex; gap:10px;">
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"비밀번호 변경"</button>
                            <a href="/admin/users" style="background:none; border:1px solid #2a2d3a; color:#9ca3af; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; text-decoration:none;">"취소"</a>
                        </div>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/users/reset-password")]
pub async fn admin_reset_password(
    cx: &Cx,
    Form(input): Form<ResetPasswordForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "users.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.users.reset-password",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    if !crate::auth::session::password_ok(&input.new_password) {
        return Ok(topcoat::router::error::see_other(
            "/admin/users?error=password",
        ));
    }

    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.new_password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW()::text WHERE id = $2")
        .bind(&hash)
        .bind(&input.user_id)
        .execute(db)
        .await
        .ok();
    crate::auth::session::destroy_user_sessions(db, &input.user_id).await;
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.user.reset_password",
        "user",
        &input.user_id,
        "",
    )
    .await;

    Ok(topcoat::router::error::see_other(
        "/admin/users?password_reset=1",
    ))
}
