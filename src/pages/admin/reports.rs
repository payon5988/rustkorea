use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

topcoat::router::path_param!(id);

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct ReportWithUser {
    id: String,
    reporter_id: String,
    target_type: String,
    target_id: String,
    reason: String,
    status: String,
    admin_note: String,
    created_at: String,
    reporter_name: String,
}

#[page("/admin/reports")]
pub async fn admin_reports(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let reports: Vec<ReportWithUser> = sqlx::query_as::<_, ReportWithUser>(
        "SELECT r.id, r.reporter_id, r.target_type, r.target_id, r.reason, r.status, r.admin_note, r.created_at,
                COALESCE(u.username, r.reporter_id) AS reporter_name
         FROM reports r LEFT JOIN users u ON u.id = r.reporter_id
         ORDER BY r.created_at DESC"
    ).fetch_all(db).await.unwrap_or_default();

    Ok(view! {
        <html lang="ko"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>"신고 관리 - Admin"</title><style>{(ADMIN_STYLE)}</style></head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/reports".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:24px;">"신고 관리"</h1>
                if reports.is_empty() {
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:64px; text-align:center;">
                        <p style="font-size:14px; color:#6b7280;">"신고 내역이 없습니다."</p>
                    </div>
                } else {
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead><tr style="border-bottom:1px solid #2a2d3a; text-align:left;">
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"신고자"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"대상"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"사유"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"상태"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; text-align:right;">"관리"</th>
                            </tr></thead>
                            <tbody>
                                for r in &reports {
                                    let status_color = match r.status.as_str() { "pending" => "#F59E0B", "resolved" => "#10B981", _ => "#6b7280" };
                                    <tr style="border-bottom:1px solid rgba(255,255,255,0.03);">
                                        <td style="padding:12px 16px; color:#9ca3af;">(r.reporter_name.clone())</td>
                                        <td style="padding:12px 16px; color:#e5e7eb;">(format!("{}: {}", r.target_type, r.target_id))</td>
                                        <td style="padding:12px 16px; color:#e5e7eb; max-width:200px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;">(r.reason.clone())</td>
                                        <td style="padding:12px 16px;"><span style=(format!("font-size:12px; padding:2px 8px; border-radius:9999px; color:{status_color}; background:{status_color}1a;"))>(r.status.clone())</span></td>
                                        <td style="padding:12px 16px; text-align:right;">
                                            <div style="display:flex; gap:6px; justify-content:flex-end;">
                                                if r.status == "pending" {
                                                    <form action=(format!("/admin/reports/{}/resolve", r.id)) method="POST" style="display:inline;"><button type="submit" style="font-size:12px; color:#10B981; background:none; border:none; cursor:pointer; font-weight:600;">"해결"</button></form>
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <form action=(format!("/admin/reports/{}/dismiss", r.id)) method="POST" style="display:inline;"><button type="submit" style="font-size:12px; color:#6b7280; background:none; border:none; cursor:pointer; font-weight:600;">"기각"</button></form>
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                }
                                            </div>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                }
            </div>
        </body></html>
    })
}

#[route(POST "/admin/reports/{id}/resolve")]
pub async fn resolve_report(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.reports.x.resolve",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("UPDATE reports SET status = 'resolved', resolved_at = NOW()::text WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/reports"))
}

#[route(POST "/admin/reports/{id}/dismiss")]
pub async fn dismiss_report(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.reports.x.dismiss",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("UPDATE reports SET status = 'dismissed', resolved_at = NOW()::text WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/reports"))
}
