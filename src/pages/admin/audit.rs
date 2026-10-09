use std::future::Future;
use topcoat::router::request::uri;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct AuditRow {
    action: String,
    target_type: String,
    target_id: String,
    detail: String,
    ip: String,
    created_at: String,
    actor_name: Option<String>,
}

#[page("/admin/audit")]
pub async fn admin_audit_page(cx: &Cx) -> Result<impl View + '_> {
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
    let filter = uri(cx)
        .query()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "action")
                .map(|(_, v)| v.into_owned())
        })
        .unwrap_or_default();

    let logs: Vec<AuditRow> = if filter.is_empty() {
        sqlx::query_as::<_, AuditRow>(
            "SELECT a.action, a.target_type, a.target_id, a.detail, a.ip, a.created_at,
                    u.username AS actor_name
             FROM audit_logs a LEFT JOIN users u ON u.id = a.actor_id
             ORDER BY a.created_at DESC LIMIT 100",
        )
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, AuditRow>(
            "SELECT a.action, a.target_type, a.target_id, a.detail, a.ip, a.created_at,
                    u.username AS actor_name
             FROM audit_logs a LEFT JOIN users u ON u.id = a.actor_id
             WHERE a.action LIKE $1 || '%' ORDER BY a.created_at DESC LIMIT 100",
        )
        .bind(&filter)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    };

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"감사 로그 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/audit".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700; color:#f9fafb; letter-spacing:-0.01em;">"감사 로그"</h1>
                        <p style="font-size:13px; color:#6b7280; margin-top:2px;">"최근 100건 (관리자 로그인·권한·사용자·콘텐츠 변경)"</p>
                    </div>
                    <form action="/admin/audit" method="GET" style="display:flex; gap:8px;">
                        <input type="text" name="action" value=(filter.clone()) placeholder="액션 필터 (예: admin.)" style="background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:8px 12px; font-size:13px; color:#e5e7eb; outline:none; width:220px;">
                        <button type="submit" style="background:#1a1d27; border:1px solid #2a2d3a; color:#e5e7eb; padding:8px 16px; border-radius:8px; font-size:13px; cursor:pointer;">"검색"</button>
                    </form>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden;">
                    <table style="width:100%; border-collapse:collapse; font-size:13px;">
                        <thead>
                            <tr>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"시간"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"행위자"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"액션"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"대상"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"IP"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for log in &logs {
                                <tr>
                                    <td style="padding:12px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#6b7280; font-size:12px; white-space:nowrap;">(log.created_at.get(..19).unwrap_or("").to_string())</td>
                                    <td style="padding:12px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#d1d5db;">(log.actor_name.clone().unwrap_or_else(|| "-".into()))</td>
                                    <td style="padding:12px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#d1d5db; font-family:monospace; font-size:12px;">(log.action.clone())</td>
                                    <td style="padding:12px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#9ca3af; font-size:12px;">(format!("{} {}", log.target_type, log.target_id).trim().to_string())</td>
                                    <td style="padding:12px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#4b5563; font-size:12px; font-family:monospace;">(log.ip.clone())</td>
                                </tr>
                            }
                        </tbody>
                    </table>
                    if logs.is_empty() {
                        <p style="padding:40px; text-align:center; font-size:13px; color:#4b5563;">"기록이 없습니다"</p>
                    }
                </div>
            </div>
        </body>
        </html>
    })
}
