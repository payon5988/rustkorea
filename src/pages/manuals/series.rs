use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::db::Db;
use crate::models::ManualSeries;

#[page("/manuals")]
pub async fn manuals_index(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    // All series are listed (titles public); per-series/chapter content is
    // gated at the detail pages. Non-public series get a lock badge.
    let series =
        sqlx::query_as::<_, ManualSeries>("SELECT * FROM manual_series ORDER BY created_at DESC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <div style="margin-bottom:40px;">
                <div style="font-size:11px; font-weight:700; color:#DE5B30; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"MANUALS"</div>
                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">"매뉴얼"</h1>
                <p class="section-sub">"Rust와 Topcoat를 한국어로 배워보세요."</p>
            </div>

            if series.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="color:#6b7280;">"매뉴얼이 곧 추가됩니다."</p>
                </div>
            } else {
                <div style="display:grid; grid-template-columns:repeat(2,1fr); gap:16px;">
                    for item in series {
                        <a href=(format!("/manuals/{}", item.slug)) class="card" style="padding:24px; display:block;">
                            <div style="display:flex; align-items:center; gap:10px; margin-bottom:14px;">
                                <span style="font-size:28px;">(item.thumbnail_url.clone().unwrap_or_else(|| "📚".into()))</span>
                                <span class="badge" style=((match item.difficulty.as_str() {
                                        "advanced" => "background:#fef2f2; color:#ef4444; border:1px solid #fecaca",
                                        "intermediate" => "background:#fffbeb; color:#d97706; border:1px solid #fde68a",
                                        _ => "background:#ecfdf5; color:#10b981; border:1px solid #a7f3d0",
                                    }).to_string())>
                                    (match item.difficulty.as_str() {
                                        "advanced" => "고급",
                                        "intermediate" => "중급",
                                        _ => "초급",
                                    })
                                </span>
                                if item.visibility != "public" {
                                    <span class="badge" style="background:#eff6ff; color:#2563EB; border:1px solid #bfdbfe;">(format!("🔒 {}", crate::auth::access::visibility_label(&item.visibility)))</span>
                                }
                            </div>
                            <h3 style="font-size:16px; font-weight:700; margin-bottom:6px; letter-spacing:-0.01em;">(item.title.clone())</h3>
                            <p style="font-size:13px; color:#6b7280; line-height:1.5;">(item.description.clone())</p>
                            <div style="margin-top:14px; font-size:12px; color:#9ca3af;">
                                (format!("약 {}분", item.estimated_minutes))
                            </div>
                        </a>
                    }
                </div>
            }
        </div>
    })
}
