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

#[page("/admin/manuals")]
pub async fn admin_manuals(cx: &Cx) -> Result<impl View + '_> {
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

    let series =
        sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series ORDER BY created_at DESC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    let mut series_with_chapters = Vec::new();
    for s in series {
        let chapters: Vec<crate::models::ManualChapter> = sqlx::query_as(
            "SELECT * FROM manual_chapters WHERE series_id = $1 ORDER BY chapter_index ASC",
        )
        .bind(&s.id)
        .fetch_all(db)
        .await
        .unwrap_or_default();
        series_with_chapters.push((s, chapters));
    }

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"매뉴얼 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/manuals".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"매뉴얼 관리"</h1>
                        <p style="font-size:13px; color:var(--muted2); margin-top:2px;">(format!("{}개 시리즈 등록됨", series_with_chapters.len()))</p>
                    </div>
                    <a href="/admin/manuals/new" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-weight:600; font-size:14px; display:inline-flex; align-items:center; gap:8px;">"+ 새 매뉴얼"</a>
                </div>

                if series_with_chapters.is_empty() {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:64px; text-align:center;">
                        <p style="font-size:14px; color:var(--muted2); margin-bottom:16px;">"등록된 매뉴얼이 없습니다"</p>
                        <a href="/admin/manuals/new" style="font-size:13px; color:#DE5B30; font-weight:600;">"첫 매뉴얼 만들기"</a>
                    </div>
                } else {
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead>
                                <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"제목"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"슬러그"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"챕터"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"난이도"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"시간"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"공개범위"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"관리"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for (s, chapters) in &series_with_chapters {
                                    <tr style="border-bottom:1px solid var(--line);">
                                        <td style="padding:14px 20px; font-weight:600; color:var(--text3);">
                                            <a href=(format!("/admin/manuals/{}", s.slug)) style="color:var(--text3); text-decoration:none;">(s.title.clone())</a>
                                        </td>
                                        <td style="padding:14px 20px; color:var(--dim); font-family:monospace; font-size:12px;">(s.slug.clone())</td>
                                        <td style="padding:14px 20px; color:var(--muted);">(chapters.len())</td>
                                        <td style="padding:14px 20px;">
                                            <span class="badge" style=((match s.difficulty.as_str() {
                                                    crate::models::difficulty::ADVANCED => "background:rgba(248,113,113,0.1); color:var(--red);",
                                                    crate::models::difficulty::INTERMEDIATE => "background:rgba(251,191,36,0.1); color:var(--amber);",
                                                    _ => "background:rgba(52,211,153,0.1); color:var(--green);",
                                                }).to_string())>(s.difficulty.clone())</span>
                                        </td>
                                        <td style="padding:14px 20px; color:var(--muted2); font-size:12px;">(format!("{}분", s.estimated_minutes))</td>
                                        <td style="padding:14px 20px;">
                                            <span class="badge" style="background:rgba(96,165,250,0.1); color:var(--blue);">(crate::auth::access::visibility_label(&s.visibility))</span>
                                        </td>
                                        <td style="padding:14px 20px; text-align:right;">
                                            <div style="display:flex; gap:8px; justify-content:flex-end; align-items:center;">
                                                <a href=(format!("/manuals/{}", s.slug)) style="font-size:12px; color:var(--muted2);">"보기"</a>
                                                <a href=(format!("/admin/manuals/{}/chapters/new", s.slug)) style="font-size:12px; color:#DE5B30; font-weight:600;">"챕터 추가"</a>
                                                <a href=(format!("/admin/manuals/edit?id={}", s.id)) style="font-size:12px; color:var(--amber);">"수정"</a>
                                                <form action="/admin/manuals/delete" method="POST" onsubmit="return confirm('이 매뉴얼과 모든 챕터를 삭제하시겠습니까?')" style="display:inline;">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <input type="hidden" name="id" value=(s.id.clone())>
                                                    <button type="submit" style="background:none; border:none; padding:0; font-size:12px; color:var(--red); cursor:pointer; font-family:inherit;">"삭제"</button>
                                                </form>
                                            </div>
                                        </td>
                                    </tr>
                                    <tr style="border-bottom:1px solid var(--line-strong);">
                                        <td colspan="7" style="padding:0 20px 14px 40px; background:var(--line);">
                                            <details>
                                                <summary style="font-size:12px; color:var(--muted); cursor:pointer; padding:8px 0;">(format!("챕터 {}개 (펼치기)", chapters.len()))</summary>
                                                <div style="display:flex; flex-direction:column; gap:6px; padding:8px 0 4px;">
                                                    for ch in chapters {
                                                        <div style="display:flex; align-items:center; gap:10px; font-size:12px;">
                                                            <span style="color:var(--dim); font-family:monospace; width:24px;">(ch.chapter_index)</span>
                                                            <a href=(format!("/admin/manuals/chapters/edit?id={}", ch.id)) style="color:var(--text3); font-weight:600; text-decoration:none;">(ch.title.clone())</a>
                                                            <span style="color:var(--dim); font-family:monospace;">(ch.slug.clone())</span>
                                                            <span style="font-size:11px; color:var(--blue);">(crate::auth::access::visibility_label(&ch.visibility))</span>
                                                            <span style="display:inline-flex; gap:4px; margin-left:auto;">
                                                                <form action="/admin/manuals/chapters/move" method="POST" style="display:inline;">
                                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                                    <input type="hidden" name="id" value=(ch.id.clone())>
                                                                    <input type="hidden" name="direction" value="up">
                                                                    <button type="submit" style="background:none; border:none; color:var(--muted2); cursor:pointer; font-size:12px; padding:2px 4px;" title="위로">"▲"</button>
                                                                </form>
                                                                <form action="/admin/manuals/chapters/move" method="POST" style="display:inline;">
                                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                                    <input type="hidden" name="id" value=(ch.id.clone())>
                                                                    <input type="hidden" name="direction" value="down">
                                                                    <button type="submit" style="background:none; border:none; color:var(--muted2); cursor:pointer; font-size:12px; padding:2px 4px;" title="아래로">"▼"</button>
                                                                </form>
                                                                <a href=(format!("/admin/manuals/chapters/edit?id={}", ch.id)) style="font-size:12px; color:var(--amber);">"수정"</a>
                                                                <form action="/admin/manuals/chapters/delete" method="POST" onsubmit="return confirm('이 챕터를 삭제하시겠습니까?')" style="display:inline;">
                                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                                    <input type="hidden" name="id" value=(ch.id.clone())>
                                                                    <input type="hidden" name="series_slug" value=(s.slug.clone())>
                                                                    <button type="submit" style="background:none; border:none; padding:0; font-size:12px; color:var(--red); cursor:pointer; font-family:inherit;">"삭제"</button>
                                                                </form>
                                                            </span>
                                                        </div>
                                                    }
                                                </div>
                                            </details>
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
