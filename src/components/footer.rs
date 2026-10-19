use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

#[component]
pub async fn Footer(cx: &Cx) -> Result<impl View + '_> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let loc = crate::services::i18n::locale(cx);
    let site_title = crate::services::site_settings::get_localized(db, "site_title", loc).await;
    let site_title = if site_title.is_empty() {
        "Rust Korea Hub".to_string()
    } else {
        site_title
    };
    let site_description =
        crate::services::site_settings::get_localized(db, "site_description", loc).await;
    let site_description = if site_description.is_empty() {
        loc.pick(
            "Rust 몰라도 10분 만에 끝내는 고성능 풀스택 앱 개발 — Rust Korea",
            "Build high-performance full-stack apps in 10 minutes, even without knowing Rust — Rust Korea",
        )
        .to_string()
    } else {
        site_description
    };
    let footer_text = crate::services::site_settings::get_localized(db, "footer_text", loc).await;
    let footer_text = if footer_text.is_empty() {
        "2026 Rust Korea. Built with Rust and Topcoat.".to_string()
    } else {
        footer_text
    };
    let logo_url = crate::services::site_settings::get_setting(db, "logo_url").await;

    let access = crate::auth::access::load_access(db, cx).await;
    let items = crate::services::navigation::visible_items(db, "footer", access.as_ref()).await?;

    Ok(view! {
        <footer class="desktop-footer" style="border-top:1px solid #e5e7eb; margin-top:80px; background:#f8f9fa;">
            <div class="container" style="padding:48px 24px;">
                <div class="mobile-footer-links" style="display:grid; grid-template-columns:1.5fr 1fr 1fr 1fr; gap:40px; margin-bottom:40px;">
                    <div>
                        <div style="display:flex; align-items:center; gap:8px; font-weight:800; font-size:15px; margin-bottom:10px; color:#16181d;">
                            if !logo_url.is_empty() {
                                <img src=(logo_url.clone()) alt="logo" style="width:24px; height:24px; border-radius:4px; object-fit:contain;">
                            } else {
                                <img src="/logo.svg" alt="Rust logo" width="18" height="18" style="object-fit:contain;">
                            }
                            (site_title.clone())
                        </div>
                        <p style="font-size:13px; color:#6b7280; line-height:1.6;">(site_description.clone())</p>
                    </div>
                    for (group, _) in crate::services::navigation::FOOTER_GROUPS {
                        if items.iter().any(|item| item.footer_group == group) {
                            let label = crate::services::navigation::footer_group_label(group, loc);
                            <nav aria-label=(format!("{} {label}", loc.pick("푸터", "Footer")))>
                                <h4 style="font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:14px;">(label)</h4>
                                <ul style="list-style:none; display:flex; flex-direction:column; gap:10px;">
                                    for item in items.iter().filter(|item| item.footer_group == group) {
                                        <li><a href=(item.href.clone()) style="font-size:14px; color:#6b7280;" target=(if item.href.starts_with('/') { "_self" } else { "_blank" }) rel="noopener noreferrer">(loc.text_owned(&item.label, &item.label_en))</a></li>
                                    }
                                </ul>
                            </nav>
                        }
                    }
                </div>
                <div class="divider" style="margin-bottom:24px;"></div>
                <p style="font-size:13px; color:#9ca3af; text-align:center;">(footer_text.clone())</p>
            </div>
        </footer>

        <footer class="mobile-footer" style="display:none; border-top:1px solid #e5e7eb; margin-top:24px; padding:24px 16px 16px; background:#f8f9fa;">
            <nav aria-label=(loc.pick("모바일 푸터", "Mobile footer")) style="display:flex; flex-wrap:wrap; gap:16px 24px; margin-bottom:16px;">
                for (group, _label) in crate::services::navigation::FOOTER_GROUPS {
                    for item in items.iter().filter(|item| item.footer_group == group) {
                        <a href=(item.href.clone()) style="font-size:13px; color:#6b7280; text-decoration:none;" target=(if item.href.starts_with('/') { "_self" } else { "_blank" }) rel="noopener noreferrer">(loc.text_owned(&item.label, &item.label_en))</a>
                    }
                }
            </nav>
            <p style="font-size:12px; color:#9ca3af; text-align:center;">(footer_text)</p>
        </footer>
    })
}
