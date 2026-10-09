use crate::components::video_card::VideoCard;
use crate::db::Db;
use crate::models::YoutubeVideo;
use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

#[page("/youtube")]
pub async fn youtube_index(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;
    let tier = crate::services::site_settings::get_setting(db, "youtube_visibility").await;
    let tier = if tier.is_empty() {
        "public".to_string()
    } else {
        tier
    };
    let gate = crate::auth::access::visibility_gate(
        access.as_ref(),
        &tier,
        "videos.view_staff",
        "premium.access",
    );
    if gate == crate::auth::access::Gate::LoginRequired {
        return Err(topcoat::router::error::redirect("/auth/login?redirect=/youtube").into());
    }
    let locked = gate == crate::auth::access::Gate::Forbidden;
    let lock_msg: String = match tier.as_str() {
        "staff" => "스태프 전용 영상입니다.".to_string(),
        "premium" => "유료 전용 영상입니다(준비중).".to_string(),
        _ => "회원 전용 영상입니다.".to_string(),
    };
    let page_num: i64 = topcoat::router::request::uri(cx)
        .query()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "page")
                .map(|(_, v)| v.into_owned())
        })
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);

    let per_page: i64 = 12;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM youtube_videos")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let (total_pages, current, offset) = crate::util::paginate(total, page_num, per_page);

    let videos = sqlx::query_as::<_, YoutubeVideo>(
        "SELECT * FROM youtube_videos ORDER BY published_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <div style="margin-bottom:40px;">
                <div style="font-size:11px; font-weight:700; color:#DE5B30; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"VIDEOS"</div>
                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">"영상"</h1>
                <p class="section-sub">"Rust 관련 최신 영상을 확인하세요."</p>
            </div>

            if locked {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="font-size:15px; font-weight:600; margin-bottom:8px;">"접근 권한이 없습니다"</p>
                    <p style="color:#6b7280;">(lock_msg.clone())</p>
                </div>
            } else if videos.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="color:#6b7280;">"아직 영상이 없습니다."</p>
                </div>
            } else {
                <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(260px, 1fr)); gap:16px;">
                    for item in videos {
                        VideoCard(video: item)
                    }
                </div>

                if total_pages > 1 {
                    <div style="display:flex; align-items:center; justify-content:center; gap:6px; margin-top:32px;">
                        if current > 1 {
                            <a href=(format!("/youtube?page={}", current - 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">"이전"</a>
                        }
                        for p in 1..=total_pages {
                            let active = p == current;
                            let style = if active {
                                "min-width:32px; height:32px; padding:0 10px; display:inline-flex; align-items:center; justify-content:center; border-radius:8px; background:#2563EB; color:#fff; font-size:13px; font-weight:600;"
                            } else {
                                "min-width:32px; height:32px; padding:0 10px; display:inline-flex; align-items:center; justify-content:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;"
                            };
                            <a href=(format!("/youtube?page={p}")) style=(style)>(p)</a>
                        }
                        if current < total_pages {
                            <a href=(format!("/youtube?page={}", current + 1)) style="height:32px; padding:0 14px; display:inline-flex; align-items:center; border-radius:8px; border:1px solid #E4E4E7; background:#fff; color:#374151; font-size:13px; font-weight:500;">"다음"</a>
                        }
                    </div>
                }
            }
        </div>
    })
}
