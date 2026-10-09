use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::Event;
use std::collections::HashMap;
use std::future::Future;
use topcoat::router::request::uri;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

topcoat::router::path_param!(id);

#[derive(serde::Deserialize)]
pub struct EventForm {
    pub csrf_token: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub event_date: String,
    pub event_time: String,
    pub max_participants: String,
}

#[derive(serde::Deserialize)]
pub struct EventUpdateForm {
    pub csrf_token: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub event_date: String,
    pub event_time: String,
    pub max_participants: String,
}

#[page("/admin/events")]
pub async fn admin_events_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "events.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let events: Vec<Event> =
        sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY event_date DESC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    let input_style = "width:100%; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;";
    let label_style =
        "display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;";

    let flatpickr_css = "https://cdn.jsdelivr.net/npm/flatpickr/dist/flatpickr.min.css";
    let flatpickr_ko = "https://cdn.jsdelivr.net/npm/flatpickr/dist/l10n/ko.js";
    let flatpickr_js = "https://cdn.jsdelivr.net/npm/flatpickr";

    Ok(view! {
        <html lang="ko"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>"이벤트 관리 - Admin"</title>
            <link rel="stylesheet" href=(flatpickr_css) />
            <style>
                {r#"
                .flatpickr-calendar { border-radius: 10px !important; border: 1px solid #2a2d3a !important; background: #1a1d27 !important; box-shadow: 0 8px 24px rgba(0,0,0,0.4) !important; font-family: inherit !important; }
                .flatpickr-months { background: #1a1d27 !important; border-bottom: 1px solid #2a2d3a !important; }
                .flatpickr-current-month { color: #e5e7eb !important; background: #1a1d27 !important; }
                .flatpickr-current-month .flatpickr-monthDropdown-months { background: #1a1d27 !important; color: #e5e7eb !important; border: 1px solid #2a2d3a !important; border-radius: 4px !important; }
                span.flatpickr-weekday { color: #9ca3af !important; background: #1a1d27 !important; }
                .flatpickr-day { color: #e5e7eb !important; border-radius: 6px !important; border-color: transparent !important; }
                .flatpickr-day:hover { background: rgba(222,91,48,0.2) !important; border-color: rgba(222,91,48,0.3) !important; color: #f9fafb !important; }
                .flatpickr-day.selected { background: #DE5B30 !important; border-color: #DE5B30 !important; color: #fff !important; }
                .flatpickr-day.prevMonthDay, .flatpickr-day.nextMonthDay { color: #4b5563 !important; }
                .flatpickr-day.today { border-color: #DE5B30 !important; }
                .flatpickr-day.today:hover { background: rgba(222,91,48,0.2) !important; }
                .flatpickr-prev-month, .flatpickr-next-month { fill: #9ca3af !important; }
                .flatpickr-prev-month:hover, .flatpickr-next-month:hover { fill: #DE5B30 !important; }
                .flatpickr-time { border-top: 1px solid #2a2d3a !important; background: #1a1d27 !important; }
                .flatpickr-time input { color: #e5e7eb !important; background: #1a1d27 !important; text-shadow: 0 0 1px rgba(229,231,235,0.3) !important; }
                .flatpickr-time .flatpickr-am-pm { color: #9ca3af !important; }
                .flatpickr-time .flatpickr-am-pm:hover { color: #DE5B30 !important; }
                span.numInputWrapper { background: transparent !important; }
                span.numInputWrapper span.arrowUp, span.numInputWrapper span.arrowDown { border-color: #4b5563 !important; }
                span.numInputWrapper span.arrowUp:hover, span.numInputWrapper span.arrowDown:hover { border-color: #DE5B30 !important; background: rgba(222,91,48,0.1) !important; }
                span.numInputWrapper span.arrowUp::after, span.numInputWrapper span.arrowDown::after { border-color: #9ca3af !important; }
                span.numInputWrapper:hover { background: transparent !important; }
                .flatpickr-months .flatpickr-next-month flatpickr-monthDropdown-months { background: #1a1d27 !important; }
                input.flatpickr-input { background: #0f1117 !important; color: #e5e7eb !important; border: 1px solid #2a2d3a !important; border-radius: 8px !important; padding: 10px 14px !important; }
                "#}
            </style>
            <style>{(ADMIN_STYLE)}</style>
            <script src=(flatpickr_js)></script>
            <script src=(flatpickr_ko)></script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/events".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700;">"이벤트 관리"</h1>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; margin-bottom:24px;">
                    <h2 style="font-size:15px; font-weight:700; margin-bottom:16px;">"새 이벤트"</h2>
                    <form action="/admin/events/create" method="POST" style="display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="grid-column:1 / -1;"><label style=(label_style)>"제목"</label><input type="text" name="title" required="true" style=(input_style)></div>
                        <div style="grid-column:1 / -1;"><label style=(label_style)>"설명"</label><textarea name="description" rows="3" style=(input_style)></textarea></div>
                        <div><label style=(label_style)>"장소"</label><input type="text" name="location" style=(input_style)></div>
                        <div><label style=(label_style)>"최대 인원 (0=무제한)"</label><input type="number" name="max_participants" value="0" min="0" style=(input_style)></div>
                        <div><label style=(label_style)>"날짜"</label><input type="text" id="event_date_picker" name="event_date" required="true" placeholder="날짜 선택" readonly="true" style=(input_style)></div>
                        <div><label style=(label_style)>"시간"</label><input type="text" id="event_time_picker" name="event_time" placeholder="시간 선택" readonly="true" style=(input_style)></div>
                        <div style="grid-column:1 / -1;"><button type="submit" style="background:#DE5B30; color:#fff; padding:10px 24px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">"이벤트 생성"</button></div>
                    </form>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:16px 20px; margin-bottom:16px; display:flex; gap:12px; align-items:center;">
                    <input type="text" id="event-search" placeholder="제목으로 검색..." oninput=(r#"var q=this.value.toLowerCase();document.querySelectorAll('#events-tbody tr').forEach(function(r){r.style.display=r.textContent.toLowerCase().includes(q)?'':'none'})"#) style="flex:1; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;">
                    <input type="date" id="event-date-filter" onchange=(r#"var d=this.value;document.querySelectorAll('#events-tbody tr').forEach(function(r){if(!d){r.style.display=''}else{var c=r.querySelector('td:nth-child(2)');r.style.display=c&&c.textContent.trim()===d?'':'none'}})"#) style="background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;">
                </div>

                if events.is_empty() {
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:48px; text-align:center;">
                        <p style="color:#6b7280;">"이벤트가 없습니다."</p>
                    </div>
                } else {
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead><tr style="border-bottom:1px solid #2a2d3a; text-align:left;">
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"제목"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"날짜"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase;">"장소"</th>
                                <th style="padding:12px 16px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; text-align:right;">"관리"</th>
                            </tr></thead>
                            <tbody id="events-tbody">
                                for ev in &events {
                                    <tr style="border-bottom:1px solid rgba(255,255,255,0.03);">
                                        <td style="padding:12px 16px; color:#e5e7eb; font-weight:600;">(ev.title.clone())</td>
                                        <td style="padding:12px 16px; color:#9ca3af;">(ev.event_date.clone())</td>
                                        <td style="padding:12px 16px; color:#9ca3af;">(ev.location.clone())</td>
                                        <td style="padding:12px 16px; text-align:right;">
                                            <a href=(format!("/admin/events/edit?id={}", ev.id)) style="font-size:12px; color:#93c5fd; background:none; border:none; cursor:pointer; font-weight:600; text-decoration:none; margin-right:8px;">"수정"</a>
                                            <form action=(format!("/admin/events/{}/delete", ev.id)) method="POST" onsubmit="return confirm('이벤트를 삭제하시겠습니까?')" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <button type="submit" style="font-size:12px; color:#f87171; background:none; border:none; cursor:pointer; font-weight:600;">"삭제"</button>
                                            </form>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                }
            </div>
            <script>
                {r#"document.addEventListener('DOMContentLoaded',function(){flatpickr('#event_date_picker',{locale:'ko',dateFormat:'Y-m-d',disableMobile:true,animate:true});flatpickr('#event_time_picker',{locale:'ko',enableTime:true,noCalendar:true,time_24hr:true,dateFormat:'H:i',disableMobile:true,animate:true})});"#}
            </script>
        </body></html>
    })
}

#[route(POST "/admin/events/create")]
pub async fn create_event(
    cx: &Cx,
    Form(input): Form<EventForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "events.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.events.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let max = input.max_participants.trim().parse::<i32>().unwrap_or(0);
    let event_date = input.event_date.trim().to_string();
    let event_time = input.event_time.trim().to_string();
    sqlx::query(
        "INSERT INTO events (creator_id, title, description, location, event_date, event_time, max_participants) VALUES ('system', $1, $2, $3, $4, $5, $6)"
    ).bind(&input.title).bind(&input.description).bind(&input.location).bind(&event_date).bind(&event_time).bind(max)
    .execute(db).await.ok();
    Ok(topcoat::router::error::see_other("/admin/events"))
}

#[route(POST "/admin/events/{id}/delete")]
pub async fn delete_event(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.events.x.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/events"))
}

#[page("/admin/events/edit")]
pub async fn admin_event_edit_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "events.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let query = uri(cx).query().unwrap_or("");
    let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let id = params.get("id").cloned().unwrap_or_default();

    let ev = match sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(&id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
    {
        Some(ev) => ev,
        None => return Err(topcoat::router::error::redirect("/admin/events").into()),
    };

    let input_style = "width:100%; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;";
    let label_style =
        "display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;";

    let flatpickr_css = "https://cdn.jsdelivr.net/npm/flatpickr/dist/flatpickr.min.css";
    let flatpickr_ko = "https://cdn.jsdelivr.net/npm/flatpickr/dist/l10n/ko.js";
    let flatpickr_js = "https://cdn.jsdelivr.net/npm/flatpickr";

    Ok(view! {
        <html lang="ko"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>"이벤트 수정 - Admin"</title>
            <link rel="stylesheet" href=(flatpickr_css) />
            <style>
                {r#"
                .flatpickr-calendar { border-radius: 10px !important; border: 1px solid #2a2d3a !important; background: #1a1d27 !important; box-shadow: 0 8px 24px rgba(0,0,0,0.4) !important; font-family: inherit !important; }
                .flatpickr-months { background: #1a1d27 !important; border-bottom: 1px solid #2a2d3a !important; }
                .flatpickr-current-month { color: #e5e7eb !important; background: #1a1d27 !important; }
                .flatpickr-current-month .flatpickr-monthDropdown-months { background: #1a1d27 !important; color: #e5e7eb !important; border: 1px solid #2a2d3a !important; border-radius: 4px !important; }
                span.flatpickr-weekday { color: #9ca3af !important; background: #1a1d27 !important; }
                .flatpickr-day { color: #e5e7eb !important; border-radius: 6px !important; border-color: transparent !important; }
                .flatpickr-day:hover { background: rgba(222,91,48,0.2) !important; border-color: rgba(222,91,48,0.3) !important; color: #f9fafb !important; }
                .flatpickr-day.selected { background: #DE5B30 !important; border-color: #DE5B30 !important; color: #fff !important; }
                .flatpickr-day.prevMonthDay, .flatpickr-day.nextMonthDay { color: #4b5563 !important; }
                .flatpickr-day.today { border-color: #DE5B30 !important; }
                .flatpickr-day.today:hover { background: rgba(222,91,48,0.2) !important; }
                .flatpickr-prev-month, .flatpickr-next-month { fill: #9ca3af !important; }
                .flatpickr-prev-month:hover, .flatpickr-next-month:hover { fill: #DE5B30 !important; }
                .flatpickr-time { border-top: 1px solid #2a2d3a !important; background: #1a1d27 !important; }
                .flatpickr-time input { color: #e5e7eb !important; background: #1a1d27 !important; text-shadow: 0 0 1px rgba(229,231,235,0.3) !important; }
                .flatpickr-time .flatpickr-am-pm { color: #9ca3af !important; }
                .flatpickr-time .flatpickr-am-pm:hover { color: #DE5B30 !important; }
                span.numInputWrapper { background: transparent !important; }
                span.numInputWrapper span.arrowUp, span.numInputWrapper span.arrowDown { border-color: #4b5563 !important; }
                span.numInputWrapper span.arrowUp:hover, span.numInputWrapper span.arrowDown:hover { border-color: #DE5B30 !important; background: rgba(222,91,48,0.1) !important; }
                span.numInputWrapper span.arrowUp::after, span.numInputWrapper span.arrowDown::after { border-color: #9ca3af !important; }
                span.numInputWrapper:hover { background: transparent !important; }
                .flatpickr-months .flatpickr-next-month flatpickr-monthDropdown-months { background: #1a1d27 !important; }
                input.flatpickr-input { background: #0f1117 !important; color: #e5e7eb !important; border: 1px solid #2a2d3a !important; border-radius: 8px !important; padding: 10px 14px !important; }
                "#}
            </style>
            <style>{(ADMIN_STYLE)}</style>
            <script src=(flatpickr_js)></script>
            <script src=(flatpickr_ko)></script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/events".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700;">"이벤트 수정"</h1>
                </div>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; margin-bottom:24px;">
                    <form action="/admin/events/update" method="POST" style="display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="hidden" name="id" value=(ev.id.clone())>
                        <div style="grid-column:1 / -1;"><label style=(label_style)>"제목"</label><input type="text" name="title" required="true" value=(ev.title.clone()) style=(input_style)></div>
                        <div style="grid-column:1 / -1;"><label style=(label_style)>"설명"</label><textarea name="description" rows="3" style=(input_style)>{(ev.description.clone())}</textarea></div>
                        <div><label style=(label_style)>"장소"</label><input type="text" name="location" value=(ev.location.clone()) style=(input_style)></div>
                        <div><label style=(label_style)>"최대 인원 (0=무제한)"</label><input type="number" name="max_participants" value=(ev.max_participants.to_string()) min="0" style=(input_style)></div>
                        <div><label style=(label_style)>"날짜"</label><input type="text" id="event_date_picker_edit" name="event_date" required="true" placeholder="날짜 선택" readonly="true" value=(ev.event_date.clone()) style=(input_style)></div>
                        <div><label style=(label_style)>"시간"</label><input type="text" id="event_time_picker_edit" name="event_time" placeholder="시간 선택" readonly="true" value=(ev.event_time.clone()) style=(input_style)></div>
                        <div style="grid-column:1 / -1;"><button type="submit" style="background:#DE5B30; color:#fff; padding:10px 24px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">"이벤트 저장"</button></div>
                    </form>
                </div>
            </div>
            <script>
                {r#"document.addEventListener('DOMContentLoaded',function(){flatpickr('#event_date_picker_edit',{locale:'ko',dateFormat:'Y-m-d',disableMobile:true,animate:true});flatpickr('#event_time_picker_edit',{locale:'ko',enableTime:true,noCalendar:true,time_24hr:true,dateFormat:'H:i',disableMobile:true,animate:true})});"#}
            </script>
        </body></html>
    })
}

#[route(POST "/admin/events/update")]
pub async fn update_event(
    cx: &Cx,
    Form(input): Form<EventUpdateForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "events.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.events.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let max = input.max_participants.trim().parse::<i32>().unwrap_or(0);
    let event_date = input.event_date.trim().to_string();
    let event_time = input.event_time.trim().to_string();
    sqlx::query(
        "UPDATE events SET title=$1, description=$2, location=$3, event_date=$4, event_time=$5, max_participants=$6 WHERE id=$7"
    ).bind(&input.title).bind(&input.description).bind(&input.location).bind(&event_date).bind(&event_time).bind(max).bind(&input.id)
    .execute(db).await.ok();
    Ok(topcoat::router::error::see_other("/admin/events"))
}
