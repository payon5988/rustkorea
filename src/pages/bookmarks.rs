use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::db::Db;
use crate::pages::community::helpers::{
    category_color, category_label_in, get_current_user, login_redirect,
};

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct BookmarkPost {
    id: String,
    title: String,
    category: String,
    view_count: i64,
    author_id: String,
    created_at: String,
    username: String,
}

#[page("/my/bookmarks")]
pub async fn bookmarks_page(cx: &Cx) -> Result<impl View + '_> {
    let Some(user) = get_current_user(cx).await else {
        return Err(login_redirect(cx, "/my/bookmarks").await);
    };
    let db: &Db = topcoat::context::app_context(cx);
    let loc = crate::services::i18n::locale(cx);

    let page_num: i64 = topcoat::router::request::uri(cx)
        .query()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "page")
                .map(|(_, v)| v.into_owned())
        })
        .and_then(|p| p.parse().ok())
        .unwrap_or(1)
        .max(1);

    let total: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bookmarks WHERE user_id = $1")
            .bind(&user.id)
            .fetch_one(db)
            .await
            .unwrap_or(0);
    let (total_pages, page_num, offset) =
        crate::util::paginate(total, page_num, crate::util::DEFAULT_PER_PAGE);

    let bookmarks: Vec<BookmarkPost> = sqlx::query_as::<_, BookmarkPost>(
        "SELECT p.id, p.title, p.category, p.view_count, p.author_id, p.created_at,
                COALESCE(u.username, p.author_id) AS username
         FROM bookmarks b
         JOIN posts p ON p.id = b.post_id
         LEFT JOIN users u ON u.id = p.author_id
         WHERE b.user_id = $1
         ORDER BY b.created_at DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(&user.id)
    .bind(crate::util::DEFAULT_PER_PAGE)
    .bind(offset)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:720px;">
            <div style="margin-bottom:32px;">
                <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em;">(loc.pick("북마크", "Bookmarks"))</h1>
                <p style="font-size:13px; color:#6b7280; margin-top:4px;">(match loc {
                    crate::services::i18n::Locale::Ko => format!("{total}개 북마크"),
                    crate::services::i18n::Locale::En => format!("{total} bookmarks"),
                })</p>
            </div>

            if bookmarks.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#d1d5db" stroke-width="1.5" stroke-linecap="round" style="margin:0 auto 16px;">
                        <path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>
                    </svg>
                    <p style="color:#6b7280; margin-bottom:16px;">(loc.pick("아직 북마크한 글이 없습니다.", "You haven't bookmarked any posts yet."))</p>
                    <a href="/community" class="btn-primary" style="display:inline-flex;">(loc.pick("커뮤니티 둘러보기", "Browse the community"))</a>
                </div>
            } else {
                <div style="display:flex; flex-direction:column; gap:2px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                    for b in &bookmarks {
                        let badge_color = category_color(&b.category);
                        let badge_label = category_label_in(&b.category, loc);
                        <a href=(format!("/community/{}", b.id)) style="display:flex; align-items:center; padding:16px 20px; background:#fff;">
                            <span style=(format!("font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:{badge_color}; background:{badge_color}1a; flex-shrink:0; white-space:nowrap;"))>(badge_label)</span>
                            <div style="min-width:0; flex:1; margin-left:12px;">
                                <span style="font-size:14px; font-weight:600;">(b.title.clone())</span>
                            </div>
                            <div style="display:flex; align-items:center; gap:16px; flex-shrink:0; margin-left:24px;">
                                <span style="font-size:12px; color:#9ca3af;">(b.username.clone())</span>
                                <span style="font-size:12px; color:#9ca3af;">(b.created_at.split(' ').next().unwrap_or(""))</span>
                                <span style="display:inline-flex; align-items:center; gap:4px; font-size:12px; color:#9ca3af;">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
                                    (b.view_count)
                                </span>
                            </div>
                        </a>
                    }
                </div>

                if total_pages > 1 {
                    <div style="display:flex; align-items:center; justify-content:center; gap:6px; margin-top:32px;">
                        if page_num > 1 {
                            <a href=(format!("/my/bookmarks?page={}", page_num - 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">(loc.pick("이전", "Previous"))</a>
                        }
                        <span style="font-size:13px; color:#6b7280;">(format!("{page_num} / {total_pages}"))</span>
                        if page_num < total_pages {
                            <a href=(format!("/my/bookmarks?page={}", page_num + 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">(loc.pick("다음", "Next"))</a>
                        }
                    </div>
                }
            }
        </div>
    })
}
