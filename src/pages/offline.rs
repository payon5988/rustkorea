use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

/// Offline fallback page (cached by the service worker, shown for
/// navigations when the network is unavailable).
#[page("/offline")]
pub async fn offline_page(cx: &Cx) -> Result<impl View + '_> {
    let _ = cx;
    Ok(view! {
        <div class="container" style="padding-top:80px; padding-bottom:80px; max-width:560px; text-align:center;">
            <div style="font-size:48px; margin-bottom:16px;">"📡"</div>
            <h1 style="font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:8px;">"오프라인 상태입니다"</h1>
            <p class="section-sub" style="margin-bottom:24px;">"네트워크에 연결되면 자동으로 최신 내용을 보여줍니다."</p>
            <a href="/" class="btn-primary">"홈으로 돌아가기"</a>
        </div>
    })
}
