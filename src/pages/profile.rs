use crate::db::Db;
use crate::models::User;
use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    router::path_param,
    view::{View, view},
};

topcoat::router::path_param!(username);

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct UserPost {
    id: String,
    title: String,
    category: String,
    view_count: i64,
    created_at: String,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct UserComment {
    id: String,
    post_id: String,
    content: String,
    created_at: String,
    post_title: String,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct UserBadgeRow {
    id: String,
    name: String,
    description: String,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct BookmarkedPost {
    id: String,
    title: String,
    category: String,
    view_count: i64,
    created_at: String,
    author_id: String,
}

fn tab_style(t: &str, current: &str) -> String {
    if t == current {
        "padding:10px 16px; font-size:14px; font-weight:700; color:#2563EB; border-bottom:2px solid #2563EB;".into()
    } else {
        "padding:10px 16px; font-size:14px; font-weight:500; color:#6b7280; border-bottom:2px solid transparent;".into()
    }
}

#[page("/users/{username}")]
pub async fn user_profile(cx: &Cx) -> Result<impl View + '_> {
    let username = path_param::<Username>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    let tab = topcoat::router::request::uri(cx)
        .query()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "tab")
                .map(|(_, v)| v.into_owned())
        })
        .unwrap_or_else(|| "posts".into());

    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    let viewer = crate::auth::session::current_user(db, cx).await;
    let is_owner = match (&viewer, &user) {
        (Some(v), Some(u)) => v.id == u.id,
        _ => false,
    };

    let posts: Vec<UserPost> = if let Some(u) = &user {
        if tab == "posts" || tab == "likes" {
            sqlx::query_as::<_, UserPost>(
                "SELECT id, title, category, view_count, created_at FROM posts WHERE author_id = $1 ORDER BY created_at DESC LIMIT 50"
            ).bind(&u.id).fetch_all(db).await.unwrap_or_default()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let comments: Vec<UserComment> = if let Some(u) = &user {
        if tab == "comments" {
            sqlx::query_as::<_, UserComment>(
            "SELECT c.id, c.post_id, c.content, c.created_at, COALESCE(p.title, '') AS post_title
             FROM comments c LEFT JOIN posts p ON p.id = c.post_id
             WHERE c.author_id = $1 ORDER BY c.created_at DESC LIMIT 50"
        ).bind(&u.id).fetch_all(db).await.unwrap_or_default()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let bookmarks: Vec<BookmarkedPost> = if is_owner && tab == "bookmarks" {
        sqlx::query_as::<_, BookmarkedPost>(
            "SELECT p.id, p.title, p.category, p.view_count, p.created_at, p.author_id
             FROM bookmarks b JOIN posts p ON p.id = b.post_id
             WHERE b.user_id = $1 ORDER BY b.created_at DESC LIMIT 50",
        )
        .bind(user.as_ref().unwrap().id.clone())
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };

    let post_count: i64 = if let Some(ref u) = user {
        sqlx::query_scalar("SELECT COUNT(*) FROM posts WHERE author_id = $1")
            .bind(&u.id)
            .fetch_one(db)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let comment_count: i64 = if let Some(ref u) = user {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE author_id = $1")
            .bind(&u.id)
            .fetch_one(db)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let like_count: i64 = if let Some(ref u) = user {
        sqlx::query_scalar("SELECT COUNT(*) FROM post_likes WHERE user_id = $1")
            .bind(&u.id)
            .fetch_one(db)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let bookmark_count_q: i64 = if let Some(ref u) = user {
        sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks WHERE user_id = $1")
            .bind(&u.id)
            .fetch_one(db)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let badges: Vec<UserBadgeRow> = if let Some(ref u) = user {
        sqlx::query_as::<_, UserBadgeRow>(
            "SELECT b.id, b.name, b.description FROM user_badges ub JOIN badges b ON b.id = ub.badge_id WHERE ub.user_id = $1"
        ).bind(&u.id).fetch_all(db).await.unwrap_or_default()
    } else {
        vec![]
    };

    let category_color = |cat: &str| -> &'static str {
        match cat {
            "qna" => "#7c3aed",
            "showcase" => "#059669",
            "jobs" => "#d97706",
            "announce" => "#dc2626",
            _ => "#2563EB",
        }
    };
    let category_label = |cat: &str| -> &'static str {
        match cat {
            "qna" => "질문",
            "showcase" => "쇼케이스",
            "jobs" => "구인",
            "announce" => "공지",
            _ => "일반",
        }
    };

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:720px;">
            if let Some(u) = user {
                <div style="display:flex; align-items:center; gap:20px; padding:24px; background:#fff; border:1px solid #E4E4E7; border-radius:12px; margin-bottom:24px;">
                    if let Some(avatar) = &u.avatar_url {
                        <img src=(avatar.clone()) alt="" style="width:64px; height:64px; border-radius:50%;">
                    } else {
                        <div style="width:64px; height:64px; border-radius:50%; background:#2563EB; color:#fff; display:flex; align-items:center; justify-content:center; font-size:24px; font-weight:700;">
                            (u.username.chars().next().unwrap_or('U'))
                        </div>
                    }
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:800; margin-bottom:4px;">(u.username.clone())</h1>
                        if !u.bio.is_empty() {
                            <p style="font-size:13px; color:#6b7280; margin-bottom:6px;">(u.bio.clone())</p>
                        }
                        <div style="display:flex; gap:12px; font-size:13px; color:#6b7280;">
                            <span style="font-weight:600; color:#DE5B30;">(format!("{}P", u.points))</span>
                            <span>(format!("{} 게시글", post_count))</span>
                            <span>(format!("{} 댓글", comment_count))</span>
                            <span>(format!("{} 좋아요", like_count))</span>
                        </div>
                        <div style="display:flex; gap:8px; margin-top:8px; flex-wrap:wrap;">
                            for b in &badges {
                                <span style="font-size:11px; padding:2px 8px; border-radius:9999px; color:#2563EB; background:#2563EB1a;">(b.name.clone())</span>
                            }
                        </div>
                    </div>
                </div>

                <div style="display:flex; gap:4px; border-bottom:1px solid #E4E4E7; margin-bottom:24px; overflow-x:auto;">
                    <a href=(format!("/users/{}?tab=posts", u.username)) style=(tab_style("posts", &tab))>(format!("작성글 ({})", post_count))</a>
                    <a href=(format!("/users/{}?tab=comments", u.username)) style=(tab_style("comments", &tab))>(format!("댓글 ({})", comment_count))</a>
                    <a href=(format!("/users/{}?tab=likes", u.username)) style=(tab_style("likes", &tab))>(format!("좋아요 ({})", like_count))</a>
                    <a href=(format!("/users/{}?tab=bookmarks", u.username)) style=(tab_style("bookmarks", &tab))>(format!("북마크 ({})", bookmark_count_q))</a>
                </div>

                if tab == "posts" {
                    if posts.is_empty() {
                        <div class="card" style="padding:60px; text-align:center;"><p style="color:#6b7280;">"작성한 글이 없습니다."</p></div>
                    } else {
                        <div style="display:flex; flex-direction:column; gap:2px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                            for p in &posts {
                                let bc = category_color(&p.category);
                                let bl = category_label(&p.category);
                                <a href=(format!("/community/{}", p.id)) style="display:flex; align-items:center; padding:14px 20px; background:#fff; gap:12px; transition:background 0.15s;">
                                    <span style=(format!("font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:{bc}; background:{bc}1a; flex-shrink:0;"))>(bl)</span>
                                    <span style="font-size:14px; font-weight:600; flex:1; color:#16181d;">(p.title.clone())</span>
                                    <span style="font-size:12px; color:#9ca3af;">(p.created_at.split(' ').next().unwrap_or(""))</span>
                                </a>
                            }
                        </div>
                    }
                } else if tab == "comments" {
                    if comments.is_empty() {
                        <div class="card" style="padding:60px; text-align:center;"><p style="color:#6b7280;">"작성한 댓글이 없습니다."</p></div>
                    } else {
                        <div style="display:flex; flex-direction:column; gap:8px;">
                            for c in &comments {
                                <a href=(format!("/community/{}", c.post_id)) class="card" style="padding:16px 20px;">
                                    <div style="font-size:12px; color:#2563EB; margin-bottom:4px;">(c.post_title.clone())</div>
                                    <p style="font-size:13px; color:#374151; line-height:1.5;">(c.content.chars().take(120).collect::<String>())</p>
                                    <div style="font-size:12px; color:#9ca3af; margin-top:4px;">(c.created_at.split(' ').next().unwrap_or(""))</div>
                                </a>
                            }
                        </div>
                    }
                } else if tab == "bookmarks" {
                    if bookmarks.is_empty() {
                        <div class="card" style="padding:60px; text-align:center;"><p style="color:#6b7280;">"북마크한 글이 없습니다."</p></div>
                    } else {
                        <div style="display:flex; flex-direction:column; gap:2px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                            for p in &bookmarks {
                                let bc = category_color(&p.category);
                                let bl = category_label(&p.category);
                                <a href=(format!("/community/{}", p.id)) style="display:flex; align-items:center; padding:14px 20px; background:#fff; gap:12px; transition:background 0.15s;">
                                    <span style=(format!("font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:{bc}; background:{bc}1a; flex-shrink:0;"))>(bl)</span>
                                    <span style="font-size:14px; font-weight:600; flex:1; color:#16181d;">(p.title.clone())</span>
                                    <span style="font-size:12px; color:#9ca3af;">(p.created_at.split(' ').next().unwrap_or(""))</span>
                                </a>
                            }
                        </div>
                    }
                } else if tab == "likes" {
                    <div class="card" style="padding:60px; text-align:center;"><p style="color:#6b7280;">"좋아요한 글 목록은 공개 설정에 따라 표시됩니다."</p></div>
                }
            } else {
                <div style="text-align:center; padding:80px 0;">
                    <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"사용자를 찾을 수 없습니다"</h1>
                    <p style="color:#6b7280;">"존재하지 않는 사용자입니다."</p>
                </div>
            }
        </div>
    })
}
