use topcoat::{
    Result,
    context::Cx,
    router::{page, path_param},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::{ManualChapter, ManualSeries};

topcoat::router::path_param!(series_slug);

#[page("/admin/manuals/{series_slug}")]
pub async fn admin_manual_detail(cx: &Cx) -> Result<impl View + '_> {
    let series_slug = path_param::<SeriesSlug>(cx);

    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::MANUALS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let series = sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE slug = $1")
        .bind(series_slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    match series {
        Some(s) => {
            let chapters = sqlx::query_as::<_, ManualChapter>(
                "SELECT * FROM manual_chapters WHERE series_id = $1 ORDER BY chapter_index ASC",
            )
            .bind(&s.id)
            .fetch_all(db)
            .await
            .unwrap_or_default();

            Ok(view! {
                <html lang="ko">
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>{(format!("{} - Rust Korea Admin", s.title))}</title>
                    <style>{(ADMIN_STYLE)}</style>
                    <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
                </head>
                <body style="display:flex; min-height:100vh;">
                    {AdminSidebar(active: "/admin/manuals".into())}
                    <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                        <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                            <div>
                                <a href="/admin/manuals" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="15 18 9 12 15 6"/></svg>
                                    "매뉴얼 목록"
                                </a>
                                <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">(s.title.clone())</h1>
                                <p style="font-size:13px; color:var(--muted2); margin-top:2px;">(s.description.clone())</p>
                            </div>
                            <div style="display:flex; gap:8px;">
                                <a href=(format!("/admin/manuals/{}/chapters/new", s.slug.clone())) style="background:var(--card); color:var(--text3); padding:10px 16px; border-radius:8px; font-size:13px; border:1px solid var(--border);">"+ 새 챕터"</a>
                                <a href=(format!("/admin/manuals/edit?id={}", s.id)) style="background:var(--card); color:var(--amber); padding:10px 16px; border-radius:8px; font-size:13px; border:1px solid var(--border);">"시리즈 수정"</a>
                            </div>
                        </div>

                        <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                            <div style="padding:14px 20px; border-bottom:1px solid var(--border);">
                                <h2 style="font-size:14px; font-weight:600; color:var(--text3);">"챕터 목록"</h2>
                            </div>
                            if chapters.is_empty() {
                                <div style="padding:48px; text-align:center;">
                                    <p style="font-size:13px; color:var(--muted2); margin-bottom:12px;">"등록된 챕터가 없습니다"</p>
                                    <a href=(format!("/admin/manuals/{}/chapters/new", s.slug)) style="font-size:13px; color:#DE5B30; font-weight:600;">"첫 챕터 추가"</a>
                                </div>
                            } else {
                                for ch in &chapters {
                                    <div style="padding:14px 20px; border-bottom:1px solid var(--line); display:flex; align-items:center; justify-content:space-between;">
                                        <div>
                                            <div style="display:flex; align-items:center; gap:10px;">
                                                <span style="font-size:11px; color:var(--dim); font-family:monospace;">(format!("{:02}", ch.chapter_index))</span>
                                                <span style="font-size:14px; font-weight:600; color:var(--text3);">(ch.title.clone())</span>
                                                <span class="badge" style="background:rgba(96,165,250,0.1); color:var(--blue);">(crate::auth::access::visibility_label(&ch.visibility))</span>
                                                if !crate::util::is_released(ch.published_at.as_deref()) {
                                                    <span style="font-size:11px; font-weight:600; color:#b45309;">" 예약됨"</span>
                                                }
                                            </div>
                                            <p style="font-size:12px; color:var(--dim); margin-top:3px; font-family:monospace;">(ch.slug.clone())</p>
                                        </div>
                                        <div style="display:flex; gap:8px; align-items:center;">
                                            <a href=(format!("/admin/manuals/chapters/edit?id={}", ch.id)) style="font-size:12px; color:var(--amber);">"수정"</a>
                                            <form action="/admin/manuals/chapters/delete" method="POST" onsubmit="return confirm('이 챕터를 삭제하시겠습니까?')" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="id" value=(ch.id.clone())>
                                                <input type="hidden" name="series_slug" value=(s.slug.clone())>
                                                <button type="submit" style="background:none; border:none; padding:0; font-size:12px; color:var(--red); cursor:pointer; font-family:inherit;">"삭제"</button>
                                            </form>
                                        </div>
                                    </div>
                                }
                            }
                        </div>
                    </div>
                </body>
                </html>
            })
        }
        None => Err(topcoat::router::error::redirect("/admin/manuals").into()),
    }
}
