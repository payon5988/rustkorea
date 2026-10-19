use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::db::Db;

/// Notification list cap and read-state flags.
pub const NOTIF_LIMIT: i64 = 50;
pub const NOTIF_UNREAD: i32 = 0;
pub const NOTIF_READ: i32 = 1;
use crate::pages::community::helpers::{get_current_user, login_redirect};

topcoat::router::path_param!(id);

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct NotiWithFrom {
    id: String,
    noti_type: String,
    from_user_id: Option<String>,
    post_id: Option<String>,
    message: String,
    is_read: i32,
    created_at: String,
    from_username: Option<String>,
}

fn noti_icon(n: &str) -> &'static str {
    match n {
        "comment" => "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",
        "like" => {
            "M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"
        }
        "mention" => {
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M22 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75"
        }
        "message" => {
            "M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z M22 6l-10 7L2 6"
        }
        _ => "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9",
    }
}

fn noti_color(n: &str) -> &'static str {
    match n {
        "comment" => "#2563EB",
        "like" => "#DC2626",
        "mention" => "#7C3AED",
        "message" => "#059669",
        _ => "#6b7280",
    }
}

#[page("/notifications")]
pub async fn notifications_page(cx: &Cx) -> Result<impl View + '_> {
    let Some(user) = get_current_user(cx).await else {
        return Err(login_redirect(cx, "/notifications").await);
    };
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let loc = crate::services::i18n::locale(cx);

    let notifications: Vec<NotiWithFrom> = sqlx::query_as::<_, NotiWithFrom>(
        "SELECT n.id, n.noti_type, n.from_user_id, n.post_id, n.message, n.is_read, n.created_at,
                COALESCE(u.username, '') AS from_username
         FROM notifications n LEFT JOIN users u ON u.id = n.from_user_id
         WHERE n.user_id = $1 ORDER BY n.created_at DESC LIMIT $2",
    )
    .bind(&user.id)
    .bind(NOTIF_LIMIT)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let unread_count = notifications
        .iter()
        .filter(|n| n.is_read == NOTIF_UNREAD)
        .count();

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:640px;">
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:32px;">
                <div>
                    <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em;">(loc.pick("알림", "Notifications"))</h1>
                    if unread_count > 0 {
                        <p style="font-size:13px; color:#6b7280; margin-top:4px;">(match loc {
                            crate::services::i18n::Locale::Ko => format!("읽지 않은 알림 {unread_count}개"),
                            crate::services::i18n::Locale::En => format!("{unread_count} unread"),
                        })</p>
                    }
                </div>
                if unread_count > 0 {
                    <form method="POST" action="/notifications/read-all">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <button type="submit" style="font-size:13px; color:#2563EB; font-weight:500; background:none; border:none; cursor:pointer;">(loc.pick("모두 읽음", "Mark all as read"))</button>
                    </form>
                }
            </div>

            if notifications.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#d1d5db" stroke-width="1.5" stroke-linecap="round" style="margin:0 auto 16px;">
                        <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
                        <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
                    </svg>
                    <p style="color:#6b7280;">(loc.pick("아직 알림이 없습니다.", "No notifications yet."))</p>
                </div>
            } else {
                <div style="display:flex; flex-direction:column; gap:2px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                    for n in &notifications {
                        let bg = if n.is_read == NOTIF_UNREAD { "#f0f7ff" } else { "#fff" };
                        let weight = if n.is_read == NOTIF_UNREAD { "600" } else { "400" };
                        let link = n.post_id.as_deref().map(|pid| format!("/community/{pid}")).unwrap_or_else(|| "/notifications".to_string());
                        let icon_path = noti_icon(&n.noti_type);
                        let icon_color = noti_color(&n.noti_type);
                        <a href=(link) style=(format!("display:flex; align-items:flex-start; padding:14px 20px; background:{bg}; gap:12px; transition:background 0.15s;"))>
                            <div style=(format!("flex-shrink:0; width:32px; height:32px; border-radius:50%; background:{icon_color}1a; display:flex; align-items:center; justify-content:center; margin-top:2px;"))>
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke=(icon_color) stroke-width="2" stroke-linecap="round"><path d=(icon_path)/></svg>
                            </div>
                            <div style="flex:1; min-width:0;">
                                <p style=(format!("font-size:14px; color:#16181d; font-weight:{weight}; line-height:1.5;"))>(n.message.clone())</p>
                                <span style="font-size:12px; color:#9ca3af;">(n.created_at.split(' ').next().unwrap_or("").to_string())</span>
                            </div>
                            if n.is_read == NOTIF_UNREAD {
                                <div style="flex-shrink:0; width:8px; height:8px; border-radius:50%; background:#2563EB; margin-top:6px;"></div>
                            }
                        </a>
                    }
                </div>
            }
        </div>
    })
}

#[route(POST "/notifications/{id}/read")]
pub async fn mark_notification_read(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    if let Some(user) = get_current_user(cx).await {
        if !crate::auth::session::verify_csrf_logged(
            db,
            cx,
            &input.csrf_token,
            "notifications.read",
        )
        .await
        {
            return Ok(topcoat::router::error::see_other("/"));
        }
        sqlx::query("UPDATE notifications SET is_read = $1 WHERE id = $2 AND user_id = $3")
            .bind(NOTIF_READ)
            .bind(id)
            .bind(&user.id)
            .execute(db)
            .await
            .ok();
    }
    Ok(topcoat::router::error::see_other("/notifications"))
}

#[route(POST "/notifications/read-all")]
pub async fn mark_all_read(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    if let Some(user) = get_current_user(cx).await {
        if !crate::auth::session::verify_csrf_logged(
            db,
            cx,
            &input.csrf_token,
            "notifications.read_all",
        )
        .await
        {
            return Ok(topcoat::router::error::see_other("/"));
        }
        sqlx::query("UPDATE notifications SET is_read = $1 WHERE user_id = $2 AND is_read = $3")
            .bind(NOTIF_READ)
            .bind(&user.id)
            .bind(NOTIF_UNREAD)
            .execute(db)
            .await
            .ok();
    }
    Ok(topcoat::router::error::see_other("/notifications"))
}
