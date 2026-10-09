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
                <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
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
                                <div style="margin-bottom:12px; display:flex; align-items:center; gap:10px;">
                                <label for="md-file" style="font-size:12px; color:#9ca3af; cursor:pointer; border:1px dashed #2a2d3a; border-radius:6px; padding:8px 14px;">".md 파일 불러오기"</label>
                                <input type="file" id="md-file" accept=".md,.markdown,.txt" style="display:none;">
                                <span style="font-size:11px; color:#4b5563;">"또는 :::youtube[영상ID] 로 영상 삽입"</span>
                                </div>
                                <textarea id="editor" name="content_md" rows="20" required="true" class="input" style="font-family:'SF Mono','Fira Code',monospace; font-size:13px; line-height:1.7;"></textarea>
                            </div>
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; width:100%; justify-content:center; border:none; cursor:pointer;">"챕터 저장"</button>
                    </form>
                    <form action="/admin/manuals/pdf-import" method="POST" enctype="multipart/form-data" style="max-width:720px; margin-top:16px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="hidden" name="series_slug" value=(s.slug.clone())>
                        <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px;">
                            <h2 style="font-size:15px; font-weight:700; color:#f9fafb; margin-bottom:4px;">"PDF 가져오기"</h2>
                            <p style="font-size:12px; color:#6b7280; margin-bottom:16px;">"PDF 텍스트를 추출해 초안 챕터로 만듭니다 (최대 2MB, 표·이미지는 수동 보정 필요)."</p>
                            <div style="display:flex; gap:12px; align-items:center;">
                                <input type="file" name="file" required="true" accept=".pdf,application/pdf" style="flex:1; font-size:13px; color:#9ca3af;">
                                <button type="submit" style="background:#1a1d27; color:#e5e7eb; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:1px solid #2a2d3a; cursor:pointer; white-space:nowrap;">"PDF에서 만들기"</button>
                            </div>
                        </div>
                    </form>
                    <script>
                        {(crate::services::markdown::EASYMDE_INIT_JS)}
                        {(crate::services::markdown::MD_UPLOAD_JS)}
                        "var easymde=initEasyMDE('editor');bindMdUpload('md-file',easymde);"
                    </script>
                </div>
            </body>
            </html>
        }),
        None => Err(topcoat::router::error::redirect("/admin/manuals").into()),
    }
}

/// PDF upload → draft chapter (manuals.manage only).
/// Body capped at 2MB by the framework; output truncated to 100k chars.
#[route(POST "/admin/manuals/pdf-import")]
pub async fn pdf_import_chapter(
    cx: &Cx,
    mut mp: topcoat::router::content::multipart::Multipart,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    let fail_upload = || topcoat::router::error::see_other("/admin/manuals?error=upload");

    let mut series_slug = String::new();
    let mut filename = String::new();
    let mut pdf_bytes: Vec<u8> = vec![];
    let mut csrf = String::new();
    loop {
        let field = match mp.next_field().await {
            Ok(f) => f,
            Err(_) => return Ok(fail_upload()),
        };
        let Some(field) = field else { break };
        match field.name().unwrap_or("") {
            "csrf_token" => csrf = field.text().await.unwrap_or_default(),
            "series_slug" => series_slug = field.text().await.unwrap_or_default(),
            "file" => {
                filename = field.file_name().unwrap_or("").to_string();
                let ctype = field.content_type().unwrap_or("").to_string();
                if !filename.to_lowercase().ends_with(".pdf") && ctype != "application/pdf" {
                    return Ok(topcoat::router::error::see_other(
                        "/admin/manuals?error=filetype",
                    ));
                }
                match field.bytes().await {
                    Ok(b) => pdf_bytes = b.to_vec(),
                    Err(_) => return Ok(fail_upload()),
                }
            }
            _ => {}
        }
    }

    if !crate::auth::session::verify_csrf_logged(db, cx, &csrf, "admin.manuals.pdf_import").await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let access = crate::auth::access::load_access(db, cx).await;
    let allowed = access
        .as_ref()
        .map(|a| a.can("manuals.manage"))
        .unwrap_or(false);
    if !allowed {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    if series_slug.is_empty() || pdf_bytes.is_empty() {
        return Ok(fail_upload());
    }

    let series = sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE slug = $1")
        .bind(&series_slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    let Some(s) = series else {
        return Ok(fail_upload());
    };

    let raw = tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&pdf_bytes))
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let mut text = String::with_capacity(raw.len().min(100_000));
    let mut blank_run = 0;
    for line in raw.lines() {
        let t = line.trim_end();
        if t.is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                text.push('\n');
            }
            continue;
        }
        blank_run = 0;
        text.push_str(t);
        text.push('\n');
        if text.len() >= 100_000 {
            break;
        }
    }
    let text = text.trim().to_string();
    if text.is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/manuals?error=empty",
        ));
    }

    let stem = filename
        .rsplit('/')
        .next()
        .unwrap_or("pdf-import")
        .trim_end_matches(".pdf")
        .trim_end_matches(".PDF");
    let title: String = stem.chars().take(120).collect();
    let base_slug: String = slugify(stem);
    let max_index: (i32,) = sqlx::query_as(
        "SELECT COALESCE(MAX(chapter_index), -1) FROM manual_chapters WHERE series_id = $1",
    )
    .bind(&s.id)
    .fetch_one(db)
    .await
    .unwrap_or((-1,));
    let mut slug = base_slug.clone();
    let content_html = render_markdown(&text);
    for n in 2..10 {
        let inserted: Option<i32> = sqlx::query_scalar(
            "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, chapter_index)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (series_id, slug) DO NOTHING RETURNING 1",
        )
        .bind(&s.id)
        .bind(&slug)
        .bind(&title)
        .bind(&text)
        .bind(&content_html)
        .bind(max_index.0 + 1)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
        if inserted == Some(1) {
            break;
        }
        slug = format!("{base_slug}-{n}");
    }

    let actor = access.map(|a| a.user.id).unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.manuals.pdf_import",
        "manual_series",
        &s.id,
        &slug,
    )
    .await;
    Ok(topcoat::router::error::see_other(format!(
        "/admin/manuals/{}",
        s.slug
    )))
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if (c == ' ' || c == '_' || c == '-') && !out.ends_with('-') {
            out.push('-');
        }
    }
    let slug = out.trim_matches('-').to_string();
    if slug.is_empty() {
        "pdf-import".to_string()
    } else {
        slug.chars().take(80).collect()
    }
}
