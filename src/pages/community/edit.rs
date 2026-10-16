use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::db::Db;
use crate::models::{Category, Post};
use crate::services::markdown::render_markdown;

use super::helpers::{
    can_edit_post, get_current_user, is_admin_user, is_moderator, load_categories, login_redirect,
    redirect_to,
};

topcoat::router::path_param!(id);

#[derive(serde::Deserialize)]
pub struct EditPost {
    pub csrf_token: String,
    pub title: String,
    pub content_md: String,
    pub category: String,
    #[serde(default)]
    pub published_at: String,
}

#[page("/community/{id}/edit")]
pub async fn edit_post_page(cx: &Cx) -> Result<impl View + '_> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    if get_current_user(cx).await.is_none() {
        return Err(login_redirect(cx, &format!("/community/{id}/edit")).await);
    };

    let post: Option<Post> = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let Some(post) = post else {
        return Err(redirect_to(cx, "/community").await);
    };

    let access = crate::auth::access::load_access(db, cx).await;
    let is_admin = is_admin_user(&access);

    if !can_edit_post(&access, &post.author_id) {
        return Err(redirect_to(cx, &format!("/community/{}", post.id)).await);
    }

    let all_categories = load_categories(db).await;

    let categories: Vec<Category> = all_categories
        .into_iter()
        .filter(|c| {
            if c.slug == super::helpers::category_slug::ANNOUNCE && !is_admin {
                return false;
            }
            let perm = c.write_permission.as_str();
            match perm {
                super::helpers::write_perm::ADMIN => is_admin,
                "editor" => is_moderator(&access),
                _ => true,
            }
        })
        .collect();

    Ok(view! {
        <div style="min-height:calc(100vh - 64px); display:flex; align-items:center; justify-content:center; padding:40px 24px; background:#f8f9fa;">
            <div style="width:100%; max-width:640px;">
                <div style="text-align:center; margin-bottom:32px;">
                    <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; color:#16181d;">"글 수정"</h1>
                    <p style="font-size:14px; color:#6b7280; margin-top:6px;">"마크다운 문법을 지원합니다."</p>
                </div>

                <div style="background:#fff; border:1px solid #e5e7eb; border-radius:8px; padding:32px; box-shadow:0 1px 3px rgba(0,0,0,0.05);">
                    <form action=(format!("/community/{}/update", post.id)) method="POST" style="display:flex; flex-direction:column; gap:20px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"카테고리"</label>
                            <select name="category" class="input">
                                for cat in &categories {
                                    <option value=(cat.slug.clone()) selected=(post.category == cat.slug)>(cat.name.clone())</option>
                                }
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"제목"</label>
                            <input type="text" name="title" required="true" class="input" value=(post.title.clone())>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"내용 (Markdown)"</label>
                            <div style="margin-bottom:12px; display:flex; align-items:center; gap:10px;">
                                <label for="md-file" style="font-size:13px; color:#6b7280; cursor:pointer; border:1px dashed #d1d5db; border-radius:6px; padding:8px 14px;">".md 파일 불러오기"</label>
                                <input type="file" id="md-file" accept=".md,.markdown,.txt" style="display:none;">
                                <span style="font-size:12px; color:#9ca3af;">"또는 :::youtube[영상ID] 로 영상 삽입"</span>
                            </div>
                            <textarea id="editor" name="content_md" rows="16" required="true" class="input">(post.content_md.clone())</textarea>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"예약 발행 (비워두면 즉시 발행)"</label>
                            <input type="datetime-local" name="published_at" class="input" value=(crate::util::format_schedule_kst(post.published_at.as_deref()))>
                            <p style="font-size:12px; color:#9ca3af; margin-top:4px;">"한국 시간 기준. 비우면 예약이 해제됩니다."</p>
                        </div>
                        <button type="submit" class="btn-primary" style="width:100%; justify-content:center; padding:14px;">"수정 완료"</button>
                    </form>

                    <div style="margin-top:24px; text-align:center; font-size:13px; color:#6b7280;">
                        <a href=(format!("/community/{}", post.id)) style="color:#2563EB; font-weight:500;">"<- 글로 돌아가기"</a>
                    </div>
                </div>
            </div>
        </div>
        <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL) integrity=(crate::admin_ui::TOAST_CSS_SRI) crossorigin="anonymous">
        <script src=(crate::admin_ui::TOAST_JS_URL) integrity=(crate::admin_ui::TOAST_JS_SRI) crossorigin="anonymous"></script>
        <script>
            {(crate::services::markdown::EASYMDE_INIT_JS)}
                    {(crate::services::markdown::MD_UPLOAD_JS)}
                    "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}bindMdUpload('md-file',easymde);"
        </script>
    })
}

#[route(POST "/community/{id}/update")]
pub async fn update_post(
    cx: &Cx,
    Form(input): Form<EditPost>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if get_current_user(cx).await.is_none() {
        return Ok(topcoat::router::error::see_other(format!(
            "/auth/login?redirect=/community/{id}/edit"
        )));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "community.post.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let post: Option<Post> = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let Some(post) = post else {
        return Ok(topcoat::router::error::see_other("/community"));
    };

    let access = crate::auth::access::load_access(db, cx).await;
    let is_admin = is_admin_user(&access);

    if !can_edit_post(&access, &post.author_id) {
        return Ok(topcoat::router::error::see_other(format!(
            "/community/{}",
            post.id
        )));
    }

    let valid = sqlx::query_scalar::<_, String>(
        "SELECT slug FROM categories WHERE slug = $1 AND visible = 1",
    )
    .bind(&input.category)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    let category = valid.unwrap_or_else(|| super::helpers::category_slug::GENERAL.to_string());
    let category = if category == super::helpers::category_slug::ANNOUNCE && !is_admin {
        super::helpers::category_slug::GENERAL.to_string()
    } else {
        category
    };

    let content_html = render_markdown(&input.content_md);

    // Empty = publish immediately (clears any schedule).
    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => {
            return Ok(topcoat::router::error::see_other(format!(
                "/community/{}/edit?error=schedule",
                post.id
            )));
        }
    };

    sqlx::query(
        "UPDATE posts SET title = $1, content_md = $2, content_html = $3, category = $4, published_at = $5, updated_at = NOW()::text
         WHERE id = $6"
    )
    .bind(&input.title)
    .bind(&input.content_md)
    .bind(&content_html)
    .bind(&category)
    .bind(&published_at)
    .bind(&post.id)
    .execute(db)
    .await
    .ok();

    Ok(topcoat::router::error::see_other(format!(
        "/community/{}",
        post.id
    )))
}
