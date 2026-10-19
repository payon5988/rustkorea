use crate::db::Db;
use crate::pages::community::helpers::get_current_user;
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
struct EventWithCount {
    id: String,
    title: String,
    description: String,
    location: String,
    event_date: String,
    event_time: String,
    max_participants: i32,
    creator_id: String,
    created_at: String,
    creator_name: String,
    participant_count: i64,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct EventRegistrationUser {
    event_id: String,
    user_id: String,
    username: String,
}

/// Event list caps; `0` max_participants means unlimited.
pub const EVENTS_LIMIT: i64 = 50;
pub const EVENT_DESC_LEN: usize = 120;
pub const UNLIMITED_PARTICIPANTS: i32 = 0;

#[page("/events")]
pub async fn events_page(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let loc = crate::services::i18n::locale(cx);

    let events: Vec<EventWithCount> = sqlx::query_as::<_, EventWithCount>(
        "SELECT e.id, e.title, e.description, e.location, e.event_date, e.event_time,
                e.max_participants, e.creator_id, e.created_at,
                COALESCE(u.username, '') AS creator_name,
                (SELECT COUNT(*) FROM event_registrations er WHERE er.event_id = e.id) AS participant_count
         FROM events e LEFT JOIN users u ON u.id = e.creator_id
         WHERE e.event_date >= CURRENT_DATE
         ORDER BY e.event_date ASC LIMIT $1"
    )
    .bind(EVENTS_LIMIT)
    .fetch_all(db).await.unwrap_or_default();

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:32px;">
                <div>
                    <div style="font-size:11px; font-weight:700; color:#2563EB; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"EVENTS"</div>
                    <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em;">(loc.pick("이벤트", "Events"))</h1>
                </div>
            </div>

            if events.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="color:#6b7280;">(loc.pick("등록된 이벤트가 없습니다.", "No events yet."))</p>
                </div>
            } else {
                <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(340px, 1fr)); gap:20px;">
                    for ev in &events {
                        <a href=(format!("/events/{}", ev.id)) class="card" style="padding:24px; display:flex; flex-direction:column;">
                            <div style="display:flex; align-items:center; gap:8px; margin-bottom:12px;">
                                <span style="font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:#059669; background:#0596691a;">(loc.pick("일정", "Event"))</span>
                                <span style="font-size:12px; color:#9ca3af;">(ev.event_date.clone())</span>
                                if !ev.event_time.is_empty() {
                                    <span style="font-size:12px; color:#9ca3af;">(ev.event_time.clone())</span>
                                }
                            </div>
                            <h3 style="font-size:16px; font-weight:700; margin-bottom:8px;">(ev.title.clone())</h3>
                            <p style="font-size:13px; color:#6b7280; line-height:1.5; margin-bottom:16px; flex:1;">(ev.description.chars().take(EVENT_DESC_LEN).collect::<String>())</p>
                            <div style="display:flex; align-items:center; justify-content:space-between; font-size:12px; color:#9ca3af; border-top:1px solid #f3f4f6; padding-top:12px;">
                                <span>(format!("{} | {}", ev.location.clone(), ev.creator_name.clone()))</span>
                                <span>(format!("{}/{}", ev.participant_count, if ev.max_participants > UNLIMITED_PARTICIPANTS { ev.max_participants.to_string() } else { "∞".into() }))</span>
                            </div>
                        </a>
                    }
                </div>
            }
        </div>
    })
}

topcoat::router::path_param!(id);

#[page("/events/{id}")]
pub async fn event_detail(cx: &Cx) -> Result<impl View + '_> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let current_user = get_current_user(cx).await;
    let loc = crate::services::i18n::locale(cx);

    let ev: Option<EventWithCount> = sqlx::query_as::<_, EventWithCount>(
        "SELECT e.id, e.title, e.description, e.location, e.event_date, e.event_time,
                e.max_participants, e.creator_id, e.created_at,
                COALESCE(u.username, '') AS creator_name,
                (SELECT COUNT(*) FROM event_registrations er WHERE er.event_id = e.id) AS participant_count
         FROM events e LEFT JOIN users u ON u.id = e.creator_id
         WHERE e.id = $1"
    ).bind(id).fetch_optional(db).await.ok().flatten();

    let participants: Vec<EventRegistrationUser> = sqlx::query_as::<_, EventRegistrationUser>(
        "SELECT er.event_id, er.user_id, COALESCE(u.username, '') AS username
         FROM event_registrations er LEFT JOIN users u ON u.id = er.user_id
         WHERE er.event_id = $1 ORDER BY er.registered_at ASC",
    )
    .bind(id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let mut is_registered = false;
    if let Some(ref u) = current_user {
        is_registered = participants.iter().any(|p| p.user_id == u.id);
    }

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:720px;">
            <a href="/events" style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">(loc.pick("<- 이벤트 목록", "<- All events"))</a>
            if let Some(ev) = ev {
                <div style="display:flex; gap:8px; margin-bottom:12px;">
                    <span style="font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:#059669; background:#0596691a;">(loc.pick("일정", "Event"))</span>
                    <span style="font-size:13px; color:#6b7280;">(format!("{} {}", ev.event_date, ev.event_time))</span>
                </div>
                <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:12px;">(ev.title.clone())</h1>
                <div style="font-size:13px; color:#6b7280; margin-bottom:24px;">
                    <span>(match loc {
                        crate::services::i18n::Locale::Ko => format!("{} | {}명 참가", ev.location, ev.participant_count),
                        crate::services::i18n::Locale::En => format!("{} | {} attending", ev.location, ev.participant_count),
                    })</span>
                </div>
                <div class="card" style="padding:32px; margin-bottom:24px;">
                    <div style="font-size:14px; line-height:1.8; white-space:pre-wrap;">(ev.description.clone())</div>
                </div>
                if current_user.is_some() {
                    if is_registered {
                        <form method="POST" action=(format!("/events/{}/unregister", ev.id)) style="margin-bottom:24px;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <button type="submit" style="background:#DC2626; color:#fff; padding:10px 20px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">(loc.pick("참가 취소", "Cancel registration"))</button>
                        </form>
                    } else {
                        let is_full = ev.max_participants > UNLIMITED_PARTICIPANTS && ev.participant_count >= ev.max_participants as i64;
                        if is_full {
                            <div style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px; margin-bottom:24px; font-size:14px; color:#fca5a5;">(loc.pick("정원이 마감되었습니다.", "This event is full."))</div>
                        } else {
                            <form method="POST" action=(format!("/events/{}/register", ev.id)) style="margin-bottom:24px;">
                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                <button type="submit" style="background:#059669; color:#fff; padding:10px 20px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">(loc.pick("참가하기", "Register"))</button>
                            </form>
                        }
                    }
                }
                <div style="margin-bottom:24px;">
                    <h2 style="font-size:15px; font-weight:700; margin-bottom:12px;">(match loc {
                        crate::services::i18n::Locale::Ko => format!("참가자 {}명", ev.participant_count),
                        crate::services::i18n::Locale::En => format!("Attendees ({})", ev.participant_count),
                    })</h2>
                    <div style="display:flex; flex-wrap:wrap; gap:8px;">
                        for p in &participants {
                            <span style="font-size:13px; padding:4px 12px; background:#f3f4f6; border-radius:9999px;">(p.username.clone())</span>
                        }
                    </div>
                </div>
            } else {
                <div style="text-align:center; padding:80px 0;">
                    <p style="color:#6b7280; margin-bottom:16px;">(loc.pick("이벤트를 찾을 수 없습니다.", "Event not found."))</p>
                    <a href="/events" class="btn-primary" style="display:inline-flex;">(loc.pick("이벤트 목록으로", "Back to events"))</a>
                </div>
            }
        </div>
    })
}

#[route(POST "/events/{id}/register")]
pub async fn register_event(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?redirect=/events",
        ));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "events.register").await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let max: i32 = sqlx::query_scalar("SELECT max_participants FROM events WHERE id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let current: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM event_registrations WHERE event_id = $1")
            .bind(id)
            .fetch_one(db)
            .await
            .unwrap_or(0);
    if max > UNLIMITED_PARTICIPANTS && current >= max as i64 {
        return Ok(topcoat::router::error::see_other(format!("/events/{id}")));
    }
    sqlx::query("INSERT INTO event_registrations (event_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(id).bind(&user.id).execute(db).await.ok();
    Ok(topcoat::router::error::see_other(format!("/events/{id}")))
}

#[route(POST "/events/{id}/unregister")]
pub async fn unregister_event(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other("/auth/login"));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "events.unregister")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }
    sqlx::query("DELETE FROM event_registrations WHERE event_id = $1 AND user_id = $2")
        .bind(id)
        .bind(&user.id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other(format!("/events/{id}")))
}
