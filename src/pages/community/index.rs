use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::components::ad_unit::AdUnit;
use crate::db::Db;
use crate::models::PostListItem;

/// Pagination window and preview caps.
pub const PAGE_WINDOW: i64 = 2;
pub const POST_PREVIEW_LEN: usize = 80;

use super::helpers::{
    category_color, category_label_in, get_current_user, load_categories, query_param,
    valid_category,
};

enum PageItem {
    Page(i64),
    Gap,
}

fn build_page_items(current: i64, total_pages: i64) -> Vec<PageItem> {
    let mut wanted: Vec<i64> = vec![1, total_pages];
    for p in (current - PAGE_WINDOW)..=(current + PAGE_WINDOW) {
        if p >= 1 && p <= total_pages {
            wanted.push(p);
        }
    }
    wanted.sort_unstable();
    wanted.dedup();

    let mut items = Vec::new();
    let mut prev = 0;
    for p in wanted {
        if prev > 0 && p - prev > 1 {
            items.push(PageItem::Gap);
        }
        items.push(PageItem::Page(p));
        prev = p;
    }
    items
}

fn page_url(cat: &Option<String>, target: i64) -> String {
    match cat {
        Some(c) => format!("/community?cat={c}&page={target}"),
        None => format!("/community?page={target}"),
    }
}

fn tab_style(active: bool) -> &'static str {
    if active {
        "padding:10px 18px; font-size:14px; font-weight:700; color:#2563EB; border-bottom:2px solid #2563EB; display:inline-flex;"
    } else {
        "padding:10px 18px; font-size:14px; font-weight:500; color:#6b7280; border-bottom:2px solid transparent; display:inline-flex;"
    }
}

fn page_link_style(active: bool) -> &'static str {
    if active {
        "min-width:32px; height:32px; padding:0 10px; display:inline-flex; align-items:center; justify-content:center; border-radius:8px; background:#2563EB; color:#fff; font-size:13px; font-weight:600;"
    } else {
        "min-width:32px; height:32px; padding:0 10px; display:inline-flex; align-items:center; justify-content:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;"
    }
}

fn badge_style(color: &str) -> String {
    format!(
        "font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:{color}; background:{color}1a; flex-shrink:0; white-space:nowrap;"
    )
}

#[page("/community")]
pub async fn community_index(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let loc = crate::services::i18n::locale(cx);
    let current_user = get_current_user(cx).await;
    let can_write = super::helpers::can_write_post(&current_user);

    let cat_filter = if let Some(cat) = query_param(cx, "cat") {
        if valid_category(db, &cat).await {
            Some(cat)
        } else {
            None
        }
    } else {
        None
    };
    let requested_page: i64 = query_param(cx, "page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1)
        .max(1);

    let total = if let Some(cat) = &cat_filter {
        sqlx::query_scalar::<_, i64>(&format!(
            "SELECT COUNT(*) FROM posts WHERE category = $1 AND {}",
            crate::util::published_clause("")
        ))
        .bind(cat)
        .fetch_one(db)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, i64>(&format!(
            "SELECT COUNT(*) FROM posts WHERE {}",
            crate::util::published_clause("")
        ))
        .fetch_one(db)
        .await
        .unwrap_or(0)
    };

    let (total_pages, page_num, offset) =
        crate::util::paginate(total, requested_page, crate::util::DEFAULT_PER_PAGE);

    let posts = if let Some(cat) = &cat_filter {
        sqlx::query_as::<_, PostListItem>(&format!(
            "SELECT p.*, COALESCE(u.username, p.author_id) AS username FROM posts p
             LEFT JOIN users u ON u.id = p.author_id
             WHERE p.category = $1 AND {} ORDER BY p.created_at DESC LIMIT $2 OFFSET $3",
            crate::util::published_clause("p")
        ))
        .bind(cat)
        .bind(crate::util::DEFAULT_PER_PAGE)
        .bind(offset)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, PostListItem>(&format!(
            "SELECT p.*, COALESCE(u.username, p.author_id) AS username FROM posts p
             LEFT JOIN users u ON u.id = p.author_id
             WHERE {} ORDER BY p.created_at DESC LIMIT $1 OFFSET $2",
            crate::util::published_clause("p")
        ))
        .bind(crate::util::DEFAULT_PER_PAGE)
        .bind(offset)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    };

    let page_items = build_page_items(page_num, total_pages);
    let has_prev = page_num > 1;
    let has_next = page_num < total_pages;

    let categories = load_categories(db).await;

    let tabs: Vec<(String, String)> = {
        let mut t = vec![("all".to_string(), loc.pick("전체", "All").to_string())];
        for cat in &categories {
            t.push((cat.slug.clone(), loc.text_owned(&cat.name, &cat.name_en)));
        }
        t
    };
    // Badge text: the DB category's localized name, else the built-in label.
    let category_name = move |slug: &str| -> String {
        categories
            .iter()
            .find(|c| c.slug == slug)
            .map(|c| loc.text_owned(&c.name, &c.name_en))
            .unwrap_or_else(|| category_label_in(slug, loc).to_string())
    };

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:32px;">
                <div>
                    <div style="font-size:11px; font-weight:700; color:#2563EB; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"COMMUNITY"</div>
                    <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em;">(loc.pick("커뮤니티", "Community"))</h1>
                </div>
                if current_user.is_some() {
                    if can_write {
                        <a href="/community/new" class="btn-primary">(loc.pick("글쓰기", "New post"))</a>
                    }
                } else {
                    <a href="/auth/login?redirect=/community" class="btn-secondary">(loc.pick("로그인", "Log in"))</a>
                }
            </div>

            <div style="display:flex; gap:4px; border-bottom:1px solid #E4E4E7; margin-bottom:24px; overflow-x:auto;">
                for (key, label) in tabs {
                    let cf = cat_filter.clone();
                    let active = match &cf {
                        Some(c) => key == c.as_str(),
                        None => key == "all",
                    };
                    if key == "all" {
                        <a href="/community" style=(tab_style(active))>(label)</a>
                    } else {
                        <a href=(format!("/community?cat={key}")) style=(tab_style(active))>(label)</a>
                    }
                }
            </div>

            if posts.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="color:#6b7280; margin-bottom:16px;">(loc.pick("아직 게시글이 없습니다.", "No posts yet."))</p>
                    if can_write {
                        <a href="/community/new" class="btn-primary" style="display:inline-flex;">(loc.pick("첫 글 쓰기", "Write the first post"))</a>
                    } else if current_user.is_none() {
                        <a href="/auth/login?redirect=/community/new" class="btn-primary" style="display:inline-flex;">(loc.pick("로그인하고 첫 글 쓰기", "Log in to write the first post"))</a>
                    }
                </div>
            } else {
                <div class="card" style="display:flex; flex-direction:column; gap:1px; background:#e5e7eb; border-radius:12px; overflow:hidden;">
                    for post in posts {
                        let badge_color = category_color(&post.category);
                        let badge_label = category_name(&post.category);
                        let preview: String = post.content_md.replace('\n', " ").chars().take(POST_PREVIEW_LEN).collect();
                        <a href=(format!("/community/{}", post.id)) style="display:flex; align-items:center; padding:16px 20px; background:#fff;">
                            <span style=(badge_style(badge_color))>(badge_label)</span>
                            <div style="min-width:0; flex:1; margin-left:12px;">
                                <span style="font-size:14px; font-weight:600;">(post.title.clone())</span>
                                <p style="font-size:12px; color:#9ca3af; margin-top:4px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">(preview)</p>
                            </div>
                            <div style="display:flex; align-items:center; gap:16px; flex-shrink:0; margin-left:24px;">
                                <span class="post-author" style="font-size:12px; color:#9ca3af;">(post.username.clone())</span>
                                <span style="font-size:12px; color:#9ca3af;">(post.created_at.split(' ').next().unwrap_or(""))</span>
                                <span class="post-view-count" style="display:inline-flex; align-items:center; gap:4px; font-size:12px; color:#9ca3af;">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/>
                                    </svg>
                                    (post.view_count)
                                </span>
                            </div>
                        </a>
                    }
                </div>

                {AdUnit(slot_id: "community_infeed".into())}

                if total_pages > 1 {
                    <div style="display:flex; align-items:center; justify-content:center; gap:6px; margin-top:32px;">
                        if has_prev {
                            <a href=(page_url(&cat_filter, page_num - 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">(loc.pick("이전", "Previous"))</a>
                        }
                        for item in page_items {
                            match item {
                                PageItem::Page(n) => {
                                    <a href=(page_url(&cat_filter, n)) style=(page_link_style(n == page_num))>(n)</a>
                                }
                                PageItem::Gap => {
                                    <span style="padding:0 4px; color:#9ca3af; font-size:13px;">"..."</span>
                                }
                            }
                        }
                        if has_next {
                            <a href=(page_url(&cat_filter, page_num + 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">(loc.pick("다음", "Next"))</a>
                        }
                    </div>
                }
            }
        </div>
    })
}
