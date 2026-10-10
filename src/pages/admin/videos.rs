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
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/videos".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"YouTube 영상 관리"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">(format!("{}개 영상 동기화됨", videos.len()))</p>
                </div>

                if videos.is_empty() {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:64px; text-align:center;">
                        <p style="font-size:14px; color:var(--muted2); margin-bottom:8px;">"동기화된 영상이 없습니다"</p>
                        <p style="font-size:12px; color:var(--dim);">"YouTube 워커가 자동으로 동기화합니다"</p>
                    </div>
                } else {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead>
                                <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"썸네일"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"제목"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"날짜"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"태그"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for v in &videos {
                                    <tr style="border-bottom:1px solid var(--line);">
                                        <td style="padding:12px 20px;">
                                            if v.thumbnail_url.is_empty() {
                                                <div style="width:80px; height:45px; background:var(--thumb); border-radius:6px; display:flex; align-items:center; justify-content:center;">
                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--dim)" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                                                </div>
                                            } else {
                                                <img src=(v.thumbnail_url.clone()) alt="" style="width:80px; height:45px; object-fit:cover; border-radius:6px;">
                                            }
                                        </td>
                                        <td style="padding:12px 20px;">
                                            <a href=(format!("https://www.youtube.com/watch?v={}", v.video_id)) style="font-weight:600; color:var(--text3);">(v.title.clone())</a>
                                        </td>
                                        <td style="padding:12px 20px; color:var(--dim); font-size:12px;">(v.published_at.get(..10).unwrap_or("").to_string())</td>
                                        <td style="padding:12px 20px;">
                                            <div style="display:flex; gap:4px; flex-wrap:wrap;">
                                                for tag in v.tags_parsed() {
                                                    <span class="badge" style="background:var(--thumb); color:var(--muted);">(tag)</span>
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
