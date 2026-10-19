use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::{Form, multipart::Multipart},
    router::{page, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::BlogPost;
use crate::services::markdown::render_markdown;
use crate::util::query_param;

fn slugify(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn clean_url(raw: &str) -> String {
    let t = raw.trim().chars().take(500).collect::<String>();
    if t.starts_with("https://") || t.starts_with("http://") {
        t
    } else {
        String::new()
    }
}

async fn require_blog(cx: &Cx) -> bool {
    let adb: &crate::db::Db = topcoat::context::app_context(cx);
    crate::auth::access::require_perm(adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
        .await
        .is_none()
}

#[derive(serde::Deserialize)]
pub struct BlogForm {
    pub csrf_token: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub content_md: String,
    #[serde(default)]
    pub content_html_import: String,
    pub thumbnail_url: String,
    pub source_url: String,
    pub tags: String,
    pub visibility: String,
    #[serde(default)]
    pub published_at: String,
}

fn parse_blog_form(
    input: &BlogForm,
) -> Result<(String, String, String, Option<String>), &'static str> {
    let slug = slugify(&input.slug);
    if slug.is_empty() || input.title.trim().is_empty() {
        return Err("empty");
    }
    if !crate::auth::access::valid_visibility(&input.visibility) {
        return Err("visibility");
    }
    let content_md = if input.content_html_import.trim().is_empty() {
        input.content_md.clone()
    } else {
        crate::services::markdown::html_to_markdown(&input.content_html_import)
    };
    if content_md.trim().is_empty() {
        return Err("empty");
    }
    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => return Err("schedule"),
    };
    let description_html = render_markdown(&content_md);
    Ok((slug, content_md, description_html, published_at))
}

#[route(POST "/admin/blog/create")]
pub async fn create_blog_post(
    cx: &Cx,
    Form(input): Form<BlogForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_blog(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.blog.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let (slug, content_md, description_html, published_at) = match parse_blog_form(&input) {
        Ok(v) => v,
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                "/admin/blog/new?error=invalid",
            ));
        }
    };
    sqlx::query(
        "INSERT INTO blog_posts (slug, title, summary, content_md, content_html, thumbnail_url,
         source_url, tags, visibility, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&slug)
    .bind(input.title.trim())
    .bind(input.summary.trim())
    .bind(&content_md)
    .bind(&description_html)
    .bind(clean_url(&input.thumbnail_url))
    .bind(clean_url(&input.source_url))
    .bind(input.tags.trim())
    .bind(&input.visibility)
    .bind(&published_at)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other("/admin/blog"))
}

#[derive(serde::Deserialize)]
pub struct BlogUpdateForm {
    pub csrf_token: String,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub content_md: String,
    #[serde(default)]
    pub content_html_import: String,
    pub thumbnail_url: String,
    pub source_url: String,
    pub tags: String,
    pub visibility: String,
    #[serde(default)]
    pub published_at: String,
}

#[route(POST "/admin/blog/update")]
pub async fn update_blog_post(
    cx: &Cx,
    Form(input): Form<BlogUpdateForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_blog(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.blog.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let slug = slugify(&input.slug);
    if slug.is_empty() || input.title.trim().is_empty() {
        return Ok(topcoat::router::error::see_other("/admin/blog"));
    }
    if !crate::auth::access::valid_visibility(&input.visibility) {
        return Ok(topcoat::router::error::see_other("/admin/blog"));
    }
    let content_md = if input.content_html_import.trim().is_empty() {
        input.content_md.clone()
    } else {
        crate::services::markdown::html_to_markdown(&input.content_html_import)
    };
    if content_md.trim().is_empty() {
        return Ok(topcoat::router::error::see_other("/admin/blog"));
    }
    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => return Ok(topcoat::router::error::see_other("/admin/blog")),
    };
    let description_html = render_markdown(&content_md);
    sqlx::query(
        "UPDATE blog_posts SET slug = $1, title = $2, summary = $3, content_md = $4,
         content_html = $5, thumbnail_url = $6, source_url = $7, tags = $8,
         visibility = $9, published_at = $10, updated_at = NOW()::text WHERE id = $11",
    )
    .bind(&slug)
    .bind(input.title.trim())
    .bind(input.summary.trim())
    .bind(&content_md)
    .bind(&description_html)
    .bind(clean_url(&input.thumbnail_url))
    .bind(clean_url(&input.source_url))
    .bind(input.tags.trim())
    .bind(&input.visibility)
    .bind(&published_at)
    .bind(&input.id)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other(format!(
        "/admin/blog/edit?id={}",
        input.id
    )))
}

#[derive(serde::Deserialize)]
pub struct DeleteBlogForm {
    pub csrf_token: String,
    pub id: String,
}

#[route(POST "/admin/blog/delete")]
pub async fn delete_blog_post(
    cx: &Cx,
    Form(input): Form<DeleteBlogForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_blog(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.blog.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM blog_posts WHERE id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/blog"))
}

/// Thumbnail upload for blog posts (file or external URL).
#[route(POST "/admin/blog/upload")]
pub async fn upload_blog_thumbnail(
    cx: &Cx,
    mut mp: Multipart,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    let fail = |id: &str| topcoat::router::error::see_other(format!("/admin/blog/edit?id={id}"));
    let mut csrf = String::new();
    let mut blog_id = String::new();
    let mut url = String::new();
    let mut filename = String::new();
    let mut ctype = String::new();
    let mut bytes: Vec<u8> = vec![];
    loop {
        let field = match mp.next_field().await {
            Ok(f) => f,
            Err(_) => return Ok(fail("")),
        };
        let Some(field) = field else { break };
        match field.name().unwrap_or("") {
            "csrf_token" => csrf = field.text().await.unwrap_or_default(),
            "blog_id" => blog_id = field.text().await.unwrap_or_default(),
            "url" => url = field.text().await.unwrap_or_default(),
            "file" => {
                filename = field.file_name().unwrap_or("").to_string();
                ctype = field.content_type().unwrap_or("").to_string();
                match field.bytes().await {
                    Ok(b) => bytes = b.to_vec(),
                    Err(_) => return Ok(fail(&blog_id)),
                }
            }
            _ => {}
        }
    }
    if !crate::auth::session::verify_csrf_logged(db, cx, &csrf, "admin.blog.upload").await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    if require_blog(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let stored = if !bytes.is_empty() {
        match crate::services::uploads::store_image(&filename, &ctype, &bytes) {
            Ok(u) => u,
            Err(_) => return Ok(fail(&blog_id)),
        }
    } else {
        let u = clean_url(&url);
        if u.is_empty() {
            return Ok(fail(&blog_id));
        }
        u
    };
    sqlx::query("UPDATE blog_posts SET thumbnail_url = $1 WHERE id = $2")
        .bind(&stored)
        .bind(&blog_id)
        .execute(db)
        .await
        .ok();
    Ok(fail(&blog_id))
}

/// Fetch an external article URL into a draft (source attribution kept).
#[route(POST "/admin/blog/fetch")]
pub async fn fetch_blog_draft(
    cx: &Cx,
    Form(input): Form<FetchBlogForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_blog(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.blog.fetch")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let url = clean_url(&input.url);
    if url.is_empty() {
        return Ok(topcoat::router::error::see_other("/admin/blog?error=url"));
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            crate::config::http_timeout_secs(),
        ))
        .user_agent(crate::config::http_user_agent())
        .build();
    let Ok(client) = client else {
        return Ok(topcoat::router::error::see_other("/admin/blog?error=fetch"));
    };
    let html = match client.get(&url).send().await {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => {
            return Ok(topcoat::router::error::see_other("/admin/blog?error=fetch"));
        }
    };
    if html.len() > 500_000 {
        return Ok(topcoat::router::error::see_other("/admin/blog?error=size"));
    }
    let content_md = crate::services::markdown::html_to_markdown(&html);
    let content_md: String = content_md.chars().take(50_000).collect();
    if content_md.trim().is_empty() {
        return Ok(topcoat::router::error::see_other("/admin/blog?error=empty"));
    }
    let title: String = format!("가져오기: {}", url.chars().take(80).collect::<String>());
    // Far-future schedule = draft inbox (unreleased until edited).
    let draft_at = "2099-01-01 00:00:00+00".to_string();
    let description_html = render_markdown(&content_md);
    let slug_base = slugify(&title);
    let slug = if slug_base.is_empty() {
        format!("fetch-{}", &uuid::Uuid::new_v4().to_string()[..8])
    } else {
        format!(
            "{}-{}",
            slug_base.chars().take(60).collect::<String>(),
            &uuid::Uuid::new_v4().to_string()[..4]
        )
    };
    sqlx::query(
        "INSERT INTO blog_posts (slug, title, summary, content_md, content_html, source_url, visibility, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'public', $7)",
    )
    .bind(&slug)
    .bind(&title)
    .bind("외부 가져오기 초안")
    .bind(&content_md)
    .bind(&description_html)
    .bind(&url)
    .bind(&draft_at)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other("/admin/blog"))
}

#[derive(serde::Deserialize)]
pub struct FetchBlogForm {
    pub csrf_token: String,
    pub url: String,
}

#[page("/admin/blog")]
pub async fn admin_blog(cx: &Cx) -> Result<impl View + '_> {
    if require_blog(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let posts = sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts ORDER BY created_at DESC")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"블로그 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/blog".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700;">"블로그 관리"</h1>
                        <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"Rust 소식·오픈소스 큐레이션"</p>
                    </div>
                    <a href="/admin/blog/new" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600;">"+ 새 글"</a>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px; max-width:760px;">
                    <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"외부 글 가져오기 (초안)"</h2>
                    <p style="font-size:12px; color:var(--muted2); margin-bottom:16px;">"URL의 본문을 변환해 미발행 초안으로 저장합니다. 출처가 함께 기록됩니다."</p>
                    <form action="/admin/blog/fetch" method="POST" style="display:flex; gap:8px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="text" name="url" required="true" placeholder="https://..." style="flex:1; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:13px; color:var(--text2); outline:none;">
                        <button type="submit" style="background:var(--card); color:var(--text2); padding:10px 20px; border-radius:8px; font-size:13px; font-weight:600; border:1px solid var(--border); cursor:pointer; white-space:nowrap;">"가져오기"</button>
                    </form>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                    <table style="width:100%; font-size:13px; border-collapse:collapse;">
                        <thead>
                            <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"제목"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"공개"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"관리"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for p in &posts {
                                <tr style="border-bottom:1px solid var(--line);">
                                    <td style="padding:14px 20px;">
                                        <a href=(format!("/admin/blog/edit?id={}", p.id)) style="font-weight:600; color:var(--text3);">(p.title.clone())</a>
                                        <div style="font-size:11px; color:var(--dim); font-family:monospace; margin-top:2px;">(p.slug.clone())</div>
                                        if !crate::util::is_released(p.published_at.as_deref()) {
                                            <span style="font-size:11px; font-weight:600; color:#b45309;">" 예약됨"</span>
                                        }
                                    </td>
                                    <td style="padding:14px 20px;">
                                        <span class="badge" style="background:rgba(96,165,250,0.1); color:var(--blue);">(crate::auth::access::visibility_label(&p.visibility))</span>
                                    </td>
                                    <td style="padding:14px 20px; text-align:right;">
                                        <div style="display:flex; gap:8px; justify-content:flex-end; align-items:center;">
                                            <a href=(format!("/blog/{}", p.slug)) style="font-size:12px; color:var(--muted2);">"보기"</a>
                                            <a href=(format!("/admin/blog/edit?id={}", p.id)) style="font-size:12px; color:var(--amber);">"수정"</a>
                                            <form action="/admin/blog/delete" method="POST" onsubmit="return confirm('이 글을 삭제하시겠습니까?')" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="id" value=(p.id.clone())>
                                                <button type="submit" style="background:none; border:none; padding:0; font-size:12px; color:var(--red); cursor:pointer; font-family:inherit; font-weight:600;">"삭제"</button>
                                            </form>
                                        </div>
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>
            </div>
        </body>
        </html>
    })
}

#[page("/admin/blog/new")]
pub async fn new_blog_post(cx: &Cx) -> Result<impl View + '_> {
    if require_blog(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"새 블로그 글 - Rust Korea Admin"</title>
            <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL)>
            <script src=(crate::admin_ui::TOAST_JS_URL)></script>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
            <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/blog".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <a href="/admin/blog" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">"← 블로그 목록"</a>
                    <h1 style="font-size:1.25rem; font-weight:700;">"새 블로그 글"</h1>
                </div>
                <form action="/admin/blog/create" method="POST" style="max-width:760px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:16px; display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"슬러그 (URL)"</label>
                            <input type="text" name="slug" required="true" class="input" style="font-family:monospace;">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"제목"</label>
                            <input type="text" name="title" required="true" class="input">
                        </div>
                        <div style="grid-column:1 / -1;">
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"요약"</label>
                            <input type="text" name="summary" class="input">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"태그 (쉼표 구분)"</label>
                            <input type="text" name="tags" class="input" placeholder="rust, oss">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"원문 URL (선택)"</label>
                            <input type="text" name="source_url" class="input" placeholder="https://...">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"공개 범위"</label>
                            <select name="visibility" class="input">
                                <option value="public">"전체공개 (비회원 포함)"</option>
                                <option value="members">"회원전용 (로그인 필요)"</option>
                                <option value="staff">"스태프전용"</option>
                                <option value="premium">"유료전용(준비중)"</option>
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"예약 발행 (비워두면 즉시)"</label>
                            <input type="datetime-local" name="published_at" class="input">
                        </div>
                        <div style="grid-column:1 / -1;">
                            <div style="display:flex; gap:8px; margin-bottom:12px;">
                                <button type="button" id="tabbtn-md" onclick="showEditorTab('md')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--brand); background:rgba(222,91,48,0.12); color:var(--text); cursor:pointer;">"마크다운"</button>
                                <button type="button" id="tabbtn-html" onclick="showEditorTab('html')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--border); background:transparent; color:var(--muted); cursor:pointer;">"HTML 가져오기"</button>
                            </div>
                            <div id="tab-md">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"내용 (Markdown)"</label>
                                <textarea id="editor" name="content_md" rows="18" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;"></textarea>
                            </div>
                            <div id="tab-html" style="display:none;">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"HTML 소스 붙여넣기"</label>
                                <textarea name="content_html_import" rows="18" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;" placeholder="여기에 HTML 소스를 붙여넣으세요"></textarea>
                            </div>
                        </div>
                    </div>
                    <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; width:100%; border:none; cursor:pointer;">"글 저장"</button>
                </form>
                <script>
                    {(crate::services::markdown::EDITOR_TAB_JS)}
                    {(crate::services::markdown::EASYMDE_INIT_JS)}
                    "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}"
                </script>
            </div>
        </body>
        </html>
    })
}

#[page("/admin/blog/edit")]
pub async fn edit_blog_post(cx: &Cx) -> Result<impl View + '_> {
    if require_blog(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let id = query_param(cx, "id").unwrap_or_default();
    if id.is_empty() {
        return Err(topcoat::router::error::redirect("/admin/blog").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let post: Option<BlogPost> =
        sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts WHERE id = $1")
            .bind(&id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();

    match post {
        Some(p) => Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"블로그 수정 - Rust Korea Admin"</title>
            <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL)>
            <script src=(crate::admin_ui::TOAST_JS_URL)></script>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
            <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/blog".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <a href="/admin/blog" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">"← 블로그 목록"</a>
                    <h1 style="font-size:1.25rem; font-weight:700;">"블로그 수정"</h1>
                </div>
                <form action="/admin/blog/update" method="POST" style="max-width:760px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <input type="hidden" name="id" value=(p.id.clone())>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:16px; display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"슬러그 (URL)"</label>
                            <input type="text" name="slug" required="true" class="input" style="font-family:monospace;" value=(p.slug.clone())>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"제목"</label>
                            <input type="text" name="title" required="true" class="input" value=(p.title.clone())>
                        </div>
                        <div style="grid-column:1 / -1;">
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"요약"</label>
                            <input type="text" name="summary" class="input" value=(p.summary.clone())>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"태그 (쉼표 구분)"</label>
                            <input type="text" name="tags" class="input" value=(p.tags.clone())>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"원문 URL"</label>
                            <input type="text" name="source_url" class="input" value=(p.source_url.clone())>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"공개 범위"</label>
                            <select name="visibility" class="input">
                                <option value="public" selected=(p.visibility == "public")>"전체공개 (비회원 포함)"</option>
                                <option value="members" selected=(p.visibility == "members")>"회원전용 (로그인 필요)"</option>
                                <option value="staff" selected=(p.visibility == "staff")>"스태프전용"</option>
                                <option value="premium" selected=(p.visibility == "premium")>"유료전용(준비중)"</option>
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"예약 발행 (비우면 즉시)"</label>
                            <input type="datetime-local" name="published_at" class="input" value=(crate::util::format_schedule_kst(p.published_at.as_deref()))>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"썸네일 URL"</label>
                            <input type="text" name="thumbnail_url" class="input" value=(p.thumbnail_url.clone())>
                            <p style="font-size:12px; color:var(--muted); margin-top:6px;">"프론트 화면에서는 이미지를 자르지 않고 원본 비율로 너비에 자동 맞춤합니다."</p>
                        </div>
                        <div style="grid-column:1 / -1;">
                            <div style="display:flex; gap:8px; margin-bottom:12px;">
                                <button type="button" id="tabbtn-md" onclick="showEditorTab('md')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--brand); background:rgba(222,91,48,0.12); color:var(--text); cursor:pointer;">"마크다운"</button>
                                <button type="button" id="tabbtn-html" onclick="showEditorTab('html')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--border); background:transparent; color:var(--muted); cursor:pointer;">"HTML 가져오기"</button>
                            </div>
                            <div id="tab-md">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"내용 (Markdown)"</label>
                                <textarea id="editor" name="content_md" rows="18" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;">(p.content_md.clone())</textarea>
                            </div>
                            <div id="tab-html" style="display:none;">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"HTML 소스 붙여넣기 (비워두면 기존 유지)"</label>
                                <textarea name="content_html_import" rows="18" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;" placeholder="여기에 HTML 소스를 붙여넣으세요"></textarea>
                            </div>
                        </div>
                    </div>
                    <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; width:100%; border:none; cursor:pointer;">"변경 사항 저장"</button>
                </form>

                <div style="max-width:760px; background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-top:16px;">
                    <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"썸네일 업로드 (5MB 이하)"</h2>
                    <form action="/admin/blog/upload" method="POST" enctype="multipart/form-data" style="display:flex; gap:8px; margin-top:12px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="hidden" name="blog_id" value=(p.id.clone())>
                        <input type="file" name="file" accept=".png,.jpg,.jpeg,.webp" style="flex:1; font-size:13px; color:var(--muted);">
                        <input type="text" name="url" placeholder="또는 이미지 URL" style="flex:1; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:13px; color:var(--text2); outline:none;">
                        <button type="submit" style="background:var(--card); color:var(--text2); padding:10px 20px; border-radius:8px; font-size:13px; font-weight:600; border:1px solid var(--border); cursor:pointer; white-space:nowrap;">"업로드"</button>
                    </form>
                </div>
                <script>
                    {(crate::services::markdown::EDITOR_TAB_JS)}
                    {(crate::services::markdown::EASYMDE_INIT_JS)}
                    "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}"
                </script>
            </div>
        </body>
        </html>
        }),
        _ => Err(topcoat::router::error::redirect("/admin/blog").into()),
    }
}
