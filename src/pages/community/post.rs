use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::db::Db;
use crate::models::{Bookmark, Post, PostLike};

use super::helpers::{
    can_edit_post, category_color, category_label, get_current_user, is_admin_user,
};

topcoat::router::path_param!(id);

/// post_views dedup window (repeat visits after this count again).
pub const VIEW_DEDUP_WINDOW_SECS: i64 = 86400;

/// Comment delete-preview length.
pub const COMMENT_PREVIEW_LEN: usize = 50;

#[derive(sqlx::FromRow)]
struct PostWithAuthor {
    id: String,
    author_id: String,
    title: String,
    content_html: String,
    category: String,
    view_count: i64,
    created_at: String,
    username: String,
}

/// Article + Breadcrumb JSON-LD for search engines (emitted raw, so `</`
/// sequences are neutralized to block `</script>` breakout).
pub fn article_json_ld(
    domain: &str,
    id: &str,
    title: &str,
    author: &str,
    created_at: &str,
    category: &str,
) -> String {
    let date = created_at.split_whitespace().next().unwrap_or(created_at);
    let url = format!("{domain}/community/{id}");
    let body = serde_json::json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Article",
                "headline": title,
                "author": {"@type": "Person", "name": author},
                "datePublished": date,
                "articleSection": category,
                "mainEntityOfPage": {"@type": "WebPage", "@id": url},
            },
            {
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "홈", "item": format!("{domain}/")},
                    {"@type": "ListItem", "position": 2, "name": "커뮤니티", "item": format!("{domain}/community")},
                    {"@type": "ListItem", "position": 3, "name": title, "item": url},
                ],
            },
        ],
    });
    serde_json::to_string(&body)
        .unwrap_or_default()
        .replace("</", "<\\/")
}

#[derive(sqlx::FromRow)]
struct CommentWithAuthor {
    id: String,
    author_id: String,
    content: String,
    created_at: String,
    username: String,
}

#[derive(serde::Deserialize)]
pub struct NewComment {
    pub csrf_token: String,
    pub content: String,
}

fn action_button_style(active_color: Option<&str>) -> String {
    match active_color {
        Some(color) => format!(
            "display:inline-flex; align-items:center; gap:6px; padding:8px 14px; border-radius:8px; border:1px solid {color}33; background:{color}0d; color:{color}; font-size:13px; font-weight:600; cursor:pointer;"
        ),
        None => "display:inline-flex; align-items:center; gap:6px; padding:8px 14px; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#6b7280; font-size:13px; font-weight:500; cursor:pointer;".to_string(),
    }
}

#[page("/community/{id}")]
pub async fn post_detail(cx: &Cx) -> Result<impl View + '_> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let hl_css = crate::services::markdown::HL_CSS_URL;
    let hl_js = crate::services::markdown::HL_JS_URL;
    let hl_rust = crate::services::markdown::HL_RUST_URL;
    let current_user = get_current_user(cx).await;

    // Deduplicated view count: one per viewer (user or IP hash) per 24h.
    let viewer_hash: String = if let Some(u) = &current_user {
        format!("user:{}", u.id)
    } else {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        crate::auth::session::client_ip(cx).hash(&mut h);
        format!("ip:{:x}", h.finish())
    };
    let counted = sqlx::query(
        "INSERT INTO post_views (post_id, viewer_hash) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(&viewer_hash)
    .execute(db)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);
    // Expire old dedup rows so repeat visits after 24h count again.
    sqlx::query("DELETE FROM post_views WHERE viewed_at < NOW() - make_interval(secs => $1)")
        .bind(VIEW_DEDUP_WINDOW_SECS)
        .execute(db)
        .await
        .ok();
    if counted == 1 {
        sqlx::query("UPDATE posts SET view_count = view_count + 1 WHERE id = $1")
            .bind(id)
            .execute(db)
            .await
            .ok();
    }

    let post: Option<PostWithAuthor> = sqlx::query_as::<_, PostWithAuthor>(
        "SELECT p.id, p.author_id, p.title, p.content_html, p.category, p.view_count, p.created_at,
                COALESCE(u.username, p.author_id) AS username
         FROM posts p LEFT JOIN users u ON u.id = p.author_id
         WHERE p.id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let comments: Vec<CommentWithAuthor> = if let Some(p) = &post {
        sqlx::query_as::<_, CommentWithAuthor>(
            "SELECT c.id, c.author_id, c.content, c.created_at,
                    COALESCE(u.username, c.author_id) AS username
             FROM comments c LEFT JOIN users u ON u.id = c.author_id
             WHERE c.post_id = $1 ORDER BY c.created_at ASC",
        )
        .bind(&p.id)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };

    let comment_count = comments.len();

    let like_count: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM post_likes WHERE post_id = $1")
            .bind(id)
            .fetch_one(db)
            .await
            .unwrap_or(0);

    let bookmark_count: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bookmarks WHERE post_id = $1")
            .bind(id)
            .fetch_one(db)
            .await
            .unwrap_or(0);

    let mut liked_by_me = false;
    let mut bookmarked_by_me = false;

    if let Some(user) = &current_user {
        liked_by_me = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM post_likes WHERE post_id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(&user.id)
        .fetch_one(db)
        .await
        .unwrap_or(0)
            > 0;

        bookmarked_by_me = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM bookmarks WHERE post_id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(&user.id)
        .fetch_one(db)
        .await
        .unwrap_or(0)
            > 0;
    }

    let access = crate::auth::access::load_access(db, cx).await;
    let is_admin = is_admin_user(&access);

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:720px;">
            if let Some(p) = post {
                <a href="/community" style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">
                    "<- 커뮤니티"
                </a>

                <script type="application/ld+json">{(topcoat::view::Unescaped::new_unchecked(article_json_ld(&crate::config::site_url(), &p.id, &p.title, &p.username, &p.created_at, category_label(&p.category))))}</script>

                <div style="margin-bottom:10px;">
                    <span style=(format!("font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:{}; background:{}1a;", category_color(&p.category), category_color(&p.category)))>
                        (category_label(&p.category))
                    </span>
                </div>

                <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:10px;">(p.title.clone())</h1>
                <div style="display:flex; gap:12px; font-size:12px; color:#9ca3af; margin-bottom:24px;">
                    <span>(p.username.clone())</span>
                    <span>(p.created_at.split(' ').next().unwrap_or(""))</span>
                    <span>(format!("조회 {}", p.view_count))</span>
                </div>

                let can_manage = can_edit_post(&access, &p.author_id);

                <div style="display:flex; align-items:center; gap:8px; margin-bottom:24px;">
                    if current_user.is_some() {
                        let like_style = action_button_style(if liked_by_me { Some("#DC2626") } else { None });
                        let bookmark_style = action_button_style(if bookmarked_by_me { Some("#2563EB") } else { None });
                        <form method="POST" action=(format!("/community/{}/like", p.id)) style="display:inline;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <button type="submit" style=(like_style)>
                                <svg width="15" height="15" viewBox="0 0 24 24" fill=(if liked_by_me { "currentColor" } else { "none" }) stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/>
                                </svg>
                                (format!("{like_count}"))
                            </button>
                        </form>
                        <form method="POST" action=(format!("/community/{}/bookmark", p.id)) style="display:inline;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <button type="submit" style=(bookmark_style)>
                                <svg width="15" height="15" viewBox="0 0 24 24" fill=(if bookmarked_by_me { "currentColor" } else { "none" }) stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16Z"/>
                                </svg>
                                (format!("북마크 ({bookmark_count})"))
                            </button>
                        </form>
                    } else {
                        <span style="display:inline-flex; align-items:center; gap:6px; padding:8px 14px; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#6b7280; font-size:13px; font-weight:500;">
                            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/>
                            </svg>
                            (format!("{like_count}"))
                        </span>
                    }
                    <div style="flex:1;"></div>
                    if can_manage {
                        <a href=(format!("/community/{}/edit", p.id)) style="display:inline-flex; align-items:center; padding:8px 14px; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">"수정"</a>
                        <form method="POST" action=(format!("/community/{}/delete", p.id)) style="display:inline;" onsubmit="return confirm('정말 삭제하시겠습니까?');">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <button type="submit" style="display:inline-flex; align-items:center; padding:8px 14px; border-radius:8px; border:1px solid #FECACA; background:#FEF2F2; color:#DC2626; font-size:13px; font-weight:500; cursor:pointer;">"삭제"</button>
                        </form>
                    }
                </div>

                <div class="card" style="padding:32px; margin-bottom:40px;">
                    <div class="markdown-body">
                        (topcoat::view::Unescaped::new_unchecked(p.content_html.clone()))
                    </div>
                </div>

                <link rel="stylesheet" href=(hl_css)>
                <script src=(hl_js)></script>
                <script src=(hl_rust)></script>
                <script>{(crate::services::markdown::HL_INIT_JS)}</script>
                <script>"initHighlight();"</script>

                <div>
                    <div style="display:flex; align-items:center; gap:8px; margin-bottom:16px;">
                        <h2 style="font-size:15px; font-weight:700;">"댓글"</h2>
                        <span style="font-size:12px; color:#9ca3af;">(format!("({comment_count})"))</span>
                    </div>

                    if current_user.is_some() {
                        <form method="POST" action=(format!("/community/{}/comment", p.id)) style="margin-bottom:24px; background:#fff; border:1px solid #E4E4E7; border-radius:8px; padding:16px;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <textarea name="content" rows="3" required="true" placeholder="댓글을 입력하세요..." style="width:100%; border:none; outline:none; resize:vertical; font-size:14px; line-height:1.6; color:#16181d; font-family:inherit;"></textarea>
                            <div style="display:flex; justify-content:flex-end; margin-top:12px;">
                                <button type="submit" class="btn-primary" style="padding:8px 18px; font-size:13px;">"댓글 등록"</button>
                            </div>
                        </form>
                    } else {
                        <div class="card" style="padding:24px; text-align:center; margin-bottom:24px;">
                            <p style="font-size:13px; color:#6b7280; margin-bottom:12px;">"댓글을 작성하려면 로그인이 필요합니다."</p>
                            <a href=(crate::auth::session::login_link(&format!("/community/{}", p.id))) class="btn-primary" style="display:inline-flex; padding:8px 18px; font-size:13px;">"로그인"</a>
                        </div>
                    }

                    if comment_count == 0 {
                        <div class="card" style="padding:40px; text-align:center;">
                            <p style="font-size:13px; color:#9ca3af;">"아직 댓글이 없습니다."</p>
                        </div>
                    } else {
                        <div style="display:flex; flex-direction:column; gap:8px;">
                            for comment in comments {
                                let can_delete_comment = is_admin || current_user.as_ref().map(|u| u.id == comment.author_id).unwrap_or(false);
                                <div class="card" style="padding:16px 20px;">
                                    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:6px;">
                                        <div style="display:flex; align-items:center; gap:8px;">
                                            <span style="font-size:12px; font-weight:600; color:#374151;">(comment.username.clone())</span>
                                            <span style="font-size:11px; color:#9ca3af;">(comment.created_at.split(' ').next().unwrap_or(""))</span>
                                        </div>
                                        if can_delete_comment {
                                            <form method="POST" action=(format!("/community/comment/{}/delete", comment.id)) onsubmit="return confirm('댓글을 삭제하시겠습니까?');">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <button type="submit" style="background:none; border:none; font-size:11px; color:#9ca3af; cursor:pointer; padding:2px 4px;">"삭제"</button>
                                            </form>
                                        }
                                    </div>
                                    <p style="font-size:13px; color:#374151; line-height:1.6; white-space:pre-wrap;">(comment.content.clone())</p>
                                </div>
                            }
                        </div>
                    }
                </div>
            } else {
                <div style="text-align:center; padding:100px 0;">
                    <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"게시글을 찾을 수 없습니다"</h1>
                    <p style="color:#6b7280; margin-bottom:24px;">"삭제되었거나 존재하지 않는 글입니다."</p>
                    <a href="/community" class="btn-primary" style="display:inline-flex;">"커뮤니티로 돌아가기"</a>
                </div>
            }
        </div>
    })
}

#[route(POST "/community/{id}/comment")]
pub async fn add_comment(
    cx: &Cx,
    Form(input): Form<NewComment>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_link(&format!("/community/{id}")),
        ));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "community.comment.add")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let post_exists: Option<Post> = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if post_exists.is_none() {
        return Ok(topcoat::router::error::see_other("/community"));
    }

    let content = input.content.trim().to_string();
    if content.is_empty() {
        return Ok(topcoat::router::error::see_other(format!(
            "/community/{id}"
        )));
    }

    sqlx::query(
        "INSERT INTO comments (post_id, author_id, content, created_at, updated_at)
         VALUES ($1, $2, $3, NOW()::text, NOW()::text)",
    )
    .bind(id)
    .bind(&user.id)
    .bind(&content)
    .execute(db)
    .await
    .ok();

    #[derive(sqlx::FromRow)]
    struct PostAuthor {
        author_id: String,
    }
    if let Ok(Some(pa)) =
        sqlx::query_as::<_, PostAuthor>("SELECT author_id FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await
    {
        crate::services::notifications::create_notification(
            db,
            &pa.author_id,
            "comment",
            Some(&user.id),
            Some(id),
            &format!(
                "{}님이 댓글을 남겼습니다: {}",
                user.username,
                content
                    .chars()
                    .take(COMMENT_PREVIEW_LEN)
                    .collect::<String>()
            ),
        )
        .await;
    }
    let comment_points = crate::services::notifications::point_rule(
        db,
        "points_comment",
        crate::services::notifications::POINTS_COMMENT_DEFAULT,
    )
    .await;
    crate::services::notifications::add_points(db, &user.id, comment_points, "댓글 작성").await;

    Ok(topcoat::router::error::see_other(format!(
        "/community/{id}"
    )))
}

#[route(POST "/community/comment/{id}/delete")]
pub async fn delete_comment(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if get_current_user(cx).await.is_none() {
        return Ok(topcoat::router::error::see_other(
            "/auth/login?redirect=/community",
        ));
    };
    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "community.comment.delete",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    #[derive(sqlx::FromRow)]
    struct CommentRow {
        id: String,
        post_id: String,
        author_id: String,
    }

    let comment: Option<CommentRow> = sqlx::query_as::<_, CommentRow>(
        "SELECT id, post_id, author_id FROM comments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let Some(comment) = comment else {
        return Ok(topcoat::router::error::see_other("/community"));
    };

    let access = crate::auth::access::load_access(db, cx).await;
    if !can_edit_post(&access, &comment.author_id) {
        return Ok(topcoat::router::error::see_other(format!(
            "/community/{}",
            comment.post_id
        )));
    }

    sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(&comment.id)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other(format!(
        "/community/{}",
        comment.post_id
    )))
}

#[route(POST "/community/{id}/like")]
pub async fn toggle_like(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_link(&format!("/community/{id}")),
        ));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "community.like").await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let existing: Option<PostLike> = sqlx::query_as::<_, PostLike>(
        "SELECT * FROM post_likes WHERE post_id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(&user.id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    if existing.is_some() {
        sqlx::query("DELETE FROM post_likes WHERE post_id = $1 AND user_id = $2")
            .bind(id)
            .bind(&user.id)
            .execute(db)
            .await
            .ok();
    } else {
        sqlx::query(
            "INSERT INTO post_likes (post_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(&user.id)
        .execute(db)
        .await
        .ok();

        #[derive(sqlx::FromRow)]
        struct PostAuthor {
            author_id: String,
        }
        let post_author: Option<PostAuthor> =
            sqlx::query_as("SELECT author_id FROM posts WHERE id = $1")
                .bind(id)
                .fetch_optional(db)
                .await
                .ok()
                .flatten();
        if let Some(pa) = post_author {
            if pa.author_id != user.id {
                crate::services::notifications::create_notification(
                    db,
                    &pa.author_id,
                    "like",
                    Some(&user.id),
                    Some(id),
                    &format!("{}님이 게시글에 좋아요를 눌렀습니다", user.username),
                )
                .await;
            }
        }
        let like_points = crate::services::notifications::point_rule(
            db,
            "points_like",
            crate::services::notifications::POINTS_LIKE_DEFAULT,
        )
        .await;
        crate::services::notifications::add_points(db, &user.id, like_points, "좋아요").await;
    }

    Ok(topcoat::router::error::see_other(format!(
        "/community/{id}"
    )))
}

#[route(POST "/community/{id}/bookmark")]
pub async fn toggle_bookmark(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    let Some(user) = get_current_user(cx).await else {
        return Ok(topcoat::router::error::see_other(
            crate::auth::session::login_link(&format!("/community/{id}")),
        ));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "community.bookmark")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let existing: Option<Bookmark> = sqlx::query_as::<_, Bookmark>(
        "SELECT * FROM bookmarks WHERE post_id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(&user.id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    if existing.is_some() {
        sqlx::query("DELETE FROM bookmarks WHERE post_id = $1 AND user_id = $2")
            .bind(id)
            .bind(&user.id)
            .execute(db)
            .await
            .ok();
    } else {
        sqlx::query(
            "INSERT INTO bookmarks (post_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(&user.id)
        .execute(db)
        .await
        .ok();
    }

    Ok(topcoat::router::error::see_other(format!(
        "/community/{id}"
    )))
}

#[cfg(test)]
mod tests {
    use super::article_json_ld;

    #[test]
    fn json_ld_is_valid_and_breakout_safe() {
        let s = article_json_ld(
            "https://rustkorea.kr",
            "abc",
            "제목</script><script>alert(1)",
            "admin&co",
            "2026-10-12 01:02:03",
            "공지사항",
        );
        assert!(!s.contains("</script>"), "script breakout possible");
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["@graph"][0]["@type"], "Article");
        assert_eq!(v["@graph"][0]["datePublished"], "2026-10-12");
        assert_eq!(v["@graph"][1]["@type"], "BreadcrumbList");
    }
}
