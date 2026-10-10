use std::future::Future;
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

#[derive(serde::Deserialize)]
pub struct PasswordForm {
    pub csrf_token: String,
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(serde::Deserialize)]
pub struct EmailForm {
    pub csrf_token: String,
    pub admin_email: String,
}

#[page("/admin/profile")]
pub async fn admin_profile_page(cx: &Cx) -> Result<impl View + '_> {
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
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let me = crate::auth::session::current_user(db, cx).await;
    let admin_email = me
        .as_ref()
        .and_then(|u| u.email.clone())
        .unwrap_or_else(crate::config::contact_email);
    let totp_on = me.as_ref().map(|u| u.totp_enabled == 1).unwrap_or(false);
    let totp_pending_secret: String = me
        .as_ref()
        .filter(|u| u.totp_enabled == 0 && !u.totp_secret.is_empty())
        .map(|u| u.totp_secret.clone())
        .unwrap_or_default();
    let (totp_uri, totp_qr): (String, String) = me
        .as_ref()
        .filter(|u| !u.totp_secret.is_empty())
        .map(|u| {
            let account = u
                .email
                .clone()
                .filter(|e| !e.is_empty())
                .unwrap_or_else(|| u.username.clone());
            (
                crate::auth::totp::provision_uri(&u.totp_secret, &account).unwrap_or_default(),
                crate::auth::totp::qr_data_uri(&u.totp_secret, &account).unwrap_or_default(),
            )
        })
        .unwrap_or_default();

    let admin_email_display = admin_email.clone();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"관리자 프로필 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/profile".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"관리자 프로필"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"관리자 계정 설정 및 비밀번호 변경"</p>
                </div>

                <div style="display:grid; grid-template-columns:1fr 1fr; gap:20px;">
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"이메일 변경"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"관리자 로그인 이메일을 변경합니다."</p>

                        <form action="/admin/profile/email" method="POST">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <div style="margin-bottom:16px;">
                                <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"이메일 주소"</label>
                                <input type="email" name="admin_email" value=(admin_email_display) required="true" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;" placeholder="admin@rustkorea.kr">
                            </div>
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer; width:100%;">"이메일 변경"</button>
                        </form>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"비밀번호 변경"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"관리자 비밀번호를 변경합니다."</p>

                        <div id="pw-error" style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px; margin-bottom:16px; font-size:13px; color:var(--red-soft); display:none;"></div>
                        <div id="pw-success" style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px; margin-bottom:16px; font-size:13px; color:var(--green); display:none;"></div>

                        <form action="/admin/profile/password" method="POST" id="pw-form">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <div style="margin-bottom:14px;">
                                <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"현재 비밀번호"</label>
                                <input type="password" name="current_password" required="true" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                            </div>
                            <div style="margin-bottom:14px;">
                                <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"새 비밀번호"</label>
                                <input type="password" name="new_password" required="true" minlength="6" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                            </div>
                            <div style="margin-bottom:16px;">
                                <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"비밀번호 확인"</label>
                                <input type="password" name="confirm_password" required="true" minlength="6" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                            </div>
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer; width:100%;">"비밀번호 변경"</button>
                        </form>
                    </div>
                </div>

                <div style="margin-top:24px; background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px;">
                    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:4px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text);">"2단계 인증 (TOTP)"</h2>
                        if totp_on {
                            <span style="font-size:12px; font-weight:700; color:var(--green); background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.3); padding:4px 12px; border-radius:9999px;">"● 활성"</span>
                        } else {
                            <span style="font-size:12px; font-weight:700; color:var(--muted2); background:rgba(107,114,128,0.1); border:1px solid var(--border); padding:4px 12px; border-radius:9999px;">"○ 비활성"</span>
                        }
                    </div>
                    <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"인증 앱(Google Authenticator 등)에 등록하면 로그인 시 6자리 코드가 추가로 필요합니다."</p>

                    if totp_on {
                        <form action="/admin/profile/totp/disable" method="POST" style="max-width:420px;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <div style="margin-bottom:14px;">
                                <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"현재 비밀번호 (확인용)"</label>
                                <input type="password" name="current_password" required="true" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                            </div>
                            <button type="submit" style="background:none; border:1px solid rgba(239,68,68,0.3); color:var(--red); padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; cursor:pointer;">"끄기"</button>
                        </form>
                    } else {
                        if !totp_pending_secret.is_empty() {
                            <div style="display:flex; gap:20px; flex-wrap:wrap; margin-bottom:16px;">
                                if !totp_qr.is_empty() {
                                    <div style="background:#fff; border-radius:8px; padding:12px; align-self:flex-start;">
                                        <img src=(totp_qr.clone()) alt="2FA QR 코드" style="width:180px; height:180px; display:block;">
                                    </div>
                                }
                                <div style="flex:1; min-width:240px; background:rgba(251,191,36,0.08); border:1px solid rgba(251,191,36,0.25); border-radius:8px; padding:16px; align-self:flex-start;">
                                    <div style="font-size:13px; font-weight:600; color:var(--amber); margin-bottom:8px;">"QR 스캔 또는 키 수동 등록"</div>
                                    <div style="font-family:monospace; font-size:15px; letter-spacing:0.1em; color:var(--text2); background:var(--bg); border-radius:6px; padding:10px 12px; margin-bottom:8px; user-select:all;">(totp_pending_secret.clone())</div>
                                    <div style="font-size:11px; color:var(--muted2); word-break:break-all;">(totp_uri.clone())</div>
                                </div>
                            </div>
                            <form action="/admin/profile/totp/enable" method="POST" style="max-width:420px;">
                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                <div style="margin-bottom:14px;">
                                    <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"앱에 표시된 6자리 코드"</label>
                                    <input type="text" name="code" required="true" inputmode="numeric" pattern="[0-9]*" maxlength="6" autocomplete="one-time-code" placeholder="000000" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                                </div>
                                <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"확인하고 켜기"</button>
                            </form>
                        } else {
                            <form action="/admin/profile/totp/setup" method="POST">
                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"켜기 (새 키 발급)"</button>
                            </form>
                        }
                    }
                </div>

                <div style="margin-top:24px; background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px;">
                    <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"관리자 계정 정보"</h2>
                    <p style="font-size:12px; color:var(--muted2); margin-bottom:16px;">"현재 관리자 계정 상태"</p>

                    <div style="display:grid; grid-template-columns:repeat(3,1fr); gap:16px;">
                        <div style="background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:16px;">
                            <div style="font-size:11px; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; margin-bottom:4px;">"이메일"</div>
                            <div style="font-size:14px; font-weight:600; color:var(--text2);">(admin_email.clone())</div>
                        </div>
                        <div style="background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:16px;">
                            <div style="font-size:11px; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; margin-bottom:4px;">"인증 방식"</div>
                            <div style="font-size:14px; font-weight:600; color:var(--green);">"이메일 + 비밀번호"</div>
                        </div>
                        <div style="background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:16px;">
                            <div style="font-size:11px; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; margin-bottom:4px;">"상태"</div>
                            <div style="font-size:14px; font-weight:600; color:var(--green);">"활성"</div>
                        </div>
                    </div>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/profile/email")]
pub async fn admin_update_email(
    cx: &Cx,
    Form(input): Form<EmailForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.profile.email")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::require_admin_area(db, cx).await {
        Some(a) => a.user,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    if !input.admin_email.is_empty() && input.admin_email.contains('@') {
        sqlx::query("UPDATE users SET email = $1, updated_at = NOW()::text WHERE id = $2")
            .bind(&input.admin_email)
            .bind(&me.id)
            .execute(db)
            .await
            .ok();
    }

    crate::auth::access::audit(
        db,
        cx,
        &me.id,
        "admin.profile.email",
        "user",
        &me.id,
        &input.admin_email,
    )
    .await;

    Ok(topcoat::router::error::see_other(
        "/admin/profile?updated=1",
    ))
}

#[route(POST "/admin/profile/password")]
pub async fn admin_update_password(
    cx: &Cx,
    Form(input): Form<PasswordForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.profile.password",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::require_admin_area(db, cx).await {
        Some(a) => a.user,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    if input.new_password != input.confirm_password {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=match",
        ));
    }

    if !crate::auth::session::password_ok(&input.new_password) {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=length",
        ));
    }

    // Verify current password against the users table.
    let current_valid = {
        use argon2::Argon2;
        use argon2::password_hash::{PasswordHash, PasswordVerifier};
        me.password_hash
            .as_deref()
            .and_then(|h| PasswordHash::new(h).ok())
            .map(|h| {
                Argon2::default()
                    .verify_password(input.current_password.as_bytes(), &h)
                    .is_ok()
            })
            .unwrap_or(false)
    };

    if !current_valid {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=current",
        ));
    }

    // Hash new password, store it, and revoke all sessions (re-login required).
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.new_password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();
    if hash.is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=hash",
        ));
    }

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW()::text WHERE id = $2")
        .bind(&hash)
        .bind(&me.id)
        .execute(db)
        .await
        .ok();
    crate::auth::session::destroy_user_sessions(db, &me.id).await;
    crate::auth::session::destroy_session(db, cx).await;

    crate::auth::access::audit(db, cx, &me.id, "admin.profile.password", "user", &me.id, "").await;

    Ok(topcoat::router::error::see_other("/admin/login?updated=1"))
}

#[derive(serde::Deserialize)]
pub struct TotpCodeForm {
    pub csrf_token: String,
    pub code: String,
}

#[derive(serde::Deserialize)]
pub struct TotpDisableForm {
    pub csrf_token: String,
    pub current_password: String,
}

fn totp_account(u: &crate::models::User) -> String {
    u.email
        .clone()
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| u.username.clone())
}

#[route(POST "/admin/profile/totp/setup")]
pub async fn admin_totp_setup(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.profile.totp.setup",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::require_admin_area(db, cx).await {
        Some(a) => a.user,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    let secret = crate::auth::totp::new_secret();
    sqlx::query("UPDATE users SET totp_secret = $1, totp_enabled = 0, updated_at = NOW()::text WHERE id = $2")
        .bind(&secret)
        .bind(&me.id)
        .execute(db)
        .await
        .ok();
    crate::auth::access::audit(db, cx, &me.id, "admin.totp.setup", "user", &me.id, "").await;

    Ok(topcoat::router::error::see_other("/admin/profile"))
}

#[route(POST "/admin/profile/totp/enable")]
pub async fn admin_totp_enable(
    cx: &Cx,
    Form(input): Form<TotpCodeForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.profile.totp.enable",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::require_admin_area(db, cx).await {
        Some(a) => a.user,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    if me.totp_secret.is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=totp",
        ));
    }
    if !crate::auth::totp::verify(&me.totp_secret, &totp_account(&me), &input.code) {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=totp",
        ));
    }
    sqlx::query("UPDATE users SET totp_enabled = 1, updated_at = NOW()::text WHERE id = $1")
        .bind(&me.id)
        .execute(db)
        .await
        .ok();
    crate::auth::access::audit(db, cx, &me.id, "admin.totp.enabled", "user", &me.id, "").await;

    Ok(topcoat::router::error::see_other(
        "/admin/profile?updated=1",
    ))
}

#[route(POST "/admin/profile/totp/disable")]
pub async fn admin_totp_disable(
    cx: &Cx,
    Form(input): Form<TotpDisableForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.profile.totp.disable",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let me = match crate::auth::access::require_admin_area(db, cx).await {
        Some(a) => a.user,
        _ => return Ok(topcoat::router::error::see_other("/admin/login")),
    };

    use argon2::Argon2;
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    let valid = me
        .password_hash
        .as_deref()
        .and_then(|h| PasswordHash::new(h).ok())
        .map(|h| {
            Argon2::default()
                .verify_password(input.current_password.as_bytes(), &h)
                .is_ok()
        })
        .unwrap_or(false);
    if !valid {
        return Ok(topcoat::router::error::see_other(
            "/admin/profile?error=current",
        ));
    }

    sqlx::query("UPDATE users SET totp_secret = '', totp_enabled = 0, updated_at = NOW()::text WHERE id = $1")
        .bind(&me.id)
        .execute(db)
        .await
        .ok();
    crate::auth::access::audit(db, cx, &me.id, "admin.totp.disabled", "user", &me.id, "").await;

    Ok(topcoat::router::error::see_other(
        "/admin/profile?updated=1",
    ))
}
