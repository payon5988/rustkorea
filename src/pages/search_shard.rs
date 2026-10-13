use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    runtime::shard,
    view::{View, view},
};

use crate::db::Db;

/// Search caps (single place; SQL LIMITs are built from these).
pub const SEARCH_QUERY_MAX: usize = 64;
pub const SEARCH_SNIPPET_LEN: usize = 120;
pub const SEARCH_LIMIT_MANUALS: i64 = 5;
pub const SEARCH_LIMIT_CHAPTERS: i64 = 5;
pub const SEARCH_LIMIT_VIDEOS: i64 = 5;
pub const SEARCH_LIMIT_POSTS: i64 = 10;
pub const SEARCH_LIMIT_USERS: i64 = 3;

#[allow(dead_code)]
#[derive(Debug)]
struct SearchResult {
    id: String,
    title: String,
    description: String,
    result_type: String,
    url: String,
    badge: String,
    badge_color: String,
    badge_bg: String,
}

#[shard]
pub async fn search_results(cx: &Cx, query: String) -> Result<impl View> {
    let db: Db = topcoat::context::app_context::<Db>(cx).clone();
    let truncated: String = query.chars().take(SEARCH_QUERY_MAX).collect();
    let pattern = format!("%{}%", crate::auth::session::escape_like(&truncated));

    let mut results: Vec<SearchResult> = vec![];

    if !query.is_empty() {
        // Manuals
        let manuals: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT slug, title, description, difficulty FROM manual_series WHERE title LIKE $1 ESCAPE '\\' OR description LIKE $1 ESCAPE '\\' LIMIT $2"
        ).bind(&pattern).bind(SEARCH_LIMIT_MANUALS).fetch_all(&db).await.unwrap_or_default();
        for (slug, title, desc, diff) in manuals {
            results.push(SearchResult {
                id: slug.clone(),
                title,
                description: desc,
                result_type: "매뉴얼".into(),
                url: format!("/manuals/{slug}"),
                badge: diff,
                badge_color: "#059669".into(),
                badge_bg: "#0596691a".into(),
            });
        }

        // Manual chapters
        let chapters: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT ms.slug, mc.title, mc.content_md, ms.title AS series_title
             FROM manual_chapters mc JOIN manual_series ms ON ms.id = mc.series_id
             WHERE mc.title LIKE $1 ESCAPE '\\' OR mc.content_md LIKE $1 ESCAPE '\\' LIMIT $2",
        )
        .bind(&pattern)
        .bind(SEARCH_LIMIT_CHAPTERS)
        .fetch_all(&db)
        .await
        .unwrap_or_default();
        for (series_slug, title, _content, series_title) in chapters {
            results.push(SearchResult {
                id: title.clone(),
                title: format!("{title} ({series_title})"),
                description: "".into(),
                result_type: "매뉴얼 챕터".into(),
                url: format!("/manuals/{series_slug}"),
                badge: "챕터".into(),
                badge_color: "#2563EB".into(),
                badge_bg: "#2563EB1a".into(),
            });
        }

        // Videos
        let videos: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT video_id, title, description, published_at FROM youtube_videos WHERE title LIKE $1 ESCAPE '\\' OR description LIKE $1 ESCAPE '\\' LIMIT $2"
        ).bind(&pattern).bind(SEARCH_LIMIT_VIDEOS).fetch_all(&db).await.unwrap_or_default();
        for (vid, title, desc, pub_at) in videos {
            results.push(SearchResult {
                id: vid.clone(),
                title,
                description: desc,
                result_type: "영상".into(),
                url: crate::util::youtube_watch_url(&vid),
                badge: pub_at.split('T').next().unwrap_or("").to_string(),
                badge_color: "#DC2626".into(),
                badge_bg: "#DC26261a".into(),
            });
        }

        // Posts
        let posts: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT p.id, p.title, p.category, p.tags, p.content_md
             FROM posts p WHERE p.title LIKE $1 ESCAPE '\\' OR p.content_md LIKE $1 ESCAPE '\\' OR p.tags LIKE $1 ESCAPE '\\' LIMIT $2"
        ).bind(&pattern).bind(SEARCH_LIMIT_POSTS).fetch_all(&db).await.unwrap_or_default();
        for (id, title, category, _tags, content_md) in posts {
            let snippet = content_md
                .chars()
                .take(SEARCH_SNIPPET_LEN)
                .collect::<String>();
            results.push(SearchResult {
                id: id.clone(),
                title,
                description: snippet,
                result_type: "게시글".into(),
                url: format!("/community/{id}"),
                badge: category,
                badge_color: "#7C3AED".into(),
                badge_bg: "#7C3AED1a".into(),
            });
        }

        // Users
        let users: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT id, username, COALESCE(display_name, '') AS display_name FROM users WHERE username LIKE $1 ESCAPE '\\' OR display_name LIKE $1 ESCAPE '\\' LIMIT $2"
        ).bind(&pattern).bind(SEARCH_LIMIT_USERS).fetch_all(&db).await.unwrap_or_default();
        for (id, username, display_name) in users {
            let label = if display_name.is_empty() {
                username.clone()
            } else {
                format!("{username} ({display_name})")
            };
            results.push(SearchResult {
                id: id.clone(),
                title: label,
                description: "".into(),
                result_type: "사용자".into(),
                url: format!("/users/{username}"),
                badge: "사용자".into(),
                badge_color: "#0891B2".into(),
                badge_bg: "#0891B21a".into(),
            });
        }
    }

    let total = results.len();
    let has_results = total > 0;
    let has_query = !query.is_empty();

    Ok(view! {
        if has_query && has_results {
            <div style="font-size:13px; color:#9ca3af; margin-bottom:16px;">(format!("{total}개의 결과"))</div>
        }

        <div style="display:flex; flex-direction:column; gap:12px;">
            for item in results {
                <a
                    href=(item.url.clone())
                    style="display:flex; align-items:center; gap:14px; background:#fff; border:1px solid #E4E4E7; border-radius:10px; padding:16px 20px; transition:border-color 0.15s, box-shadow 0.15s; text-decoration:none; color:inherit;"
                    class="search-card"
                >
                    <div style="flex-shrink:0; width:40px; height:40px; border-radius:8px; display:flex; align-items:center; justify-content:center; font-size:11px; font-weight:700; color:#fff; background:(item.badge_color.clone());">
                        (item.result_type.chars().next().unwrap_or('X'))
                    </div>
                    <div style="min-width:0; flex:1;">
                        <div style="display:flex; align-items:center; gap:8px; margin-bottom:2px;">
                            <h3 style="font-size:15px; font-weight:600; color:#16181d; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">(item.title.clone())</h3>
                            <span style="flex-shrink:0; font-size:11px; font-weight:500; padding:2px 8px; border-radius:9999px; color:(item.badge_color.clone()); background:(item.badge_bg.clone()); white-space:nowrap;">(item.badge.clone())</span>
                        </div>
                        if !item.description.is_empty() {
                            <p style="font-size:13px; color:#6b7280; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">(item.description.clone())</p>
                        }
                        <span style="font-size:11px; color:#d1d5db;">(item.result_type.clone())</span>
                    </div>
                </a>
            }
        </div>

        if !has_results && has_query {
            <div style="text-align:center; padding:60px 0;">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#d1d5db" stroke-width="1.5" stroke-linecap="round" style="margin:0 auto 16px;">
                    <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
                </svg>
                <p style="font-size:15px; color:#9ca3af; font-weight:500;">"검색 결과가 없습니다"</p>
                <p style="font-size:13px; color:#d1d5db; margin-top:4px;">"다른 검색어를 시도해보세요."</p>
            </div>
        }
    })
}
