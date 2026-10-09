use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::ManualSeries;
use crate::services::markdown::render_markdown;

topcoat::router::path_param!(series_slug);

#[derive(serde::Deserialize)]
pub struct NewChapter {
    pub csrf_token: String,
    pub slug: String,
    pub title: String,
    pub content_md: String,
}

#[route(POST "/admin/manuals/{series_slug}/chapters/create")]
pub async fn create_chapter(
    cx: &Cx,
    Form(input): Form<NewChapter>,
) -> Result<topcoat::router::error::SeeOther> {
    let series_slug = path_param::<SeriesSlug>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.manuals.x.chapters.create",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    if let Some(s) =
        sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE slug = $1")
            .bind(series_slug)
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
    {
        let content_html = render_markdown(&input.content_md);
        let max_index: (i32,) = sqlx::query_as(
            "SELECT COALESCE(MAX(chapter_index), -1) FROM manual_chapters WHERE series_id = $1",
        )
        .bind(&s.id)
        .fetch_one(db)
        .await
        .unwrap_or((-1,));

        sqlx::query(
            "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, chapter_index)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&s.id)
        .bind(&input.slug)
        .bind(&input.title)
        .bind(&input.content_md)
        .bind(&content_html)
        .bind(max_index.0 + 1)
        .execute(db)
        .await.ok();
    }

    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.chapter.create",
        "manual_chapters",
        "",
        "",
    )
    .await;

    Ok(topcoat::router::error::see_other("/admin/manuals"))
}

#[page("/admin/manuals/{series_slug}/chapters/new")]
pub async fn new_chapter(cx: &Cx) -> Result<impl View + '_> {
    let series_slug = path_param::<SeriesSlug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "manuals.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let series = sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE slug = $1")
        .bind(series_slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    match series {
        Some(s) => Ok(view! {
            <html lang="ko">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>"새 챕터 - Rust Korea Admin"</title>
                <link rel="stylesheet" href=(crate::admin_ui::EASYMDE_CSS_URL)>
                <script src=(crate::admin_ui::EASYMDE_JS_URL)></script>
                <style>{(ADMIN_STYLE)}</style>
            </head>
            <body style="display:flex; min-height:100vh;">
                {AdminSidebar(active: "/admin/chapters/new".into())}
                <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                    <div style="margin-bottom:24px;">
                        <a href="/admin/chapters/new" style="font-size:12px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                            "시리즈 선택"
                        </a>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"새 챕터 추가"</h1>
                        <p style="font-size:13px; color:#6b7280; margin-top:2px;">(format!("시리즈: {}", s.title.clone()))</p>
                    </div>

                    <form action=(format!("/admin/manuals/{}/chapters/create", series_slug)) method="POST" style="max-width:720px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; margin-bottom:16px;">
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:#9ca3af; margin-bottom:8px;">"챕터 슬러그"</label>
                                <input type="text" name="slug" required="true" placeholder="01-introduction" class="input" style="font-family:monospace;">
                            </div>
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:#9ca3af; margin-bottom:8px;">"챕터 제목"</label>
                                <input type="text" name="title" required="true" placeholder="1장: Introduction" class="input">
                            </div>
                            <div>
                                <label style="display:block; font-size:13px; font-weight:600; color:#9ca3af; margin-bottom:8px;">"내용 (Markdown)"</label>
                                <textarea id="editor" name="content_md" rows="20" required="true" class="input" style="font-family:'SF Mono','Fira Code',monospace; font-size:13px; line-height:1.7;"></textarea>
                            </div>
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; width:100%; justify-content:center; border:none; cursor:pointer;">"챕터 저장"</button>
                    </form>
                    <script>
                        {r#"new EasyMDE({element:document.getElementById('editor'),minHeight:'300px',status:false,toolbar:['bold','italic','heading','|','quote','unordered-list','ordered-list','|','link','image','code','preview','side-by-side','fullscreen','guide']})"#}
                    </script>
                </div>
            </body>
            </html>
        }),
        None => Err(topcoat::router::error::redirect("/admin/manuals").into()),
    }
}
