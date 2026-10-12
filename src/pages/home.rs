use std::future::Future;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::page;
use topcoat::view::{View, view};

use crate::components::ad_unit::AdUnit;
use crate::components::video_card::VideoCard;
use crate::db::Db;
use crate::models::{ManualSeries, PostListItem, YoutubeVideo};

#[page("/")]
pub async fn home(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);

    let manuals = sqlx::query_as::<_, ManualSeries>(
        "SELECT * FROM manual_series ORDER BY created_at DESC LIMIT 3",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let videos = sqlx::query_as::<_, YoutubeVideo>(
        "SELECT * FROM youtube_videos ORDER BY published_at DESC LIMIT 4",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let posts = sqlx::query_as::<_, PostListItem>(
        "SELECT p.*, COALESCE(u.username, p.author_id) AS username FROM posts p
         LEFT JOIN users u ON u.id = p.author_id
         ORDER BY p.created_at DESC LIMIT 5",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let series_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM manual_series WHERE visibility = 'public'")
            .fetch_one(db)
            .await
            .unwrap_or(0);
    let chapter_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM manual_chapters mc
         JOIN manual_series ms ON ms.id = mc.series_id
         WHERE ms.visibility = 'public'",
    )
    .fetch_one(db)
    .await
    .unwrap_or(0);
    let video_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM youtube_videos")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(db)
        .await
        .unwrap_or(0);

    Ok(view! {
        <div>

            <section style="padding:80px 0 60px; background:#f8f9fa;">
                <div class="container">
                    <div style="max-width:640px;">
                        <div class="badge fade-up" style="background:#dbeafe; color:#1e40af; margin-bottom:24px;">
                            "Rust Korea Community"
                        </div>
                        <h1 class="section-heading fade-up delay-1" style="margin-bottom:20px;">
                            "한국 Rust 생태계의 "
                            <span style="color:#2563EB; font-style:italic;">"허브"</span>
                        </h1>
                        <p class="section-sub fade-up delay-2" style="margin-bottom:32px; max-width:480px;">
                            "매뉴얼, 영상, 커뮤니티 - Rust를 배우고 나누는 공간."
                        </p>
                        <div class="fade-up delay-3" style="display:flex; gap:12px;">
                            <a href="/manuals" class="btn-primary">"매뉴얼 시작하기"</a>
                            <a href="/community" class="btn-secondary">"커뮤니티 참여"</a>
                        </div>
                    </div>
                </div>
            </section>

            <section class="container" style="margin-top:-30px; margin-bottom:60px;">
                <div class="grid-3" style="display:grid; grid-template-columns:repeat(3,1fr); gap:24px;">
                    <div class="card" style="padding:24px; text-align:center;">
                        <div style="font-size:2rem; font-weight:800; color:#16181d; letter-spacing:-0.02em;">(series_count)</div>
                        <div style="font-size:13px; color:#6b7280; margin-top:4px;">"매뉴얼 시리즈"</div>
                        <div style="font-size:12px; color:#9ca3af; margin-top:2px;">(format!("총 {chapter_count}개 챕터"))</div>
                    </div>
                    <div class="card" style="padding:24px; text-align:center;">
                        <div style="font-size:2rem; font-weight:800; color:#16181d; letter-spacing:-0.02em;">(video_count)</div>
                        <div style="font-size:13px; color:#6b7280; margin-top:4px;">"기술 영상"</div>
                    </div>
                    <div class="card" style="padding:24px; text-align:center;">
                        <div style="font-size:2rem; font-weight:800; color:#16181d; letter-spacing:-0.02em;">(post_count)</div>
                        <div style="font-size:13px; color:#6b7280; margin-top:4px;">"게시글"</div>
                    </div>
                </div>
            </section>

            <section class="container" style="margin-bottom:60px;">
                <div style="display:flex; align-items:baseline; justify-content:space-between; margin-bottom:24px;">
                    <h2 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"매뉴얼"</h2>
                    <a href="/manuals" style="font-size:14px; color:#2563EB; font-weight:500; transition:color 0.15s;">"전체 보기"</a>
                </div>
                if !manuals.is_empty() {
                    <div class="grid-3" style="display:grid; grid-template-columns:repeat(3,1fr); gap:16px;">
                        for item in manuals {
                            <a href=(format!("/manuals/{}", item.slug)) class="card" style="padding:24px; display:block;">
                                <div style="display:flex; align-items:center; gap:8px; margin-bottom:12px;">
                                    <span style="font-size:24px;">(item.thumbnail_url.clone().unwrap_or_else(|| "📚".into()))</span>
                                    <span class="badge" style=((match item.difficulty.as_str() {
                                            "advanced" => "background:#fef2f2; color:#dc2626;",
                                            "intermediate" => "background:#fffbeb; color:#d97706;",
                                            _ => "background:#f0fdf4; color:#16a34a;",
                                        }).to_string())>
                                        (match item.difficulty.as_str() {
                                            "advanced" => "고급",
                                            "intermediate" => "중급",
                                            _ => "초급",
                                        })
                                    </span>
                                </div>
                                <h3 style="font-size:15px; font-weight:700; margin-bottom:6px; letter-spacing:-0.01em; color:#16181d;">(item.title.clone())</h3>
                                <p style="font-size:13px; color:#6b7280; line-height:1.5; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden;">(item.description.clone())</p>
                            </a>
                        }
                    </div>
                } else {
                    <div class="card" style="padding:60px; text-align:center;">
                        <p style="color:#6b7280;">"등록된 매뉴얼이 없습니다"</p>
                    </div>
                }
            </section>

            <section class="container" style="margin-bottom:60px;">
                <div style="display:flex; align-items:baseline; justify-content:space-between; margin-bottom:24px;">
                    <h2 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"영상"</h2>
                    <a href="/youtube" style="font-size:14px; color:#2563EB; font-weight:500; transition:color 0.15s;">"전체 보기"</a>
                </div>
                if !videos.is_empty() {
                    <div class="grid-4" style="display:grid; grid-template-columns:repeat(4,1fr); gap:16px;">
                        for item in videos {
                            VideoCard(video: item)
                        }
                    </div>
                } else {
                    <div class="card" style="padding:60px; text-align:center;">
                        <p style="color:#6b7280;">"등록된 영상이 없습니다"</p>
                    </div>
                }
            </section>

            {AdUnit(slot_id: "home_infeed".into())}

            <section class="container" style="margin-bottom:60px;">
                <div style="display:flex; align-items:baseline; justify-content:space-between; margin-bottom:24px;">
                    <h2 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"커뮤니티"</h2>
                    <a href="/community" style="font-size:14px; color:#2563EB; font-weight:500; transition:color 0.15s;">"전체 보기"</a>
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
                        <p style="color:#6b7280; margin-bottom:12px;">"아직 게시글이 없습니다"</p>
                        <a href="/community/new" class="btn-primary" style="display:inline-flex;">"첫 글 쓰기"</a>
                    </div>
                }
            </section>

            <section class="container" style="padding-bottom:80px;">
                <div class="card" style="padding:60px; text-align:center; background:#f8f9fa;">
                    <h2 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:12px;">
                        "Rust를 함께 배우실 분?"
                    </h2>
                    <p style="font-size:14px; color:#6b7280; margin-bottom:28px; max-width:400px; margin-left:auto; margin-right:auto;">
                        "한국 Rust 커뮤니티에서 함께 성장하세요."
                    </p>
                    <a href="/community/new" class="btn-primary">"글쓰기 시작하기"</a>
                </div>
            </section>

        </div>
    })
}
