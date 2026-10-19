use std::future::Future;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::page;
use topcoat::view::{View, view};

use crate::components::ad_unit::AdUnit;
use crate::components::video_card::VideoCard;
use crate::db::Db;
use crate::models::{ManualSeries, PostListItem, YoutubeVideo};

/// Home section caps.
pub const HOME_MANUALS: i64 = 3;
pub const HOME_VIDEOS: i64 = 4;
pub const HOME_POSTS: i64 = 5;

#[page("/")]
pub async fn home(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let loc = crate::services::i18n::locale(cx);

    let manuals = sqlx::query_as::<_, ManualSeries>(&format!(
        "SELECT * FROM manual_series WHERE {} ORDER BY created_at DESC LIMIT $1",
        crate::util::published_clause("")
    ))
    .bind(HOME_MANUALS)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let videos = sqlx::query_as::<_, YoutubeVideo>(
        "SELECT * FROM youtube_videos ORDER BY published_at DESC LIMIT $1",
    )
    .bind(HOME_VIDEOS)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let posts = sqlx::query_as::<_, PostListItem>(&format!(
        "SELECT p.*, COALESCE(u.username, p.author_id) AS username FROM posts p
         LEFT JOIN users u ON u.id = p.author_id
         WHERE {} ORDER BY p.created_at DESC LIMIT $1",
        crate::util::published_clause("p"),
    ))
    .bind(HOME_POSTS)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let series_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM manual_series WHERE published_at IS NULL OR published_at::timestamptz <= NOW()")
    .fetch_one(db)
    .await
    .unwrap_or(0);
    let chapter_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM manual_chapters mc
         JOIN manual_series ms ON ms.id = mc.series_id
         WHERE (mc.published_at IS NULL OR mc.published_at::timestamptz <= NOW())
           AND (ms.published_at IS NULL OR ms.published_at::timestamptz <= NOW())",
    )
    .fetch_one(db)
    .await
    .unwrap_or(0);
    let video_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM youtube_videos")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let post_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM posts WHERE {}",
        crate::util::published_clause("")
    ))
    .fetch_one(db)
    .await
    .unwrap_or(0);

    // Promo banner: settings-driven copy + latest notice link + expiry.
    let banner_enabled =
        crate::services::site_settings::get_setting(db, "banner_enabled").await == "true";
    let banner_until = crate::services::site_settings::get_setting(db, "banner_until").await;
    let setting = |key: &'static str| crate::services::site_settings::get_localized(db, key, loc);
    let banner_kicker = setting("banner_kicker").await;
    let banner_title = setting("banner_title").await;
    let banner_text = setting("banner_text").await;
    let banner_cta = setting("banner_cta").await;
    let banner_post: Option<String> = sqlx::query_scalar(&format!(
        "SELECT id FROM posts WHERE category = 'announce' AND {} ORDER BY created_at DESC LIMIT 1",
        crate::util::published_clause("")
    ))
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    let show_banner =
        crate::util::banner_visible_on(banner_enabled, &banner_until, &crate::util::today_kst())
            && banner_post.is_some();

    Ok(view! {
        <div>

            <section style="padding:80px 0 60px; background:#f8f9fa;">
                <div class="container">
                    <div style="max-width:640px;">
                        <div class="badge fade-up" style="background:#dbeafe; color:#1e40af; margin-bottom:24px;">
                            "Rust Korea Community"
                        </div>
                        <h1 class="section-heading fade-up delay-1" style="margin-bottom:20px;">
                            (loc.pick("한국 Rust 생태계의 ", "The "))
                            <span style="color:#2563EB; font-style:italic;">(loc.pick("허브", "hub"))</span>
                            (loc.pick("", " of the Korean Rust ecosystem"))
                        </h1>
                        <p class="section-sub fade-up delay-2" style="margin-bottom:32px; max-width:480px;">
                            (loc.pick("매뉴얼, 영상, 커뮤니티 - Rust를 배우고 나누는 공간.", "Manuals, videos and community — a place to learn and share Rust."))
                        </p>
                        <div class="fade-up delay-3" style="display:flex; gap:12px;">
                            <a href="/manuals" class="btn-primary">(loc.pick("매뉴얼 시작하기", "Start the manuals"))</a>
                            <a href="/community" class="btn-secondary">(loc.pick("커뮤니티 참여", "Join the community"))</a>
                        </div>
                    </div>
                </div>
            </section>

            if show_banner {
                <style>
                    {r#".book-banner-inner { display: flex; align-items: center; justify-content: space-between; gap: 1rem; }
                    .book-banner-cta { flex-shrink: 0; }
                    @media (max-width: 640px) { .book-banner-inner { flex-direction: column; align-items: flex-start; } }"#}
                </style>
                <div class="book-banner-container" id="book-banner" style="max-width:1200px; margin:1.5rem auto; padding:0 1rem;">
                    <a href=(format!("/community/{}", banner_post.clone().unwrap_or_default())) style="text-decoration:none; display:block;">
                        <div class="topcoat-book-banner" style="background:linear-gradient(135deg, #1f2833 0%, #25283d 100%); border:1px solid #45a29e; border-radius:6px; padding:1rem 1.5rem; box-shadow:0 4px 15px rgba(0,0,0,0.2);">
                            <div class="book-banner-inner">
                                <div style="display:flex; align-items:center; gap:1rem; min-width:0;">
                                    <span style="background:#45a29e; color:#fff; padding:0.2rem 0.6rem; font-size:0.8rem; font-weight:bold; border-radius:4px; text-transform:uppercase; letter-spacing:1px; white-space:nowrap;">(banner_kicker.clone())</span>
                                    <p style="color:#fff; margin:0; font-size:1.05rem; font-weight:500; line-height:1.4;">
                                        (loc.pick("신간 ", "New release: "))
                                        <strong style="color:#e5c158;">(banner_title.clone())</strong>
                                        (loc.pick(" 출시 — ", " — "))
                                        (banner_text.clone())
                                    </p>
                                </div>
                                <div class="book-banner-cta" style="color:#45a29e; font-weight:bold; font-size:0.95rem; display:flex; align-items:center; gap:0.3rem; white-space:nowrap;">
                                    (banner_cta.clone())
                                    <span>" ➔"</span>
                                </div>
                            </div>
                        </div>
                    </a>
                    <div style="text-align:right; margin-top:4px;">
                        <button type="button" onclick="dismissBookBanner()" style="background:none; border:none; font-size:11px; color:#9ca3af; cursor:pointer;">(loc.pick("닫기", "Dismiss"))</button>
                    </div>
                </div>
                <script>
                    {r#"function dismissBookBanner(){try{localStorage.setItem('rk-book-banner','0');}catch(e){}var b=document.getElementById('book-banner');if(b){b.style.display='none';}}(function(){try{if(localStorage.getItem('rk-book-banner')==='0'){var b=document.getElementById('book-banner');if(b){b.style.display='none';}}}catch(e){}})();"#}
                </script>
            }

            <section class="container" style="margin-top:-30px; margin-bottom:60px;">
                <div class="grid-3" style="display:grid; grid-template-columns:repeat(3,1fr); gap:24px;">
                    <div class="card" style="padding:24px; text-align:center;">
                        <div style="font-size:2rem; font-weight:800; color:#16181d; letter-spacing:-0.02em;">(series_count)</div>
                        <div style="font-size:13px; color:#6b7280; margin-top:4px;">(loc.pick("매뉴얼 시리즈", "Manual series"))</div>
                        <div style="font-size:12px; color:#9ca3af; margin-top:2px;">(match loc {
                            crate::services::i18n::Locale::Ko => format!("총 {chapter_count}개 챕터"),
                            crate::services::i18n::Locale::En => format!("{chapter_count} chapters in total"),
                        })</div>
                    </div>
                    <div class="card" style="padding:24px; text-align:center;">
                        <div style="font-size:2rem; font-weight:800; color:#16181d; letter-spacing:-0.02em;">(video_count)</div>
                        <div style="font-size:13px; color:#6b7280; margin-top:4px;">(loc.pick("기술 영상", "Tech videos"))</div>
                    </div>
                    <div class="card" style="padding:24px; text-align:center;">
                        <div style="font-size:2rem; font-weight:800; color:#16181d; letter-spacing:-0.02em;">(post_count)</div>
                        <div style="font-size:13px; color:#6b7280; margin-top:4px;">(loc.pick("게시글", "Posts"))</div>
                    </div>
                </div>
            </section>

            <section class="container" style="margin-bottom:60px;">
                <div style="display:flex; align-items:baseline; justify-content:space-between; margin-bottom:24px;">
                    <h2 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">(loc.pick("매뉴얼", "Manuals"))</h2>
                    <a href="/manuals" style="font-size:14px; color:#2563EB; font-weight:500; transition:color 0.15s;">(loc.pick("전체 보기", "View all"))</a>
                </div>
                if !manuals.is_empty() {
                    <div class="grid-3" style="display:grid; grid-template-columns:repeat(3,1fr); gap:16px;">
                        for item in manuals {
                            <a href=(format!("/manuals/{}", item.slug)) class="card" style="padding:24px; display:block;">
                                <div style="display:flex; align-items:center; gap:8px; margin-bottom:12px;">
                                    <span style="font-size:24px;">(item.thumbnail_url.clone().unwrap_or_else(|| "📚".into()))</span>
                                    <span class="badge" style=((match item.difficulty.as_str() {
                                            crate::models::difficulty::ADVANCED => "background:#fef2f2; color:#dc2626;",
                                            crate::models::difficulty::INTERMEDIATE => "background:#fffbeb; color:#d97706;",
                                            _ => "background:#f0fdf4; color:#16a34a;",
                                        }).to_string())>
                                        (crate::models::difficulty::label_in(item.difficulty.as_str(), loc))
                                    </span>
                                </div>
                                <h3 style="font-size:15px; font-weight:700; margin-bottom:6px; letter-spacing:-0.01em; color:#16181d;">(loc.text_owned(&item.title, &item.title_en))</h3>
                                <p style="font-size:13px; color:#6b7280; line-height:1.5; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden;">(loc.text_owned(&item.description, &item.description_en))</p>
                            </a>
                        }
                    </div>
                } else {
                    <div class="card" style="padding:60px; text-align:center;">
                        <p style="color:#6b7280;">(loc.pick("등록된 매뉴얼이 없습니다", "No manuals yet"))</p>
                    </div>
                }
            </section>

            <section class="container" style="margin-bottom:60px;">
                <div style="display:flex; align-items:baseline; justify-content:space-between; margin-bottom:24px;">
                    <h2 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">(loc.pick("영상", "Videos"))</h2>
                    <a href="/youtube" style="font-size:14px; color:#2563EB; font-weight:500; transition:color 0.15s;">(loc.pick("전체 보기", "View all"))</a>
                </div>
                if !videos.is_empty() {
                    <div class="grid-4" style="display:grid; grid-template-columns:repeat(4,1fr); gap:16px;">
                        for item in videos {
                            VideoCard(video: item)
                        }
                    </div>
                } else {
                    <div class="card" style="padding:60px; text-align:center;">
                        <p style="color:#6b7280;">(loc.pick("등록된 영상이 없습니다", "No videos yet"))</p>
                    </div>
                }
            </section>

            {AdUnit(slot_id: "home_infeed".into())}

            <section class="container" style="margin-bottom:60px;">
                <div style="display:flex; align-items:baseline; justify-content:space-between; margin-bottom:24px;">
                    <h2 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">(loc.pick("커뮤니티", "Community"))</h2>
                    <a href="/community" style="font-size:14px; color:#2563EB; font-weight:500; transition:color 0.15s;">(loc.pick("전체 보기", "View all"))</a>
                </div>
                if !posts.is_empty() {
                    <div class="card" style="overflow:hidden;">
                        for item in posts {
                            <a href=(format!("/community/{}", item.id)) style="display:flex; align-items:center; justify-content:space-between; padding:14px 20px; border-bottom:1px solid #f3f4f6; transition:background 0.15s;">
                                <div style="display:flex; align-items:center; gap:16px; min-width:0; flex:1;">
                                    <span style="font-size:14px; font-weight:600; color:#16181d; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">(item.title.clone())</span>
                                </div>
                                <div style="display:flex; align-items:center; gap:16px; flex-shrink:0; margin-left:20px;">
                                    <span style="font-size:12px; color:#6b7280;">(item.username.clone())</span>
                                    <span style="font-size:12px; color:#9ca3af;">(item.created_at.split(' ').next().unwrap_or(""))</span>
                                </div>
                            </a>
                        }
                    </div>
                } else {
                    <div class="card" style="padding:60px; text-align:center;">
                        <p style="color:#6b7280; margin-bottom:12px;">(loc.pick("아직 게시글이 없습니다", "No posts yet"))</p>
                        <a href="/community/new" class="btn-primary" style="display:inline-flex;">(loc.pick("첫 글 쓰기", "Write the first post"))</a>
                    </div>
                }
            </section>

            <section class="container" style="padding-bottom:80px;">
                <div class="card" style="padding:60px; text-align:center; background:#f8f9fa;">
                    <h2 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:12px;">
                        (loc.pick("Rust를 함께 배우실 분?", "Want to learn Rust together?"))
                    </h2>
                    <p style="font-size:14px; color:#6b7280; margin-bottom:28px; max-width:400px; margin-left:auto; margin-right:auto;">
                        (loc.pick("한국 Rust 커뮤니티에서 함께 성장하세요.", "Grow with the Korean Rust community."))
                    </p>
                    <a href="/community/new" class="btn-primary">(loc.pick("글쓰기 시작하기", "Start writing"))</a>
                </div>
            </section>

        </div>
    })
}
