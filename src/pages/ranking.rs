use crate::db::Db;
use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct RankedUser {
    username: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    points: i64,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct RankedUserCount {
    username: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    cnt: i64,
}

fn rank_badge(i: usize) -> &'static str {
    match i {
        0 => "🥇",
        1 => "🥈",
        2 => "🥉",
        _ => "",
    }
}

#[page("/ranking")]
pub async fn ranking_page(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);

    let by_points: Vec<RankedUser> = sqlx::query_as::<_, RankedUser>(
        "SELECT username, display_name, avatar_url, points FROM users WHERE status = 'active' ORDER BY points DESC LIMIT 20"
    ).fetch_all(db).await.unwrap_or_default();

    let by_posts: Vec<RankedUserCount> = sqlx::query_as::<_, RankedUserCount>(
        "SELECT u.username, u.display_name, u.avatar_url,
                (SELECT COUNT(*) FROM posts p WHERE p.author_id = u.id) AS cnt
         FROM users u WHERE u.username != 'guest' AND u.status = 'active' ORDER BY cnt DESC LIMIT 20"
    ).fetch_all(db).await.unwrap_or_default();

    let by_comments: Vec<RankedUserCount> = sqlx::query_as::<_, RankedUserCount>(
        "SELECT u.username, u.display_name, u.avatar_url,
                (SELECT COUNT(*) FROM comments c WHERE c.author_id = u.id) AS cnt
         FROM users u WHERE u.username != 'guest' AND u.status = 'active' ORDER BY cnt DESC LIMIT 20"
    ).fetch_all(db).await.unwrap_or_default();

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <div style="margin-bottom:32px;">
                <div style="font-size:11px; font-weight:700; color:#2563EB; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"RANKING"</div>
                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em;">"사용자 랭킹"</h1>
            </div>

            <div style="margin-bottom:24px;">
                <h2 style="font-size:15px; font-weight:700; margin-bottom:16px;">"포인트 랭킹"</h2>
                <div style="background:#e5e7eb; border-radius:12px; overflow:hidden;">
                    if by_points.is_empty() {
                        <div style="padding:40px; text-align:center; background:#fff;"><p style="color:#6b7280;">"아직 데이터가 없습니다."</p></div>
                    } else {
                        for (i, u) in by_points.iter().enumerate() {
                            let name = u.display_name.as_deref().unwrap_or(&u.username);
                            <div style=(format!("display:flex; align-items:center; padding:14px 20px; background:{}; gap:12px;", if i % 2 == 0 { "#fff" } else { "#fafafa" }))>
                                <span style=(if i < 3 { "font-size:18px;" } else { "font-size:13px; color:#9ca3af; min-width:24px; text-align:center;" })>(rank_badge(i))</span>
                                if i >= 3 {
                                    <span style="font-size:13px; color:#9ca3af; min-width:24px; text-align:center;">(i + 1)</span>
                                }
                                <span style="font-size:14px; font-weight:500; flex:1;">(name)</span>
                                <span style="font-size:14px; font-weight:700; color:#2563EB;">(format!("{} P", u.points))</span>
                            </div>
                        }
                    }
                </div>
            </div>

            <div style="display:grid; grid-template-columns:1fr 1fr; gap:24px;">
                <div>
                    <h2 style="font-size:15px; font-weight:700; margin-bottom:16px;">"게시글 랭킹"</h2>
                    <div style="background:#e5e7eb; border-radius:12px; overflow:hidden;">
                        for (i, u) in by_posts.iter().enumerate() {
                            let name = u.display_name.as_deref().unwrap_or(&u.username);
                            <div style=(format!("display:flex; align-items:center; padding:12px 16px; background:{}; gap:10px;", if i % 2 == 0 { "#fff" } else { "#fafafa" }))>
                                <span style="font-size:13px; color:#9ca3af; min-width:20px; text-align:center;">(i + 1)</span>
                                <span style="font-size:13px; font-weight:500; flex:1;">(name)</span>
                                <span style="font-size:13px; font-weight:700; color:#059669;">(format!("{}개", u.cnt))</span>
                            </div>
                        }
                    </div>
                </div>
                <div>
                    <h2 style="font-size:15px; font-weight:700; margin-bottom:16px;">"댓글 랭킹"</h2>
                    <div style="background:#e5e7eb; border-radius:12px; overflow:hidden;">
                        for (i, u) in by_comments.iter().enumerate() {
                            let name = u.display_name.as_deref().unwrap_or(&u.username);
                            <div style=(format!("display:flex; align-items:center; padding:12px 16px; background:{}; gap:10px;", if i % 2 == 0 { "#fff" } else { "#fafafa" }))>
                                <span style="font-size:13px; color:#9ca3af; min-width:20px; text-align:center;">(i + 1)</span>
                                <span style="font-size:13px; font-weight:500; flex:1;">(name)</span>
                                <span style="font-size:13px; font-weight:700; color:#7C3AED;">(format!("{}개", u.cnt))</span>
                            </div>
                        }
                    </div>
                </div>
            </div>
        </div>
    })
}
