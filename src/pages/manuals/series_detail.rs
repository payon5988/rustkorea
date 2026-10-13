use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::{page, path_param},
    view::{View, view},
};

use crate::db::Db;
use crate::models::ManualSeries;

topcoat::router::path_param!(slug);

#[page("/manuals/{slug}")]
pub async fn series_detail(cx: &Cx) -> Result<impl View + '_> {
    let slug = path_param::<Slug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;

    let series = sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series WHERE slug = $1")
        .bind(slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    // Series page itself is always visible (titles public). Each chapter is
    // gated by its own visibility tier; locked chapters show a lock row.
    let chapters = if let Some(ref s) = series {
        sqlx::query_as::<_, crate::models::ManualChapter>(
            "SELECT * FROM manual_chapters WHERE series_id = $1 ORDER BY chapter_index",
        )
        .bind(&s.id)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };
    let chapter_gates: Vec<crate::auth::access::Gate> = chapters
        .iter()
        .map(|c| {
            crate::auth::access::visibility_gate(
                access.as_ref(),
                &c.visibility,
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS,
            )
        })
        .collect();

    let current_user = crate::auth::session::current_user(db, cx).await;
    let done_count: usize = if let (Some(u), true) = (current_user.as_ref(), !chapters.is_empty()) {
        let ids: Vec<String> = chapters.iter().map(|c| c.id.clone()).collect();
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM reading_progress WHERE user_id = $1 AND completed = 1 AND chapter_id = ANY($2)",
        )
        .bind(&u.id)
        .bind(&ids)
        .fetch_one(db)
        .await
        .unwrap_or(0) as usize
    } else {
        0
    };
    let total_count = chapters.len();
    let show_progress = current_user.is_some() && total_count > 0;
    let progress_pct: usize = match total_count {
        0 => 0,
        n => done_count * 100 / n,
    };

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px;">
            if let Some(s) = series {
                <a href="/manuals" style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">
                    "<- 매뉴얼 목록"
                </a>

                <div style="margin-bottom:40px;">
                    <div style="display:flex; align-items:center; gap:10px; margin-bottom:10px;">
                        <span style="font-size:28px;">(s.thumbnail_url.clone().unwrap_or_else(|| "📚".into()))</span>
                        <span class="badge" style=((match s.difficulty.as_str() {
                                crate::models::difficulty::ADVANCED => "background:#fef2f2; color:#ef4444; border:1px solid #fecaca",
                                crate::models::difficulty::INTERMEDIATE => "background:#fffbeb; color:#d97706; border:1px solid #fde68a",
                                _ => "background:#ecfdf5; color:#10b981; border:1px solid #a7f3d0",
                            }).to_string())>
                            (crate::models::difficulty::label(s.difficulty.as_str()))
                        </span>
                        if s.visibility != "public" {
                            <span class="badge" style="background:#eff6ff; color:#2563EB; border:1px solid #bfdbfe;">(format!("🔒 {}", crate::auth::access::visibility_label(&s.visibility)))</span>
                        }
                    </div>
                    <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">(s.title.clone())</h1>
                    <p class="section-sub" style="margin-bottom:16px;">(s.description.clone())</p>
                    <div style="display:flex; gap:20px; font-size:13px; color:#9ca3af;">
                        <span>(format!("약 {}분", s.estimated_minutes))</span>
                        <span>(format!("{}개 챕터", chapters.len()))</span>
                    </div>
                    if show_progress {
                        <div style="margin-top:12px; max-width:320px;">
                            <div style="display:flex; justify-content:space-between; font-size:12px; color:#6b7280; margin-bottom:6px;">
                                <span>"내 진행률"</span>
                                <span>(format!("{done_count}/{total_count} ({progress_pct}%)"))</span>
                            </div>
                            <div style="height:8px; border-radius:4px; background:#f3f4f6; overflow:hidden;">
                                <div style=(format!("height:100%; width:{progress_pct}%; background:#2563EB; border-radius:4px; transition:width 0.3s;"))></div>
                            </div>
                        </div>
                    }
                </div>

                if chapters.is_empty() {
                    <div class="card" style="padding:60px; text-align:center;">
                        <p style="color:#6b7280;">"챕터가 준비 중입니다."</p>
                    </div>
                } else {
                    <div style="display:flex; flex-direction:column; gap:8px;">
                        for (i, chapter) in chapters.iter().enumerate() {
                            if chapter_gates[i] == crate::auth::access::Gate::Allow {
                                <a href=(format!("/manuals/{}/{}", s.slug, chapter.slug)) class="card" style="display:flex; align-items:center; gap:16px; padding:16px 20px;">
                                    <span style="width:32px; height:32px; border-radius:8px; background:#f9fafb; border:1px solid #f3f4f6; display:flex; align-items:center; justify-content:center; font-size:13px; font-weight:600; color:#6b7280; flex-shrink:0;">(i + 1)</span>
                                    <span style="font-size:14px; font-weight:600; flex:1;">(chapter.title.clone())</span>
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#9ca3af" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
                                </a>
                            } else if chapter_gates[i] == crate::auth::access::Gate::LoginRequired {
                                <a href=(format!("/auth/login?redirect=/manuals/{}/{}", s.slug, chapter.slug)) class="card" style="display:flex; align-items:center; gap:16px; padding:16px 20px; background:#f8fafb;">
                                    <span style="width:32px; height:32px; border-radius:8px; background:#eef2ff; border:1px solid #bfdbfe; display:flex; align-items:center; justify-content:center; font-size:13px; font-weight:600; color:#2563EB; flex-shrink:0;">"🔒"</span>
                                    <span style="font-size:14px; font-weight:600; flex:1;">(chapter.title.clone())</span>
                                    <span style="font-size:11px; color:#2563EB; font-weight:600;">"로그인 필요"</span>
                                </a>
                            } else {
                                <div class="card" style="display:flex; align-items:center; gap:16px; padding:16px 20px; background:#f8fafb; opacity:0.75;">
                                    <span style="width:32px; height:32px; border-radius:8px; background:#fef2f2; border:1px solid #fecaca; display:flex; align-items:center; justify-content:center; font-size:13px; font-weight:600; color:#ef4444; flex-shrink:0;">"🔒"</span>
                                    <span style="font-size:14px; font-weight:600; flex:1;">(chapter.title.clone())</span>
                                    <span style="font-size:11px; color:#6b7280; font-weight:600;">(crate::auth::access::visibility_label(&chapter.visibility))</span>
                                </div>
                            }
                        }
                    </div>
                }
            } else {
                <div style="text-align:center; padding:100px 0;">
                    <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"시리즈를 찾을 수 없습니다"</h1>
                    <p style="color:#6b7280; margin-bottom:24px;">"요청하신 매뉴얼이 존재하지 않습니다."</p>
                    <a href="/manuals" class="btn-primary" style="display:inline-flex;">"매뉴얼 목록으로"</a>
                </div>
            }
        </div>
    })
}
