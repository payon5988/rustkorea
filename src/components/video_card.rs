use crate::models::YoutubeVideo;
use std::future::Future;
use topcoat::{
    Result,
    view::{View, component, view},
};

#[component]
pub async fn VideoCard(cx: &topcoat::context::Cx, video: YoutubeVideo) -> Result<impl View + '_> {
    let _ = cx;
    let thumb_url = video.thumbnail_url.clone();
    let title = video.title.clone();
    let date = video
        .published_at
        .split('T')
        .next()
        .unwrap_or("")
        .to_string();
    let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
    let has_thumb = !thumb_url.is_empty();

    Ok(view! {
        <a href=(url) target="_blank" rel="noopener" class="card" style="overflow:hidden; display:block;">
            <div style="position:relative; aspect-ratio:16/9; background:#f3f4f6;">
                if has_thumb {
                    <img src=(thumb_url) alt=(title.clone()) style="width:100%; height:100%; object-fit:cover;" loading="lazy">
                }
                <div style="position:absolute; inset:0; display:flex; align-items:center; justify-content:center;">
                    <div style="width:40px; height:40px; border-radius:50%; background:#DE5B30; display:flex; align-items:center; justify-content:center; box-shadow:0 4px 20px rgba(222,91,48,0.3);">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="white" style="margin-left:2px;"><path d="M8 5v14l11-7z"/></svg>
                    </div>
                </div>
            </div>
            <div style="padding:14px 16px;">
                <h3 style="font-size:13px; font-weight:600; line-height:1.4; margin-bottom:6px; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden;">(title)</h3>
                <div style="font-size:11px; color:#9ca3af;">(date)</div>
            </div>
        </a>
    })
}
