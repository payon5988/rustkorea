use crate::models::ManualSeries;
use std::future::Future;
use topcoat::{
    Result,
    view::{View, component, view},
};

#[component]
pub async fn ManualCard(cx: &topcoat::context::Cx, series: ManualSeries) -> Result<impl View + '_> {
    let _ = cx;
    let slug = series.slug.clone();
    let emoji = series.thumbnail_url.clone().unwrap_or_default();
    let title = series.title.clone();
    let description = series.description.clone();

    let (level_text, level_class) = match series.difficulty.as_str() {
        crate::models::difficulty::ADVANCED => (
            crate::models::difficulty::label(crate::models::difficulty::ADVANCED),
            "text-[#F87171] bg-[#F87171]/10 border-[#F87171]/20",
        ),
        crate::models::difficulty::INTERMEDIATE => (
            crate::models::difficulty::label(crate::models::difficulty::INTERMEDIATE),
            "text-[#FBBF24] bg-[#FBBF24]/10 border-[#FBBF24]/20",
        ),
        _ => (
            crate::models::difficulty::label(""),
            "text-[#34D399] bg-[#34D399]/10 border-[#34D399]/20",
        ),
    };

    let display_emoji = if emoji.is_empty() {
        "📚".to_string()
    } else {
        emoji
    };

    Ok(view! {
        <a href=(format!("/manuals/{}", slug)) class="block bg-[#141416] border border-white/[0.06] rounded-xl overflow-hidden hover:border-[#DE5B30]/20 transition-colors group">
            <div class="w-full h-28 bg-gradient-to-br from-[#1C1B1A] to-[#141416] flex items-center justify-center border-b border-white/[0.04]">
                <span class="text-3xl opacity-80 group-hover:scale-110 transition-transform">(display_emoji)</span>
            </div>
            <div class="p-4">
                <div class="flex items-center gap-2 mb-2">
                    <span class=(format!("text-[10px] font-semibold px-2 py-0.5 rounded border {}", level_class))>(level_text)</span>
                </div>
                <h3 class="text-sm font-semibold text-[#F5F1EA] group-hover:text-[#DE5B30] transition-colors mb-1.5 line-clamp-1">(title)</h3>
                <p class="text-xs text-[#78716C] line-clamp-2 leading-relaxed">(description)</p>
            </div>
        </a>
    })
}
