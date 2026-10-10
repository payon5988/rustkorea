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
use crate::models::ManualSeries;

#[page("/admin/chapters/new")]
pub async fn select_series_for_chapter(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "manuals.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);

    let series =
        sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series ORDER BY title ASC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"챕터 선택 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/chapters/new".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                <div style="margin-bottom:24px;">
                    <a href="/admin" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                        "대시보드"
                    </a>
                    <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"새 챕터 - 시리즈 선택"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"챕터를 추가할 시리즈를 선택하세요"</p>
                </div>

                if series.is_empty() {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:64px; text-align:center;">
                        <p style="font-size:14px; color:var(--muted2); margin-bottom:16px;">"등록된 매뉴얼이 없습니다"</p>
                        <a href="/admin/manuals/new" style="font-size:13px; color:#DE5B30; font-weight:600;">"먼저 매뉴얼을 만드세요"</a>
                    </div>
                } else {
                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:12px; max-width:640px;">
                        for s in &series {
                            <a href=(format!("/admin/manuals/{}/chapters/new", s.slug)) style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:20px;">
                                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:8px;">
                                    <h3 style="font-size:14px; font-weight:700; color:var(--text3);">(s.title.clone())</h3>
                                    <span class="badge" style=((match s.difficulty.as_str() {
                                            "advanced" => "background:rgba(248,113,113,0.1); color:var(--red);",
                                            "intermediate" => "background:rgba(251,191,36,0.1); color:var(--amber);",
                                            _ => "background:rgba(52,211,153,0.1); color:var(--green);",
                                        }).to_string())>(s.difficulty.clone())</span>
                                </div>
                                <p style="font-size:12px; color:var(--dim); font-family:monospace;">(s.slug.clone())</p>
                            </a>
                        }
                    </div>
                }
            </div>
        </body>
        </html>
    })
}
