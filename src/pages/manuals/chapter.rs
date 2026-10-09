use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::components::ad_unit::AdUnit;
use crate::db::Db;
use crate::models::ManualChapter;

topcoat::router::path_param!(series_slug);

topcoat::router::path_param!(chapter_slug);

#[page("/manuals/{series_slug}/{chapter_slug}")]
pub async fn chapter_detail(cx: &Cx) -> Result<impl View + '_> {
    let series_slug = path_param::<SeriesSlug>(cx);
    let chapter_slug = path_param::<ChapterSlug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let access = crate::auth::access::load_access(db, cx).await;

    let series_visibility: Option<String> =
        sqlx::query_scalar("SELECT visibility FROM manual_series WHERE slug = $1")
            .bind(series_slug)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();

    let gate = series_visibility
        .as_deref()
        .map(|v| {
            crate::auth::access::visibility_gate(
                access.as_ref(),
                v,
                "manuals.view_staff",
                "premium.access",
            )
        })
        .unwrap_or(crate::auth::access::Gate::Allow);
    if gate == crate::auth::access::Gate::LoginRequired {
        return Err(topcoat::router::error::redirect(format!(
            "/auth/login?redirect=/manuals/{series_slug}/{chapter_slug}"
        ))
        .into());
    }
    let forbidden = gate == crate::auth::access::Gate::Forbidden;
    let forbidden_msg: String = series_visibility
        .as_deref()
        .map(|v| match v {
            "members" => "이 매뉴얼은 회원 전용입니다.",
            "staff" => "이 매뉴얼은 스태프 전용입니다.",
            _ => "이 매뉴얼은 유료 전용입니다(준비중).",
        })
        .unwrap_or_default()
        .to_string();

    let chapter = if forbidden {
        None
    } else {
        sqlx::query_as::<_, ManualChapter>(
            r#"SELECT mc.* FROM manual_chapters mc
            JOIN manual_series ms ON mc.series_id = ms.id
            WHERE ms.slug = $1 AND mc.slug = $2"#,
        )
        .bind(series_slug)
        .bind(chapter_slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
    };

    let current_user = crate::auth::session::current_user(db, cx).await;
    let completed: bool = if let (Some(u), Some(ch)) = (&current_user, &chapter) {
        sqlx::query_scalar::<_, i32>(
            "SELECT completed FROM reading_progress WHERE user_id = $1 AND chapter_id = $2",
        )
        .bind(&u.id)
        .bind(&ch.id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .map(|c| c == 1)
        .unwrap_or(false)
    } else {
        false
    };

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:780px;">
            if let Some(ch) = chapter {
                <a href=(format!("/manuals/{}", series_slug)) style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">
                    "<- 시리즈 목록"
                </a>

                <h1 style="font-size:1.75rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:32px;">(ch.title.clone())</h1>

                <div class="card" style="padding:40px 48px;">
                    <div class="prose prose-headings:text-zinc-900 prose-p:text-zinc-600 prose-p:leading-relaxed prose-a:text-[#DE5B30] prose-a:no-underline hover:prose-a:underline prose-code:text-[#DE5B30] prose-code:bg-zinc-100 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm prose-code:font-normal prose-pre:bg-zinc-50 prose-pre:border prose-pre:border-zinc-200 prose-pre:rounded-lg max-w-none" style="font-size:14px; line-height:1.8;">
                        (topcoat::view::Unescaped::new_unchecked(ch.content_html.clone()))
                    </div>
                </div>

                {AdUnit(slot_id: "chapter_bottom".into())}

                <div style="margin-top:32px; display:flex; align-items:center; gap:12px; flex-wrap:wrap;">
                    if current_user.is_some() {
                        if completed {
                            <span style="display:inline-flex; align-items:center; gap:6px; font-size:13px; font-weight:600; color:#059669; background:#ecfdf5; border:1px solid #a7f3d0; padding:8px 16px; border-radius:8px;">"✓ 읽음 완료"</span>
                        } else {
                            <form action=(format!("/manuals/{series_slug}/{chapter_slug}/complete")) method="POST" style="display:inline;">
                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                <button type="submit" class="btn-primary">"읽음 완료로 표시"</button>
                            </form>
                        }
                    } else {
                        <a href=(format!("/auth/login?redirect=/manuals/{series_slug}/{chapter_slug}")) class="btn-secondary">"로그인하고 진행률 저장"</a>
                    }
                    <a href=(format!("/manuals/{}", series_slug)) style="font-size:13px; color:#6b7280; transition:color 0.15s;">
                        "<- 시리즈 목록으로 돌아가기"
                    </a>
                </div>
            } else {
                <div style="text-align:center; padding:100px 0;">
                    if forbidden {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"접근 권한이 없습니다"</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">(forbidden_msg.clone())</p>
                    } else {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"챕터를 찾을 수 없습니다"</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">"요청하신 챕터가 존재하지 않습니다."</p>
                    }
                    <a href="/manuals" class="btn-primary" style="display:inline-flex;">"매뉴얼 목록으로"</a>
                </div>
            }
        </div>
    })
}

#[route(POST "/manuals/{series_slug}/{chapter_slug}/complete")]
pub async fn complete_chapter(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let series_slug = path_param::<SeriesSlug>(cx);
    let chapter_slug = path_param::<ChapterSlug>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    let back = format!("/manuals/{series_slug}/{chapter_slug}");
    let Some(user) = crate::auth::session::current_user(db, cx).await else {
        return Ok(topcoat::router::error::see_other(format!(
            "/auth/login?redirect={back}"
        )));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "manuals.complete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let chapter_id: Option<String> = sqlx::query_scalar(
        "SELECT mc.id FROM manual_chapters mc
         JOIN manual_series ms ON mc.series_id = ms.id
         WHERE ms.slug = $1 AND mc.slug = $2",
    )
    .bind(series_slug)
    .bind(chapter_slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    if let Some(cid) = chapter_id {
        sqlx::query(
            "INSERT INTO reading_progress (user_id, chapter_id, scroll_pct, completed)
             VALUES ($1, $2, 100, 1)
             ON CONFLICT (user_id, chapter_id) DO UPDATE SET completed = 1, scroll_pct = 100, updated_at = NOW()::text",
        )
        .bind(&user.id)
        .bind(&cid)
        .execute(db)
        .await
        .ok();
    }
    Ok(topcoat::router::error::see_other(&back))
}
