use std::future::Future;
use topcoat::router::request::uri;
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
use crate::util::query_param;

const PER_PAGE: i64 = 20;

#[derive(sqlx::FromRow)]
struct CommentRow {
    id: String,
    post_id: String,
    content: String,
    created_at: String,
    username: String,
    post_title: String,
}

#[derive(sqlx::FromRow)]
struct PostOption {
    id: String,
    title: String,
}

#[derive(serde::Deserialize)]
pub struct DeleteCommentForm {
    pub csrf_token: String,
    pub comment_id: String,
}

fn page_url(post_filter: &Option<String>, target: i64) -> String {
    match post_filter {
        Some(post_id) => format!("/admin/comments?post={post_id}&page={target}"),
        None => format!("/admin/comments?page={target}"),
    }
}

#[page("/admin/comments")]
pub async fn admin_comments_page(cx: &Cx) -> Result<impl View + '_> {
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
    let post_filter = query_param(cx, "post");
    let requested_page: i64 = query_param(cx, "page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1)
        .max(1);

    let total: i64 = if let Some(post_id) = &post_filter {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE post_id = $1")
            .bind(post_id)
            .fetch_one(db)
            .await
            .unwrap_or(0)
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments")
            .fetch_one(db)
            .await
            .unwrap_or(0)
    };

    let (total_pages, page_num, offset) = crate::util::paginate(total, requested_page, PER_PAGE);

    let comments: Vec<CommentRow> = if let Some(post_id) = &post_filter {
        sqlx::query_as::<_, CommentRow>(
            "SELECT c.id, c.post_id, c.content, c.created_at,
                    COALESCE(u.username, c.author_id) AS username,
                    COALESCE(p.title, '삭제된 게시글') AS post_title
             FROM comments c
             LEFT JOIN users u ON u.id = c.author_id
             LEFT JOIN posts p ON p.id = c.post_id
             WHERE c.post_id = $1
             ORDER BY c.created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(post_id)
        .bind(PER_PAGE)
        .bind(offset)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, CommentRow>(
            "SELECT c.id, c.post_id, c.content, c.created_at,
                    COALESCE(u.username, c.author_id) AS username,
                    COALESCE(p.title, '삭제된 게시글') AS post_title
             FROM comments c
             LEFT JOIN users u ON u.id = c.author_id
             LEFT JOIN posts p ON p.id = c.post_id
             ORDER BY c.created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(PER_PAGE)
        .bind(offset)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    };

    let post_options =
        sqlx::query_as::<_, PostOption>("SELECT id, title FROM posts ORDER BY created_at DESC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    let has_prev = page_num > 1;
    let has_next = page_num < total_pages;

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"댓글 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/comments".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"댓글 관리"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"전체 댓글을 확인하고 관리합니다."</p>
                </div>

                if uri(cx).query().unwrap_or("").contains("deleted=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:var(--green);">"댓글이 삭제되었습니다."</div>
                }

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:16px 20px; margin-bottom:20px; display:flex; align-items:end; gap:12px;">
                    <form method="get" action="/admin/comments" style="display:flex; align-items:end; gap:12px; flex:1;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="min-width:280px;">
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"게시글 필터"</label>
                            <select name="post" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:9px 14px; font-size:13px; color:var(--text2); outline:none; appearance:auto;">
                                <option value="">"전체 게시글"</option>
                                for opt in &post_options {
                                    <option value=(opt.id.clone()) selected=(post_filter.as_deref() == Some(opt.id.as_str()))>(opt.title.clone())</option>
                                }
                            </select>
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:9px 18px; border-radius:8px; font-size:13px; font-weight:600; border:none; cursor:pointer; white-space:nowrap;">"필터 적용"</button>
                    </form>
                    <span style="font-size:13px; color:var(--muted2); white-space:nowrap;">(format!("총 {total}개"))</span>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                    if comments.is_empty() {
                        <div style="padding:60px; text-align:center;">
                            <p style="font-size:14px; color:var(--dim);">"등록된 댓글이 없습니다"</p>
                        </div>
                    } else {
                        <table style="width:100%; border-collapse:collapse; font-size:13px;">
                            <thead>
                                <tr>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid var(--border); background:var(--line);">"게시글"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid var(--border); background:var(--line);">"작성자"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid var(--border); background:var(--line);">"내용"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid var(--border); background:var(--line);">"작성일"</th>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid var(--border); background:var(--line);">"관리"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for comment in &comments {
                                    <tr>
                                        <td style="padding:14px 20px; border-bottom:1px solid var(--line); max-width:220px;">
                                            <a href=(format!("/community/{}", comment.post_id)) style="display:block; font-size:13px; font-weight:600; color:var(--text3); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">
                                                (comment.post_title.clone())
                                            </a>
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid var(--line);">
                                            <div style="display:flex; align-items:center; gap:8px;">
                                                <div style="width:26px; height:26px; border-radius:6px; background:rgba(222,91,48,0.1); display:flex; align-items:center; justify-content:center; flex-shrink:0; font-size:11px; font-weight:700; color:#DE5B30;">(comment.username.chars().next().unwrap_or('?').to_string())</div>
                                                <span style="color:var(--muted);">(comment.username.clone())</span>
                                            </div>
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid var(--line); max-width:320px;">
                                            <p style="color:var(--text3); line-height:1.5; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden;">(comment.content.clone())</p>
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid var(--line); color:var(--muted2); font-size:12px; white-space:nowrap;">
                                            (comment.created_at.get(..10).unwrap_or("").to_string())
                                        </td>
                                        <td style="padding:14px 20px; border-bottom:1px solid var(--line);">
                                            <form action="/admin/comments/delete" method="POST" onsubmit="return confirm('정말 이 댓글을 삭제하시겠습니까?')">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="comment_id" value=(comment.id.clone())>
                                                <button type="submit" style="background:none; border:1px solid rgba(239,68,68,0.3); color:var(--red); padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600; transition:all 0.15s;">"삭제"</button>
                                            </form>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    }
                </div>

                if total_pages > 1 {
                    <div style="display:flex; align-items:center; justify-content:center; gap:6px; margin-top:24px;">
                        if has_prev {
                            <a href=(page_url(&post_filter, page_num - 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid var(--border); background:none; color:var(--muted); font-size:13px; font-weight:500;">"이전"</a>
                        }
                        <span style="height:32px; padding:0 14px; display:inline-flex; align-items:center; font-size:13px; color:var(--muted2);">(format!("{page_num} / {total_pages}"))</span>
                        if has_next {
                            <a href=(page_url(&post_filter, page_num + 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid var(--border); background:none; color:var(--muted); font-size:13px; font-weight:500;">"다음"</a>
                        }
                    </div>
                }
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/comments/delete")]
pub async fn admin_delete_comment(
    cx: &Cx,
    Form(input): Form<DeleteCommentForm>,
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

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.comments.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(&input.comment_id)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other(
        "/admin/comments?deleted=1",
    ))
}
