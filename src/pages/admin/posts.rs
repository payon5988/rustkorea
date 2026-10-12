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
use crate::models::{Category, Post, PostListItem};
use crate::services::markdown::render_markdown;

#[page("/admin/posts")]
pub async fn admin_posts(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "posts.moderate")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let posts = sqlx::query_as::<_, PostListItem>(
        "SELECT p.*, COALESCE(u.username, p.author_id) AS username FROM posts p
         LEFT JOIN users u ON u.id = p.author_id
         ORDER BY p.created_at DESC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"게시글 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/posts".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"게시글 관리"</h1>
                        <p style="font-size:13px; color:var(--muted2); margin-top:2px;">(format!("{}개 게시글", posts.len()))</p>
                    </div>
                    <a href="/admin/posts/new" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; gap:8px;">"+ 새 글쓰기"</a>
                </div>

                if posts.is_empty() {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:64px; text-align:center;">
                        <p style="font-size:14px; color:var(--muted2);">"등록된 게시글이 없습니다"</p>
                    </div>
                } else {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead>
                                <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"제목"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"작성자"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"작성일"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"관리"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for p in &posts {
                                    <tr style="border-bottom:1px solid var(--line);">
                                        <td style="padding:14px 20px;">
                                            <a href=(format!("/community/{}", p.id)) style="font-weight:600; color:var(--text3);">(p.title.clone())</a>
                                        </td>
                                        <td style="padding:14px 20px; color:var(--muted);">(p.username.clone())</td>
                                        <td style="padding:14px 20px; color:var(--dim); font-size:12px;">(p.created_at.get(..10).unwrap_or("").to_string())</td>
                                        <td style="padding:14px 20px; text-align:right;">
                                            <div style="display:flex; gap:8px; justify-content:flex-end; align-items:center;">
                                                <a href=(format!("/community/{}", p.id)) style="font-size:12px; color:var(--muted2);">"보기"</a>
                                                <a href=(format!("/admin/posts/{}/edit", p.id)) style="font-size:12px; color:var(--blue);">"수정"</a>
                                                <form action=(format!("/admin/posts/{}/delete", p.id)) method="POST" style="display:inline;" onsubmit="return confirm('정말 삭제하시겠습니까?')">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <button type="submit" style="font-size:12px; color:var(--red); background:none; border:none; cursor:pointer; font-weight:600;">"삭제"</button>
                                                </form>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                }
            </div>
        </body>
        </html>
    })
}

topcoat::router::path_param!(post_id);

#[route(POST "/admin/posts/{post_id}/delete")]
pub async fn delete_post(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let post_id = path_param::<PostId>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.posts.x.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(post_id)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other("/admin/posts"))
}

#[page("/admin/posts/{post_id}/edit")]
pub async fn admin_edit_post_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "posts.moderate")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let post_id = path_param::<PostId>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let post = match post {
        Some(p) => p,
        None => return Err(topcoat::router::error::redirect("/admin/posts").into()),
    };

    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE visible = 1 ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let current_category = post.category.clone();

    let input_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;";
    let textarea_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none; resize:vertical; font-family:inherit; line-height:1.6;";
    let label_style =
        "display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;";
    let update_url = format!("/admin/posts/{}/update", post_id);

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"게시글 수정 - Rust Korea Admin"</title>
            <link rel="stylesheet" href=(crate::admin_ui::EASYMDE_CSS_URL)>
            <script src=(crate::admin_ui::EASYMDE_JS_URL)></script>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
                <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/posts".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"게시글 수정"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">(format!("{} / {}", post.title, post.author_id))</p>
                </div>

                <form action=(update_url) method="POST" style="max-width:720px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <div style="margin-bottom:20px;">
                            <label style=(label_style)>"카테고리"</label>
                            <select name="category" style=(input_style)>
                                for cat in &categories {
                                    if cat.slug == current_category {
                                        <option value=(cat.slug.clone()) selected="selected">{(cat.name.clone())}</option>
                                    } else {
                                        <option value=(cat.slug.clone())>{(cat.name.clone())}</option>
                                    }
                                }
                            </select>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(label_style)>"제목"</label>
                            <input type="text" name="title" required="true" value=(post.title) style=(input_style)>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(label_style)>"내용 (Markdown)"</label>
                            <div style="margin-bottom:12px; display:flex; align-items:center; gap:10px;">
                            <label for="md-file" style="font-size:12px; color:var(--muted); cursor:pointer; border:1px dashed var(--border); border-radius:6px; padding:8px 14px;">".md 파일 불러오기"</label>
                            <input type="file" id="md-file" accept=".md,.markdown,.txt" style="display:none;">
                            <span style="font-size:11px; color:var(--dim);">"또는 :::youtube[영상ID] 로 영상 삽입"</span>
                            </div>
                            <textarea id="editor" name="content_md" rows="16" required="true" style=(textarea_style)>(post.content_md)</textarea>
                        </div>
                        <div style="display:flex; gap:10px;">
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 24px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">"수정 완료"</button>
                            <a href="/admin/posts" style="background:var(--border-soft); color:var(--muted); padding:10px 24px; border-radius:8px; font-weight:600; font-size:14px; border:1px solid var(--border); display:inline-flex; align-items:center;">"취소"</a>
                        </div>
                    </div>
                </form>
                <script>
                    {(crate::services::markdown::EASYMDE_INIT_JS)}
                        {(crate::services::markdown::MD_UPLOAD_JS)}
                        "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}bindMdUpload('md-file',easymde);"
                </script>
            </div>
        </body>
        </html>
    })
}

#[derive(serde::Deserialize)]
pub struct AdminUpdatePost {
    pub csrf_token: String,
    pub title: String,
    pub content_md: String,
    pub category: String,
}

#[route(POST "/admin/posts/{post_id}/update")]
pub async fn admin_update_post(
    cx: &Cx,
    Form(input): Form<AdminUpdatePost>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "posts.moderate")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let post_id = path_param::<PostId>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.posts.x.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let valid = sqlx::query_scalar::<_, String>(
        "SELECT slug FROM categories WHERE slug = $1 AND visible = 1",
    )
    .bind(&input.category)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    let category = valid.unwrap_or_else(|| "general".to_string());
    let content_html = render_markdown(&input.content_md);

    sqlx::query(
        "UPDATE posts SET title = $1, content_md = $2, content_html = $3, category = $4, updated_at = NOW()::text WHERE id = $5"
    )
    .bind(&input.title)
    .bind(&input.content_md)
    .bind(&content_html)
    .bind(&category)
    .bind(post_id)
    .execute(db)
    .await
    .ok();

    Ok(topcoat::router::error::see_other("/admin/posts"))
}

#[derive(serde::Deserialize)]
pub struct AdminNewPost {
    pub csrf_token: String,
    pub title: String,
    pub content_md: String,
    pub category: String,
}

#[route(POST "/admin/posts/create")]
pub async fn admin_create_post(
    cx: &Cx,
    Form(input): Form<AdminNewPost>,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.posts.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let admin_email = sqlx::query_scalar::<_, String>(
        "SELECT value FROM site_settings WHERE key = 'admin_email'",
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .unwrap_or_else(crate::config::contact_email);

    let admin_id = sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE email = $1")
        .bind(&admin_email)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "guest".into());

    let valid = sqlx::query_scalar::<_, String>(
        "SELECT slug FROM categories WHERE slug = $1 AND visible = 1",
    )
    .bind(&input.category)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    let category = valid.unwrap_or_else(|| "general".to_string());
    let content_html = render_markdown(&input.content_md);

    sqlx::query(
        "INSERT INTO posts (author_id, title, content_md, content_html, tags, category, created_at, updated_at)
         VALUES ($1, $2, $3, $4, '[]', $5, NOW()::text, NOW()::text)"
    )
    .bind(&admin_id)
    .bind(&input.title)
    .bind(&input.content_md)
    .bind(&content_html)
    .bind(&category)
    .execute(db)
    .await
    .ok();

    Ok(topcoat::router::error::see_other("/admin/posts"))
}

#[page("/admin/posts/new")]
pub async fn admin_new_post_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "posts.moderate")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE visible = 1 ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let input_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;";
    let textarea_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none; resize:vertical; font-family:inherit; line-height:1.6;";
    let label_style =
        "display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;";

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"새 글쓰기 - Rust Korea Admin"</title>
            <link rel="stylesheet" href=(crate::admin_ui::EASYMDE_CSS_URL)>
            <script src=(crate::admin_ui::EASYMDE_JS_URL)></script>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
                <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/posts".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"새 글쓰기"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"마크다운 문법을 지원합니다."</p>
                </div>

                <form action="/admin/posts/create" method="POST" style="max-width:720px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <div style="margin-bottom:20px;">
                            <label style=(label_style)>"카테고리"</label>
                            <select name="category" style=(input_style)>
                                for cat in &categories {
                                    <option value=(cat.slug.clone())>{(cat.name.clone())}</option>
                                }
                            </select>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(label_style)>"제목"</label>
                            <input type="text" name="title" required="true" placeholder="글 제목을 입력하세요" style=(input_style)>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label style=(label_style)>"내용 (Markdown)"</label>
                            <div style="margin-bottom:12px; display:flex; align-items:center; gap:10px;">
                            <label for="md-file" style="font-size:12px; color:var(--muted); cursor:pointer; border:1px dashed var(--border); border-radius:6px; padding:8px 14px;">".md 파일 불러오기"</label>
                            <input type="file" id="md-file" accept=".md,.markdown,.txt" style="display:none;">
                            <span style="font-size:11px; color:var(--dim);">"또는 :::youtube[영상ID] 로 영상 삽입"</span>
                            </div>
                            <textarea id="editor" name="content_md" rows="16" required="true" placeholder="글 내용을 마크다운으로 작성하세요..." style=(textarea_style)></textarea>
                        </div>
                        <div style="display:flex; gap:10px;">
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 24px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">"글 등록"</button>
                            <a href="/admin/posts" style="background:var(--border-soft); color:var(--muted); padding:10px 24px; border-radius:8px; font-weight:600; font-size:14px; border:1px solid var(--border); display:inline-flex; align-items:center;">"취소"</a>
                        </div>
                    </div>
                </form>
                <script>
                    {(crate::services::markdown::EASYMDE_INIT_JS)}
                        {(crate::services::markdown::MD_UPLOAD_JS)}
                        "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}bindMdUpload('md-file',easymde);"
                </script>
            </div>
        </body>
        </html>
    })
}
