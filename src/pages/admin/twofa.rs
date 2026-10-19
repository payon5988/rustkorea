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
use crate::db::Db;

#[derive(serde::Deserialize)]
pub struct TotpForm {
    pub csrf_token: String,
    pub code: String,
}

fn totp_account(username: &str, email: &Option<String>) -> String {
    email
        .clone()
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| username.to_string())
}

#[page("/admin/2fa")]
pub async fn twofa_page(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let pending = sessions::pending_user(db, cx).await;
    if pending.is_none() {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"2단계 인증 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body>
            <div class="wrap">
                <div class="logo">
                    <h1>"2단계 인증"</h1>
                    <p>"인증 앱의 6자리 코드를 입력하세요"</p>
                </div>
                <div class="card">
                    <form action="/admin/2fa" method="POST">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div class="field">
                            <label for="code">"인증 코드"</label>
                            <input type="text" id="code" name="code" required="true" inputmode="numeric"
                                pattern="[0-9]*" maxlength="6" autocomplete="one-time-code"
                                placeholder="000000" autofocus="true">
                        </div>
                        <button type="submit" class="btn btn-block">"확인"</button>
                    </form>
                </div>
                <div class="site-link">
                    <form action="/admin/logout" method="POST" style="display:inline;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <button type="submit" style="background:none; border:none; padding:0; font:inherit; color:inherit; text-decoration:underline; cursor:pointer;">"취소하고 로그아웃"</button>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/2fa")]
pub async fn twofa_submit(
    cx: &Cx,
    Form(input): Form<TotpForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.2fa").await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let fail = || topcoat::router::error::see_other("/admin/2fa?error=1");

    if !sessions::same_origin_ok(cx) {
        return Ok(fail());
    }
    let Some(u) = sessions::pending_user(db, cx).await else {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    };
    if !sessions::rate_limit_ok(db, cx, &u.id).await {
        return Ok(fail());
    }
    sessions::record_login_attempt(db, cx, &u.id).await;

    if u.totp_enabled != 1 || u.totp_secret.is_empty() {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    if !crate::auth::totp::verify(
        &u.totp_secret,
        &totp_account(&u.username, &u.email),
        &input.code,
    ) {
        crate::auth::access::audit(db, cx, &u.id, "admin.login.2fa_failed", "user", &u.id, "")
            .await;
        return Ok(fail());
    }
    if !sessions::complete_2fa(db, cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sessions::clear_login_attempts(db, cx, &u.id).await;
    crate::auth::access::audit(db, cx, &u.id, "admin.login", "user", &u.id, "2fa").await;
    Ok(topcoat::router::error::see_other("/admin"))
}
