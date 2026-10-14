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
use crate::models::Inquiry;

async fn require_products(cx: &Cx) -> bool {
    let adb: &crate::db::Db = topcoat::context::app_context(cx);
    crate::auth::access::require_perm(adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
        .await
        .is_none()
}

#[derive(serde::Deserialize)]
pub struct InquiryStatusForm {
    pub csrf_token: String,
    pub id: String,
    pub status: String,
}

#[route(POST "/admin/inquiries/status")]
pub async fn update_inquiry_status(
    cx: &Cx,
    Form(input): Form<InquiryStatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.inquiries.status",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    if !matches!(input.status.as_str(), "new" | "read" | "replied") {
        return Ok(topcoat::router::error::see_other("/admin/inquiries"));
    }
    sqlx::query("UPDATE inquiries SET status = $1 WHERE id = $2")
        .bind(&input.status)
        .bind(&input.id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/inquiries"))
}

#[page("/admin/inquiries")]
pub async fn admin_inquiries(cx: &Cx) -> Result<impl View + '_> {
    if require_products(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let inquiries: Vec<Inquiry> =
        sqlx::query_as::<_, Inquiry>("SELECT * FROM inquiries ORDER BY created_at DESC LIMIT 200")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    let titles: Vec<(String, String)> = sqlx::query_as("SELECT p.id, p.title FROM products p")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    let title_map: std::collections::HashMap<String, String> = titles.into_iter().collect();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"문의 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/inquiries".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700;">"문의 관리"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"상품 도입 문의 접수 내역"</p>
                </div>
                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                    <table style="width:100%; font-size:13px; border-collapse:collapse;">
                        <thead>
                            <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"상품"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"이름/연락처"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"내용"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"접수일"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"상태"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for q in &inquiries {
                                <tr style="border-bottom:1px solid var(--line);">
                                    <td style="padding:14px 20px; color:var(--muted); font-size:12px;">(q.product_id.clone().and_then(|id| title_map.get(&id).cloned()).unwrap_or_else(|| "-".to_string()))</td>
                                    <td style="padding:14px 20px;">
                                        <div style="font-weight:600; color:var(--text3);">(q.name.clone())</div>
                                        <div style="font-size:11px; color:var(--dim); margin-top:2px;">(q.contact.clone())</div>
                                    </td>
                                    <td style="padding:14px 20px; color:var(--text3); font-size:12px; max-width:320px;">(q.message.clone())</td>
                                    <td style="padding:14px 20px; color:var(--dim); font-size:12px; white-space:nowrap;">(q.created_at.get(..10).unwrap_or("").to_string())</td>
                                    <td style="padding:14px 20px; text-align:right;">
                                        <form action="/admin/inquiries/status" method="POST" style="display:inline-flex; gap:6px; align-items:center;">
                                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                            <input type="hidden" name="id" value=(q.id.clone())>
                                            <select name="status" style="background:var(--bg); border:1px solid var(--border); border-radius:6px; padding:6px 8px; font-size:12px; color:var(--text2); outline:none;">
                                                <option value="new" selected=(q.status == "new")>"미확인"</option>
                                                <option value="read" selected=(q.status == "read")>"확인"</option>
                                                <option value="replied" selected=(q.status == "replied")>"답변완료"</option>
                                            </select>
                                            <button type="submit" style="background:none; border:1px solid rgba(96,165,250,0.3); color:var(--blue); padding:6px 10px; border-radius:6px; font-size:12px; cursor:pointer;">"저장"</button>
                                        </form>
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>
            </div>
        </body>
        </html>
    })
}
