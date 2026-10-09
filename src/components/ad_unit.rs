use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::db::Db;
use topcoat::cookie::{Cookies, cookies};

#[component]
pub async fn AdUnit(cx: &Cx, slot_id: String) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let enabled = crate::services::site_settings::get_setting(db, "adsense_enabled").await;
    let client_id = crate::services::site_settings::get_setting(db, "adsense_client").await;
    let consented = cookies(cx)
        .get("cookie_consent")
        .map(|c| c.value() == "1")
        .unwrap_or(false);

    let is_active = enabled == "true" && !client_id.is_empty() && consented;
    let slot = slot_id.clone();
    let client = client_id.clone();

    Ok(view! {
        if is_active {
            <div style="margin:32px 0; text-align:center; min-height:100px; display:flex; align-items:center; justify-content:center;">
                <ins class="adsbygoogle"
                    data-ad-client=(client)
                    data-ad-slot=(slot)
                    data-ad-format="auto"
                    data-full-width-responsive="true"
                    style="display:block;">
                </ins>
                <script>{r#"(adsbygoogle = window.adsbygoogle || []).push({});"#}</script>
            </div>
        }
    })
}
