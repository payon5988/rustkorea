use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, route},
    view::{View, view},
};

use crate::db::Db;
use crate::models::Category;
use crate::services::markdown::render_markdown;

use super::helpers::{
    can_write_post, get_current_user, is_admin_user, load_categories, login_redirect,
};

#[derive(serde::Deserialize)]
pub struct NewPost {
    pub csrf_token: String,
    pub title: String,
    pub content_md: String,
    pub category: String,
    pub tags: Option<String>,
    #[serde(default)]
    pub published_at: String,
}

#[route(POST "/community/create")]
pub async fn create_post(
    cx: &Cx,
    Form(input): Form<NewPost>,
) -> Result<topcoat::router::error::SeeOther> {
    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?redirect=/community/new",
        ));
    };

    if !can_write_post(&Some(user.clone())) {
        return Ok(topcoat::router::error::see_other("/community"));
    }

    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "community.post.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let valid = sqlx::query_scalar::<_, String>(
        "SELECT slug FROM categories WHERE slug = $1 AND visible = 1",
    )
    .bind(&input.category)
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    let category = valid.unwrap_or_else(|| super::helpers::category_slug::GENERAL.to_string());
    let category = if category == super::helpers::category_slug::ANNOUNCE && !is_admin_user(&access)
    {
        super::helpers::category_slug::GENERAL.to_string()
    } else {
        category
    };

    if !super::helpers::can_write_to_category(db, &access, &category).await {
        return Ok(topcoat::router::error::see_other("/community"));
    }

    let content_html = render_markdown(&input.content_md);

    let tags_json = crate::util::tags_json(input.tags);

    // Empty = publish immediately; otherwise a KST datetime for scheduled release.
    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => {
            return Ok(topcoat::router::error::see_other(
                "/community/new?error=schedule",
            ));
        }
    };

    sqlx::query(
        "INSERT INTO posts (author_id, title, content_md, content_html, tags, category, published_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW()::text, NOW()::text)"
    )
    .bind(&user.id)
    .bind(&input.title)
    .bind(&input.content_md)
    .bind(&content_html)
    .bind(&tags_json)
    .bind(&category)
    .bind(&published_at)
    .execute(db)
    .await
    .ok();

    let post_points = crate::services::notifications::point_rule(
        db,
        "points_post",
        crate::services::notifications::POINTS_POST_DEFAULT,
    )
    .await;
    crate::services::notifications::add_points(db, &user.id, post_points, "게시글 작성").await;
    crate::services::notifications::check_and_award_badges(db, &user.id).await;

    Ok(topcoat::router::error::see_other("/community"))
}

#[page("/community/new")]
pub async fn new_post_page(cx: &Cx) -> Result<impl View + '_> {
    let Some(user) = get_current_user(cx).await else {
        return Err(login_redirect(cx, "/community/new").await);
    };

    if !can_write_post(&Some(user.clone())) {
        return Err(super::helpers::redirect_to(cx, "/community").await);
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let access = crate::auth::access::load_access(db, cx).await;
    let all_categories = load_categories(db).await;

    let is_admin = is_admin_user(&access);
    let categories: Vec<Category> = all_categories
        .into_iter()
        .filter(|c| {
            if c.slug == super::helpers::category_slug::ANNOUNCE && !is_admin {
                return false;
            }
            let perm = c.write_permission.as_str();
            match perm {
                super::helpers::write_perm::ADMIN => is_admin,
                "editor" => super::helpers::is_moderator(&access),
                _ => true,
            }
        })
        .collect();

    let select_style = "width:100%; background:#fff; border:1px solid #d1d5db; border-radius:6px; padding:10px 12px; font-size:16px; color:#111827; outline:none;";

    Ok(view! {
        <div style="min-height:calc(100vh - 64px); display:flex; align-items:center; justify-content:center; padding:40px 24px; background:#f8f9fa;">
            <div style="width:100%; max-width:640px;">
                <div style="text-align:center; margin-bottom:32px;">
                    <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; color:#16181d;">"새 글쓰기"</h1>
                    <p style="font-size:14px; color:#6b7280; margin-top:6px;">"마크다운 문법을 지원합니다."</p>
                </div>

                <div style="background:#fff; border:1px solid #e5e7eb; border-radius:8px; padding:32px; box-shadow:0 1px 3px rgba(0,0,0,0.05);">
                    <form action="/community/create" method="POST" style="display:flex; flex-direction:column; gap:20px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"작성자"</label>
                            <div style="display:flex; align-items:center; gap:10px; padding:10px 12px; background:#f9fafb; border:1px solid #E4E4E7; border-radius:6px;">
                                if let Some(avatar) = &user.avatar_url {
                                    <img src=(avatar.clone()) alt="" style="width:24px; height:24px; border-radius:50%;">
                                }
                                <span style="font-size:14px; font-weight:600; color:#16181d;">(user.username.clone())</span>
                            </div>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"카테고리"</label>
                            <select name="category" style=(select_style)>
                                for cat in &categories {
                                    <option value=(cat.slug.clone())>{(cat.name.clone())}</option>
                                }
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"제목"</label>
                            <input type="text" name="title" required="true" placeholder="글 제목을 입력하세요" class="input">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"태그 (쉼표 구분, 최대 5개)"</label>
                            <input type="text" name="tags" placeholder="예: rust, async, tokio" class="input">
                            <p style="font-size:12px; color:#9ca3af; margin-top:4px;">"검색에 활용됩니다."</p>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"예약 발행 (비워두면 즉시 발행)"</label>
                            <input type="datetime-local" name="published_at" class="input">
                            <p style="font-size:12px; color:#9ca3af; margin-top:4px;">"한국 시간 기준. 예약된 글은 시간이 되면 자동 공개됩니다."</p>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:#374151; margin-bottom:8px;">"내용 (Markdown)"</label>
                            <div style="margin-bottom:12px; display:flex; align-items:center; gap:10px;">
                                <label for="md-file" style="font-size:13px; color:#6b7280; cursor:pointer; border:1px dashed #d1d5db; border-radius:6px; padding:8px 14px;">".md 파일 불러오기"</label>
                                <input type="file" id="md-file" accept=".md,.markdown,.txt" style="display:none;">
                                <span style="font-size:12px; color:#9ca3af;">"또는 :::youtube[영상ID] 로 영상 삽입"</span>
                            </div>
                            <textarea id="editor" name="content_md" rows="16" required="true" placeholder="글 내용을 마크다운으로 작성하세요..." class="input"></textarea>
                        </div>
                        <button type="submit" class="btn-primary" style="width:100%; justify-content:center; padding:14px;">"글 등록"</button>
                    </form>

                    <div style="margin-top:24px; text-align:center; font-size:13px; color:#6b7280;">
                        <a href="/community" style="color:#2563EB; font-weight:500;">"<- 커뮤니티로 돌아가기"</a>
                    </div>
                </div>
                <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL) integrity=(crate::admin_ui::TOAST_CSS_SRI) crossorigin="anonymous">
                <script src=(crate::admin_ui::TOAST_JS_URL) integrity=(crate::admin_ui::TOAST_JS_SRI) crossorigin="anonymous"></script>
                <script>
                    {(crate::services::markdown::EASYMDE_INIT_JS)}
                    {(crate::services::markdown::MD_UPLOAD_JS)}
                    "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}bindMdUpload('md-file',easymde);"
                </script>
            </div>
        </div>
    })
}
