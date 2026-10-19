use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, signal},
    view::{View, view},
};

use crate::pages::search_shard::search_results;

#[page("/search")]
pub async fn search_page(cx: &Cx) -> Result<impl View + '_> {
    let loc = crate::services::i18n::locale(cx);
    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px; max-width:720px;">
            <div style="margin-bottom:32px;">
                <div style="font-size:11px; font-weight:700; color:#2563EB; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"SEARCH"</div>
                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px; color:#16181d;">(loc.pick("검색", "Search"))</h1>
                <p class="section-sub">(loc.pick("매뉴얼, 영상, 게시글을 검색하세요.", "Search manuals, videos and posts."))</p>
            </div>

            let query = signal(cx, String::new);

            <div style="position:relative; margin-bottom:32px;">
                <input
                    type="text"
                    placeholder=(loc.pick("검색어를 입력하세요...", "Type to search..."))
                    class="input"
                    style="padding-left:44px; padding-top:14px; padding-bottom:14px; font-size:16px; border-radius:12px;"
                    :value=$(query.get())
                    @input=$(|e: Event| {
                        query.set(e.target.value);
                    })
                >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#9ca3af" stroke-width="2" stroke-linecap="round" style="position:absolute; left:14px; top:50%; transform:translateY(-50%);">
                    <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
                </svg>
            </div>

            search_results(query: $(query.get()))
        </div>
    })
}
