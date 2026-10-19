use crate::db::Db;

/// Message list caps and read-state flags.
pub const MESSAGES_LIMIT: i64 = 50;
pub const MSG_UNREAD: i32 = 0;
pub const MSG_READ: i32 = 1;
use crate::pages::community::helpers::{get_current_user, login_redirect};
use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct MessageWithUser {
    id: String,
    from_user_id: String,
    to_user_id: String,
    subject: String,
    content: String,
    is_read: i32,
    created_at: String,
    other_username: String,
}

topcoat::router::path_param!(id);

#[derive(serde::Deserialize)]
pub struct ComposeForm {
    pub csrf_token: String,
    pub to_username: String,
    pub subject: String,
    pub content: String,
}

#[page("/messages")]
pub async fn messages_page(cx: &Cx) -> Result<impl View + '_> {
    let Some(user) = get_current_user(cx).await else {
        return Err(login_redirect(cx, "/messages").await);
    };
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let loc = crate::services::i18n::locale(cx);

    let tab = topcoat::router::request::uri(cx)
        .query()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "tab")
                .map(|(_, v)| v.into_owned())
        })
        .unwrap_or_else(|| "inbox".into());

    let inbox: Vec<MessageWithUser> = sqlx::query_as::<_, MessageWithUser>(
        "SELECT m.id, m.from_user_id, m.to_user_id, m.subject, m.content, m.is_read, m.created_at,
                COALESCE(u.username, '') AS other_username
         FROM messages m LEFT JOIN users u ON u.id = m.from_user_id
         WHERE m.to_user_id = $1 ORDER BY m.created_at DESC LIMIT $2",
    )
    .bind(&user.id)
    .bind(MESSAGES_LIMIT)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let sent: Vec<MessageWithUser> = sqlx::query_as::<_, MessageWithUser>(
        "SELECT m.id, m.from_user_id, m.to_user_id, m.subject, m.content, m.is_read, m.created_at,
                COALESCE(u.username, '') AS other_username
         FROM messages m LEFT JOIN users u ON u.id = m.to_user_id
         WHERE m.from_user_id = $1 ORDER BY m.created_at DESC LIMIT $2",
    )
    .bind(&user.id)
    .bind(MESSAGES_LIMIT)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let unread = inbox.iter().filter(|m| m.is_read == MSG_UNREAD).count();

    let inbox_style: &'static str = if tab == "inbox" {
        "padding:10px 18px; font-size:14px; font-weight:700; color:#2563EB; border-bottom:2px solid #2563EB; display:inline-flex;"
    } else {
        "padding:10px 18px; font-size:14px; font-weight:500; color:#6b7280; border-bottom:2px solid transparent; display:inline-flex;"
    };
    let sent_style: &'static str = if tab == "sent" {
        "padding:10px 18px; font-size:14px; font-weight:700; color:#2563EB; border-bottom:2px solid #2563EB; display:inline-flex;"
    } else {
        "padding:10px 18px; font-size:14px; font-weight:500; color:#6b7280; border-bottom:2px solid transparent; display:inline-flex;"
    };
    let compose_style: &'static str = if tab == "compose" {
        "padding:10px 18px; font-size:14px; font-weight:700; color:#2563EB; border-bottom:2px solid #2563EB; display:inline-flex;"
    } else {
        "padding:10px 18px; font-size:14px; font-weight:500; color:#6b7280; border-bottom:2px solid transparent; display:inline-flex;"
    };

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:640px;">
            <div style="margin-bottom:24px;">
                <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em;">(loc.pick("쪽지", "Messages"))</h1>
                if unread > 0 {
                    <p style="font-size:13px; color:#6b7280; margin-top:4px;">(match loc {
                        crate::services::i18n::Locale::Ko => format!("읽지 않은 쪽지 {unread}개"),
                        crate::services::i18n::Locale::En => format!("{unread} unread"),
                    })</p>
                }
            </div>

            <div style="display:flex; gap:4px; border-bottom:1px solid #E4E4E7; margin-bottom:24px;">
                <a href="/messages?tab=inbox" style=(inbox_style)>(loc.pick("받은 쪽지", "Inbox"))</a>
                <a href="/messages?tab=sent" style=(sent_style)>(loc.pick("보낸 쪽지", "Sent"))</a>
                <a href="/messages?tab=compose" style=(compose_style)>(loc.pick("새 쪽지", "New message"))</a>
            </div>

            if tab == "inbox" {
                if inbox.is_empty() {
                    <div class="card" style="padding:60px; text-align:center;"><p style="color:#6b7280;">(loc.pick("받은 쪽지가 없습니다.", "Your inbox is empty."))</p></div>
                } else {
                    <div style="display:flex; flex-direction:column; gap:2px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                        for m in &inbox {
                            let bg = if m.is_read == MSG_UNREAD { "#f0f7ff" } else { "#fff" };
                            <a href=(format!("/messages/{}", m.id)) style=(format!("display:flex; flex-direction:column; padding:14px 20px; background:{bg}; gap:4px;"))>
                                <div style="display:flex; align-items:center; gap:8px;">
                                    <span style="font-size:13px; font-weight:600;">(m.other_username.clone())</span>
                                    <span style="font-size:14px; flex:1;">(m.subject.clone())</span>
                                    <span style="font-size:12px; color:#9ca3af;">(m.created_at.split(' ').next().unwrap_or(""))</span>
                                    if m.is_read == MSG_UNREAD {
                                        <span style="width:8px; height:8px; border-radius:50%; background:#2563EB; flex-shrink:0;"></span>
                                    }
                                </div>
                            </a>
                        }
                    </div>
                }
            } else if tab == "sent" {
                if sent.is_empty() {
                    <div class="card" style="padding:60px; text-align:center;"><p style="color:#6b7280;">(loc.pick("보낸 쪽지가 없습니다.", "No sent messages."))</p></div>
                } else {
                    <div style="display:flex; flex-direction:column; gap:2px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                        for m in &sent {
                            <div style="display:flex; flex-direction:column; padding:14px 20px; background:#fff; gap:4px;">
                                <div style="display:flex; align-items:center; gap:8px;">
                                    <span style="font-size:12px; color:#9ca3af;">"→ "</span>
                                    <span style="font-size:13px; font-weight:600;">(m.other_username.clone())</span>
                                    <span style="font-size:14px; flex:1;">(m.subject.clone())</span>
                                    <span style="font-size:12px; color:#9ca3af;">(m.created_at.split(' ').next().unwrap_or(""))</span>
                                </div>
                            </div>
                        }
                    </div>
                }
            } else if tab == "compose" {
                <form method="POST" action="/messages/send" style="background:#fff; border:1px solid #E4E4E7; border-radius:8px; padding:24px; display:flex; flex-direction:column; gap:16px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div>
                        <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:6px;">(loc.pick("받는 사람", "To"))</label>
                        <input type="text" name="to_username" required="true" placeholder=(loc.pick("사용자 이름", "Username")) class="input">
                    </div>
                    <div>
                        <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:6px;">(loc.pick("제목", "Subject"))</label>
                        <input type="text" name="subject" required="true" placeholder=(loc.pick("쪽지 제목", "Subject")) class="input">
                    </div>
                    <div>
                        <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:6px;">(loc.pick("내용", "Message"))</label>
                        <textarea name="content" rows="8" required="true" placeholder=(loc.pick("쪽지 내용을 입력하세요...", "Write your message...")) class="input"></textarea>
                    </div>
                    <button type="submit" class="btn-primary" style="align-self:flex-start;">(loc.pick("쪽지 보내기", "Send"))</button>
                </form>
            }
        </div>
    })
}

#[page("/messages/{id}")]
pub async fn message_detail(cx: &Cx) -> Result<impl View + '_> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let Some(user) = get_current_user(cx).await else {
        return Err(login_redirect(cx, "/messages").await);
    };
    let loc = crate::services::i18n::locale(cx);

    let msg: Option<MessageWithUser> = sqlx::query_as::<_, MessageWithUser>(
        "SELECT m.id, m.from_user_id, m.to_user_id, m.subject, m.content, m.is_read, m.created_at,
                COALESCE(u.username, '') AS other_username
         FROM messages m LEFT JOIN users u ON u.id = CASE WHEN m.from_user_id = $2 THEN m.to_user_id ELSE m.from_user_id END
         WHERE m.id = $1 AND (m.from_user_id = $2 OR m.to_user_id = $2)"
    ).bind(id).bind(&user.id).fetch_optional(db).await.ok().flatten();

    if let Some(ref m) = msg {
        if m.is_read == MSG_UNREAD && m.to_user_id == user.id {
            sqlx::query("UPDATE messages SET is_read = $1 WHERE id = $2")
                .bind(MSG_READ)
                .bind(id)
                .execute(db)
                .await
                .ok();
        }
    }

    let other_name = msg
        .as_ref()
        .map(|m| m.other_username.clone())
        .unwrap_or_default();

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:640px;">
            <a href="/messages" style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">(loc.pick("<- 쪽지함", "<- Messages"))</a>
            if let Some(m) = msg {
                <div class="card" style="padding:24px;">
                    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:16px; padding-bottom:16px; border-bottom:1px solid #f3f4f6;">
                        <div>
                            <h1 style="font-size:16px; font-weight:700;">(m.subject.clone())</h1>
                            <p style="font-size:12px; color:#9ca3af; margin-top:4px;">(format!("{} | {}", other_name, m.created_at))</p>
                        </div>
                        <a href=("/messages?tab=compose".to_string()) style="font-size:13px; color:#2563EB; font-weight:500;">(loc.pick("답장", "Reply"))</a>
                    </div>
                    <div style="font-size:14px; line-height:1.8; white-space:pre-wrap;">(m.content.clone())</div>
                </div>
            } else {
                <div style="text-align:center; padding:80px 0;"><p style="color:#6b7280;">(loc.pick("쪽지를 찾을 수 없습니다.", "Message not found."))</p></div>
            }
        </div>
    })
}

#[route(POST "/messages/send")]
pub async fn send_message(
    cx: &Cx,
    Form(input): Form<ComposeForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?redirect=/messages",
        ));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "messages.send").await {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let target: Option<crate::models::User> =
        sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(&input.to_username)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
    if let Some(t) = target {
        sqlx::query("INSERT INTO messages (from_user_id, to_user_id, subject, content) VALUES ($1, $2, $3, $4)")
            .bind(&user.id).bind(&t.id).bind(&input.subject).bind(&input.content)
            .execute(db).await.ok();
        crate::services::notifications::create_notification(
            db,
            &t.id,
            "message",
            Some(&user.id),
            None,
            &format!("{}님이 쪽지를 보냈습니다: {}", user.username, input.subject),
        )
        .await;
    }
    Ok(topcoat::router::error::see_other("/messages?tab=sent"))
}
