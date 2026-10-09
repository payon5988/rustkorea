use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::YoutubeVideo;

#[page("/admin/videos")]
pub async fn admin_videos(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "youtube.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);

    let videos = sqlx::query_as::<_, YoutubeVideo>(
        "SELECT * FROM youtube_videos ORDER BY published_at DESC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"YouTube 영상 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/videos".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"YouTube 영상 관리"</h1>
                    <p style="font-size:13px; color:#6b7280; margin-top:2px;">(format!("{}개 영상 동기화됨", videos.len()))</p>
                </div>

                if videos.is_empty() {
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:64px; text-align:center;">
                        <p style="font-size:14px; color:#6b7280; margin-bottom:8px;">"동기화된 영상이 없습니다"</p>
                        <p style="font-size:12px; color:#4b5563;">"YouTube 워커가 자동으로 동기화합니다"</p>
                    </div>
                } else {
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead>
                                <tr style="border-bottom:1px solid #2a2d3a; text-align:left;">
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"썸네일"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"제목"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"날짜"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"태그"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for v in &videos {
                                    <tr style="border-bottom:1px solid rgba(255,255,255,0.03);">
                                        <td style="padding:12px 20px;">
                                            if v.thumbnail_url.is_empty() {
                                                <div style="width:80px; height:45px; background:#27262b; border-radius:6px; display:flex; align-items:center; justify-content:center;">
                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#4b5563" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                                                </div>
                                            } else {
                                                <img src=(v.thumbnail_url.clone()) alt="" style="width:80px; height:45px; object-fit:cover; border-radius:6px;">
                                            }
                                        </td>
                                        <td style="padding:12px 20px;">
                                            <a href=(format!("https://www.youtube.com/watch?v={}", v.video_id)) style="font-weight:600; color:#d1d5db;">(v.title.clone())</a>
                                        </td>
                                        <td style="padding:12px 20px; color:#4b5563; font-size:12px;">(v.published_at.get(..10).unwrap_or("").to_string())</td>
                                        <td style="padding:12px 20px;">
                                            <div style="display:flex; gap:4px; flex-wrap:wrap;">
                                                for tag in v.tags_parsed() {
                                                    <span class="badge" style="background:#27262b; color:#9ca3af;">(tag)</span>
                                                }
                                            </div>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                }
            </div>
        </body>
        </html>
    })
}
