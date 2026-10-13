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
    pub visibility: String,
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

    if !crate::auth::access::valid_visibility(&input.visibility) {
        return Ok(topcoat::router::error::see_other("/admin/manuals"));
    }

    sqlx::query(
        "UPDATE manual_chapters SET slug = $1, title = $2, content_md = $3, content_html = $4, visibility = $5, updated_at = NOW()::text
         WHERE id = $6"
    )
    .bind(&input.slug)
    .bind(&input.title)
    .bind(&input.content_md)
    .bind(&content_html)
    .bind(&input.visibility)
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
                <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
                <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
            </head>
            <body style="display:flex; min-height:100vh;">
                {AdminSidebar(active: "/admin/manuals".into())}
                <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                    <div style="margin-bottom:24px;">
                        <a href=(format!("/admin/manuals/{}", s.slug)) style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                            "시리즈로 돌아가기"
                        </a>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"챕터 수정"</h1>
                        <p style="font-size:13px; color:var(--muted2); margin-top:2px;">(format!("시리즈: {}", s.title.clone()))</p>
                    </div>

                    <form action="/admin/manuals/chapters/update" method="POST" style="max-width:720px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:16px;">
                            <input type="hidden" name="id" value=(c.id.clone())>
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"챕터 슬러그"</label>
                                <input type="text" name="slug" required="true" class="input" style="font-family:monospace;" value=(c.slug.clone())>
                                <p style="font-size:11px; color:var(--amber); margin-top:4px;">"주의: 슬러그를 바꾸면 기존 공유 링크가 깨집니다."</p>
                            </div>
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"챕터 제목"</label>
                                <input type="text" name="title" required="true" class="input" value=(c.title.clone())>
                            </div>
                            <div style="margin-bottom:20px;">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"공개 범위 (챕터별)"</label>
                                <select name="visibility" class="input">
                                    <option value="public" selected=(c.visibility == "public")>"전체공개 (비회원 포함)"</option>
                                    <option value="members" selected=(c.visibility == "members")>"회원전용 (로그인 필요)"</option>
                                    <option value="staff" selected=(c.visibility == "staff")>"스태프전용"</option>
                                    <option value="premium" selected=(c.visibility == "premium")>"유료전용(준비중)"</option>
                                </select>
                            </div>
                            <div>
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"내용 (Markdown)"</label>
                                <div style="margin-bottom:12px; display:flex; align-items:center; gap:10px;">
                                <label for="md-file" style="font-size:12px; color:var(--muted); cursor:pointer; border:1px dashed var(--border); border-radius:6px; padding:8px 14px;">".md 파일 불러오기"</label>
                                <input type="file" id="md-file" accept=".md,.markdown,.txt" style="display:none;">
                                <span style="font-size:11px; color:var(--dim);">"또는 :::youtube[영상ID] 로 영상 삽입"</span>
                                </div>
                                <textarea id="editor" name="content_md" rows="20" required="true" class="input" style="font-family:'SF Mono','Fira Code',monospace; font-size:13px; line-height:1.7;">(c.content_md.clone())</textarea>
                            </div>
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; width:100%; justify-content:center; border:none; cursor:pointer;">"변경 사항 저장"</button>
                    </form>
                    <script>
                        {(crate::services::markdown::EASYMDE_INIT_JS)}
                        {(crate::services::markdown::MD_UPLOAD_JS)}
                        "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}bindMdUpload('md-file',easymde);"
                    </script>
                </div>
            </body>
            </html>
        }),
        _ => Err(topcoat::router::error::redirect("/admin/manuals").into()),
    }
}

#[derive(serde::Deserialize)]
pub struct MoveChapterForm {
    pub csrf_token: String,
    pub id: String,
    pub direction: String,
}

/// Swap a chapter with its neighbor (up/down) inside the same series.
#[route(POST "/admin/manuals/chapters/move")]
pub async fn move_chapter(
    cx: &Cx,
    Form(input): Form<MoveChapterForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.manuals.chapters.move",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::MANUALS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let current: Option<ManualChapter> =
        sqlx::query_as::<_, ManualChapter>("SELECT * FROM manual_chapters WHERE id = $1")
            .bind(&input.id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
    let Some(current) = current else {
        return Ok(topcoat::router::error::see_other("/admin/manuals"));
    };

    let neighbor: Option<ManualChapter> = if input.direction == "up" {
        sqlx::query_as::<_, ManualChapter>(
            "SELECT * FROM manual_chapters WHERE series_id = $1 AND chapter_index < $2
             ORDER BY chapter_index DESC LIMIT 1",
        )
        .bind(&current.series_id)
        .bind(current.chapter_index)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
    } else {
        sqlx::query_as::<_, ManualChapter>(
            "SELECT * FROM manual_chapters WHERE series_id = $1 AND chapter_index > $2
             ORDER BY chapter_index ASC LIMIT 1",
        )
        .bind(&current.series_id)
        .bind(current.chapter_index)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
    };

    if let Some(other) = neighbor {
        let mut tx = match db.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                eprintln!("[admin] move chapter tx begin failed: {e}");
                return Ok(topcoat::router::error::see_other("/admin/manuals?error=1"));
            }
        };
        // Swap via a temporary out-of-range value to avoid transient ties.
        let tmp_index = -1;
        let ok = sqlx::query("UPDATE manual_chapters SET chapter_index = $1 WHERE id = $2")
            .bind(tmp_index)
            .bind(&current.id)
            .execute(&mut *tx)
            .await
            .is_ok()
            && sqlx::query("UPDATE manual_chapters SET chapter_index = $1 WHERE id = $2")
                .bind(current.chapter_index)
                .bind(&other.id)
                .execute(&mut *tx)
                .await
                .is_ok()
            && sqlx::query("UPDATE manual_chapters SET chapter_index = $1 WHERE id = $2")
                .bind(other.chapter_index)
                .bind(&current.id)
                .execute(&mut *tx)
                .await
                .is_ok();
        if ok && tx.commit().await.is_ok() {
            let actor = crate::auth::access::load_access(db, cx)
                .await
                .map(|a| a.user.id)
                .unwrap_or_default();
            crate::auth::access::audit(
                db,
                cx,
                &actor,
                "admin.manuals.chapters.move",
                "manual_chapters",
                &current.id,
                &input.direction,
            )
            .await;
        } else {
            eprintln!("[admin] move chapter tx failed: {}", current.id);
        }
    }

    Ok(topcoat::router::error::see_other("/admin/manuals"))
}
