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
use crate::models::ManualSeries;
use crate::util::query_param;

#[derive(serde::Deserialize)]
pub struct EditManual {
    pub csrf_token: String,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub estimated_minutes: i32,
    pub visibility: String,
    #[serde(default)]
    pub published_at: String,
}

#[route(POST "/admin/manuals/update")]
pub async fn update_manual(
    cx: &Cx,
    Form(input): Form<EditManual>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.manuals.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    if !crate::auth::access::valid_visibility(&input.visibility) {
        return Ok(topcoat::router::error::see_other("/admin/manuals"));
    }

    if !crate::models::difficulty::valid(&input.difficulty) {
        return Ok(topcoat::router::error::see_other("/admin/manuals"));
    }

    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => {
            return Ok(topcoat::router::error::see_other("/admin/manuals"));
        }
    };

    sqlx::query(
        "UPDATE manual_series SET slug = $1, title = $2, description = $3, difficulty = $4, estimated_minutes = $5, visibility = $6, published_at = $7, updated_at = NOW()::text
         WHERE id = $8"
    )
    .bind(&input.slug)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.difficulty)
    .bind(input.estimated_minutes)
    .bind(&input.visibility)
    .bind(&published_at)
    .bind(&input.id)
    .execute(db)
    .await.ok();
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.manual.update",
        "manual_series",
        &input.id,
        "",
    )
    .await;

    Ok(topcoat::router::error::see_other("/admin/manuals"))
}

#[page("/admin/manuals/edit")]
pub async fn edit_manual(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::MANUALS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let id = query_param(cx, "id").unwrap_or_default();
    if id.is_empty() {
        return Err(topcoat::router::error::not_found().into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let series = sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE id = $1")
        .bind(&id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    match series {
        Some(s) => {
            let diff_beginner = s.difficulty == crate::models::difficulty::BEGINNER;
            let diff_intermediate = s.difficulty == crate::models::difficulty::INTERMEDIATE;
            let diff_advanced = s.difficulty == crate::models::difficulty::ADVANCED;
            let vis_public = s.visibility == "public";
            let vis_members = s.visibility == "members";
            let vis_staff = s.visibility == "staff";
            let vis_premium = s.visibility == "premium";

            Ok(view! {
                <html lang="ko">
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>"매뉴얼 수정 - Rust Korea Admin"</title>
                    <style>{(ADMIN_STYLE)}</style>
                    <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
                </head>
                <body style="display:flex; min-height:100vh;">
                    {AdminSidebar(active: "/admin/manuals".into())}
                    <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                        <div style="margin-bottom:24px;">
                            <a href="/admin/manuals" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                                "매뉴얼 목록"
                            </a>
                            <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"매뉴얼 수정"</h1>
                        </div>

                        <form action="/admin/manuals/update" method="POST" style="max-width:640px;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:16px;">
                                <input type="hidden" name="id" value=(s.id.clone())>
                                <div style="margin-bottom:20px;">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"URL 슬러그"</label>
                                    <input type="text" name="slug" required="true" class="input" style="font-family:monospace;" value=(s.slug.clone())>
                                    <p style="font-size:11px; color:var(--dim); margin-top:6px;">"영문, 하이픈 사용. 예: topcoat-getting-started"</p>
                                    <p style="font-size:11px; color:var(--amber); margin-top:4px;">"주의: 슬러그를 바꾸면 기존 공유 링크가 깨집니다."</p>
                                </div>
                                <div style="margin-bottom:20px;">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"제목"</label>
                                    <input type="text" name="title" required="true" class="input" value=(s.title.clone())>
                                </div>
                                <div style="margin-bottom:20px;">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"설명"</label>
                                    <textarea name="description" rows="3" class="input">(s.description.clone())</textarea>
                                </div>
                                <div style="display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                                    <div>
                                        <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"난이도"</label>
                                        <select name="difficulty" class="input">
                                            <option value="beginner" selected=(diff_beginner)>"초급"</option>
                                            <option value="intermediate" selected=(diff_intermediate)>"중급"</option>
                                            <option value="advanced" selected=(diff_advanced)>"고급"</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"예상 소요시간 (분)"</label>
                                        <input type="number" name="estimated_minutes" min="0" class="input" value=(s.estimated_minutes.to_string())>
                                    </div>
                                </div>
                                <div style="margin-top:16px;">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"공개 범위"</label>
                                    <select name="visibility" class="input">
                                        <option value="public" selected=(vis_public)>"전체공개 (비회원 포함)"</option>
                                        <option value="members" selected=(vis_members)>"회원전용 (로그인 필요)"</option>
                                        <option value="staff" selected=(vis_staff)>"스태프전용"</option>
                                        <option value="premium" selected=(vis_premium)>"유료전용(준비중)"</option>
                                    </select>
                                </div>
                                <div style="margin-top:16px;">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"예약 발행 (비우면 즉시 발행)"</label>
                                    <input type="datetime-local" name="published_at" class="input" value=(crate::util::format_schedule_kst(s.published_at.as_deref()))>
                                    <p style="font-size:11px; color:var(--dim); margin-top:4px;">"한국 시간 기준. 비우면 예약이 해제됩니다."</p>
                                </div>
                            </div>
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; width:100%; justify-content:center; border:none; cursor:pointer;">"변경 사항 저장"</button>
                        </form>
                    </div>
                </body>
                </html>
            })
        }
        None => Err(topcoat::router::error::redirect("/admin/manuals").into()),
    }
}
