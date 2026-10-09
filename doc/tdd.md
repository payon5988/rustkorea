# TDD: Rust Korea Hub

## 1. Topcoat 특화 아키텍처 패턴
### 1.1 모듈 기반 라우팅 및 레이아웃
```rust
// src/app.rs (루트 레이아웃)
#[page("/")]
pub async fn root_layout(cx: &Cx) -> Result {
    view! {
        <!DOCTYPE html>
        <html lang="ko" class="dark">
            <head>
                <title>"Rust Korea Hub"</title>
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                {topcoat::tailwind::stylesheet!()}
            </head>
            <body class="bg-rust-dark text-rust-light antialiased">
                <Header />
                <main class="container mx-auto px-4 py-8">
                    {cx.children()} // 하위 페이지 콘텐츠 주입
                </main>
                <Footer />
            </body>
        </html>
    }
}
use topcoat::{view::{component, view, signal}, Result};

#[component]
pub async fn ManualSearch(cx: &Cx) -> Result {
    view! {
        // 클라이언트 사이드 시그널
        signal query = String::new();
        
        <div class="relative">
            <input 
                type="text" 
                placeholder="매뉴얼 또는 Crate 검색..."
                class="w-full bg-rust-gray-800 border border-rust-orange/30 rounded-lg px-4 py-2 focus:ring-2 focus:ring-rust-orange"
                // 입력 시 시그널 업데이트 (순수 클라이언트 실행)
                @input=$(|e: web_sys::Event| {
                    let target: web_sys::HtmlInputElement = e.target().unwrap().into();
                    query.set(target.value());
                })
            />
        </div>

        // query 시그널이 변경될 때마다 서버에서 이 샤드만 재렌더링하여 교체
        <div class="mt-4">
            {SearchResults(query: $(query.get()))}
        </div>
    }
}

#[memoize]
async fn fetch_search_results(cx: &Cx, q: &str) -> Result<Vec<ManualItem>> {
    if q.is_empty() { return Ok(vec![]); }
    // Meilisearch 또는 PostgreSQL LIKE 쿼리 실행 (요청당 1회 캐싱)
    db(cx).search_manuals(q).await
}

#[shard]
pub async fn SearchResults(cx: &Cx, query: String) -> Result {
    let items = fetch_search_results(cx, &query).await?;
    view! {
        <ul class="space-y-2">
            for item in items {
                <li class="p-3 bg-rust-gray-800 rounded hover:bg-rust-gray-700 transition">
                    <a href=(item.url) class="text-rust-orange font-semibold">{item.title}</a>
                    <p class="text-sm text-gray-400">{item.summary}</p>
                </li>
            }
        </ul>
    }
}

3. 외부 API 연동 명세
YouTube Data API v3: 할당량 절약을 위해 search.list 대신 채널별 activities 또는 RSS(https://www.youtube.com/feeds/videos.xml?channel_id=UC...)를 reqwest + quick-xml로 파싱하여 15분 주기 크론잡 실행.

