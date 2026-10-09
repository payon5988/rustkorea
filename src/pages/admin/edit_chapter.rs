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
use crate::models::{ManualChapter, ManualSeries};
use crate::services::markdown::render_markdown;
use crate::util::query_param;

#[derive(serde::Deserialize)]
pub struct EditChapter {
    pub csrf_token: String,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub content_md: String,
}

#[route(POST "/admin/manuals/chapters/update")]
pub async fn update_chapter(
    cx: &Cx,
    Form(input): Form<EditChapter>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.manuals.chapters.update",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let content_html = render_markdown(&input.content_md);

    sqlx::query(
        "UPDATE manual_chapters SET slug = $1, title = $2, content_md = $3, content_html = $4, updated_at = NOW()::text
         WHERE id = $5"
    )
    .bind(&input.slug)
    .bind(&input.title)
    .bind(&input.content_md)
    .bind(&content_html)
    .bind(&input.id)
    .execute(db)
    .await.ok();

    let series_slug = sqlx::query_scalar::<_, String>(
        "SELECT ms.slug FROM manual_series ms
         JOIN manual_chapters mc ON mc.series_id = ms.id
         WHERE mc.id = $1",
    )
    .bind(&input.id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.chapter.update",
        "manual_chapters",
        &input.id,
        "",
    )
    .await;
    match series_slug {
        Some(slug) => Ok(topcoat::router::error::see_other(format!(
            "/admin/manuals/{}",
            slug
        ))),
        None => Ok(topcoat::router::error::see_other("/admin/manuals")),
    }
}

#[page("/admin/manuals/chapters/edit")]
pub async fn edit_chapter(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "manuals.manage")
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

    let chapter = sqlx::query_as::<_, ManualChapter>("SELECT * FROM manual_chapters WHERE id = $1")
        .bind(&id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let series = match &chapter {
        Some(c) => sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE id = $1")
            .bind(&c.series_id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten(),
        None => None,
    };

    match (chapter, series) {
        (Some(c), Some(s)) => Ok(view! {
            <html lang="ko">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>"챕터 수정 - Rust Korea Admin"</title>
                <link rel="stylesheet" href=(crate::admin_ui::EASYMDE_CSS_URL)>
                <script src=(crate::admin_ui::EASYMDE_JS_URL)></script>
                <style>{(ADMIN_STYLE)}</style>
            </head>
            <body style="display:flex; min-height:100vh;">
                {AdminSidebar(active: "/admin/manuals".into())}
                <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                    <div style="margin-bottom:24px;">
                        <a href=(format!("/admin/manuals/{}", s.slug)) style="font-size:12px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                            "시리즈로 돌아가기"
                        </a>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"챕터 수정"</h1>
                        <p style="font-size:13px; color:#6b7280; margin-top:2px;">(format!("시리즈: {}", s.title.clone()))</p>
                    </div>

                    <form action="/admin/manuals/chapters/update" method="POST" style="max-width:720px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; margin-bottom:16px;">
                            <input type="hidden" name="id" value=(c.id.clone())>
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:#9ca3af; margin-bottom:8px;">"챕터 슬러그"</label>
                                <input type="text" name="slug" required="true" class="input" style="font-family:monospace;" value=(c.slug.clone())>
                            </div>
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:#9ca3af; margin-bottom:8px;">"챕터 제목"</label>
                                <input type="text" name="title" required="true" class="input" value=(c.title.clone())>
                            </div>
                            <div>
                                <label style="display:block; font-size:13px; font-weight:600; color:#9ca3af; margin-bottom:8px;">"내용 (Markdown)"</label>
                                <textarea id="editor" name="content_md" rows="20" required="true" class="input" style="font-family:'SF Mono','Fira Code',monospace; font-size:13px; line-height:1.7;">(c.content_md.clone())</textarea>
                            </div>
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; width:100%; justify-content:center; border:none; cursor:pointer;">"변경 사항 저장"</button>
                    </form>
                    <script>
                        {r#"new EasyMDE({element:document.getElementById('editor'),minHeight:'300px',status:false,toolbar:['bold','italic','heading','|','quote','unordered-list','ordered-list','|','link','image','code','preview','side-by-side','fullscreen','guide']})"#}
                    </script>
                </div>
            </body>
            </html>
        }),
        _ => Err(topcoat::router::error::redirect("/admin/manuals").into()),
    }
}
