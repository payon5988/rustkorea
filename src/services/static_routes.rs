use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::header::{CONTENT_TYPE, HeaderValue};
use topcoat::router::{
    Body,
    response::{IntoResponse, Response},
};
use topcoat::router::{path_param, route};

pub struct TypedBody {
    pub content_type: &'static str,
    pub body: String,
}

impl IntoResponse for TypedBody {
    fn into_response(self, _cx: &Cx) -> Result<Response> {
        let ct = HeaderValue::from_static(self.content_type);
        let mut resp = Response::new(Body::from(self.body));
        resp.headers_mut().insert(CONTENT_TYPE, ct);
        Ok(resp)
    }
}

/// Binary (non-UTF8) response body, e.g. PNG icons baked in at compile time.
pub struct BinBody {
    pub content_type: &'static str,
    pub body: &'static [u8],
}

impl IntoResponse for BinBody {
    fn into_response(self, _cx: &Cx) -> Result<Response> {
        let ct = HeaderValue::from_static(self.content_type);
        let mut resp = Response::new(Body::from(self.body));
        resp.headers_mut().insert(CONTENT_TYPE, ct);
        Ok(resp)
    }
}

/// PWA manifest identity (single source; the JSON template below inlines these).
pub const MANIFEST_SHORT_NAME: &str = "RustKorea";
pub const MANIFEST_BACKGROUND: &str = "#0f1117";
pub const MANIFEST_THEME: &str = "#DE5B30";

/// Offline shell + cache policy for the service worker.
pub const SW_SHELL: [&str; 5] = ["/", "/community", "/manuals", "/search", "/offline"];
pub const SW_BYPASS_PREFIXES: [&str; 3] = ["/admin", "/auth", "/.well-known"];
pub const SW_MAX_ENTRIES: usize = 100;

/// robots.txt policy.
pub const ROBOTS_DISALLOW: [&str; 2] = ["/admin", "/auth"];

/// AdSense ads.txt DIRECT tag (Google-assigned).
pub const ADSENSE_DIRECT_TAG: &str = "f08c47fec0942fa0";

fn manifest_json_body(name: &str, desc: &str) -> String {
    format!(
        "{{\"name\":\"{name}\",\"short_name\":\"{short}\",\"id\":\"/\",\"start_url\":\"/\",\"scope\":\"/\",\"display\":\"standalone\",\"orientation\":\"portrait\",\"background_color\":\"{bg}\",\"theme_color\":\"{theme}\",\"description\":\"{desc}\",\"categories\":[\"education\",\"developer tools\"],\"shortcuts\":[{{\"name\":\"매뉴얼\",\"url\":\"/manuals\"}},{{\"name\":\"커뮤니티\",\"url\":\"/community\"}}],\"icons\":[{{\"src\":\"/icon-192.png\",\"sizes\":\"192x192\",\"type\":\"image/png\",\"purpose\":\"any\"}},{{\"src\":\"/icon-512.png\",\"sizes\":\"512x512\",\"type\":\"image/png\",\"purpose\":\"any\"}},{{\"src\":\"/maskable-512.png\",\"sizes\":\"512x512\",\"type\":\"image/png\",\"purpose\":\"maskable\"}}]}}",
        short = MANIFEST_SHORT_NAME,
        bg = MANIFEST_BACKGROUND,
        theme = MANIFEST_THEME,
    )
}

fn service_worker_body() -> String {
    let shell = SW_SHELL
        .iter()
        .map(|p| format!("'{p}'"))
        .collect::<Vec<_>>()
        .join(",");
    let bypass = SW_BYPASS_PREFIXES
        .iter()
        .map(|p| format!("p.startsWith('{p}')"))
        .collect::<Vec<_>>()
        .join("||");
    format!(
        "const CACHE_NAME='rustkorea-{ver}';const SHELL=[{shell}];self.addEventListener('install',e=>{{e.waitUntil(caches.open(CACHE_NAME).then(c=>c.addAll(SHELL)).then(()=>self.skipWaiting()))}});self.addEventListener('activate',e=>{{e.waitUntil(caches.keys().then(k=>Promise.all(k.filter(n=>n!==CACHE_NAME).map(n=>caches.delete(n)))).then(()=>self.clients.claim()))}});self.addEventListener('fetch',e=>{{if(e.request.method!=='GET')return;const u=new URL(e.request.url);if(u.origin!==location.origin)return;const p=u.pathname;if({bypass})return;const nav=e.request.mode==='navigate';e.respondWith(fetch(e.request).then(r=>{{const c=r.clone();caches.open(CACHE_NAME).then(cache=>{{cache.put(e.request,c);cache.keys().then(ks=>{{if(ks.length>{max})cache.delete(ks[0])}})}});return r}}).catch(()=>caches.match(e.request).then(m=>m||(nav?caches.match('/offline'):undefined))))}});",
        ver = crate::config::app_version(),
        max = SW_MAX_ENTRIES,
    )
}

#[route(GET "/manifest.json")]
pub async fn manifest_json(cx: &Cx) -> Result<TypedBody> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let name = crate::services::site_settings::get_setting(db, "site_title").await;
    let name = if name.is_empty() {
        "Rust Korea Hub".to_string()
    } else {
        name
    };
    let desc = crate::services::site_settings::get_setting(db, "site_description").await;
    let desc = if desc.is_empty() {
        "한국 Rust 생태계의 허브".to_string()
    } else {
        desc
    };
    Ok(TypedBody {
        content_type: "application/manifest+json",
        body: manifest_json_body(&name, &desc),
    })
}

#[route(GET "/logo.svg")]
pub async fn rust_logo(_cx: &Cx) -> Result<TypedBody> {
    Ok(TypedBody {
        content_type: "image/svg+xml",
        body: include_str!("../../public/icons/rust-logo.svg").to_string(),
    })
}

#[route(GET "/icon-192.png")]
pub async fn icon_192_png(_cx: &Cx) -> Result<BinBody> {
    Ok(BinBody {
        content_type: "image/png",
        body: include_bytes!("../../public/icons/icon-192.png"),
    })
}

#[route(GET "/icon-512.png")]
pub async fn icon_512_png(_cx: &Cx) -> Result<BinBody> {
    Ok(BinBody {
        content_type: "image/png",
        body: include_bytes!("../../public/icons/icon-512.png"),
    })
}

#[route(GET "/maskable-512.png")]
pub async fn icon_maskable_png(_cx: &Cx) -> Result<BinBody> {
    Ok(BinBody {
        content_type: "image/png",
        body: include_bytes!("../../public/icons/maskable-512.png"),
    })
}

#[route(GET "/apple-touch-icon.png")]
pub async fn apple_touch_icon(_cx: &Cx) -> Result<BinBody> {
    Ok(BinBody {
        content_type: "image/png",
        body: include_bytes!("../../public/icons/apple-touch-icon.png"),
    })
}

#[route(GET "/og-image.png")]
pub async fn og_image(_cx: &Cx) -> Result<BinBody> {
    Ok(BinBody {
        content_type: "image/png",
        body: include_bytes!("../../public/og-image.png"),
    })
}

#[route(GET "/vendor/toastui-editor.min.css")]
pub async fn vendor_toast_css(_cx: &Cx) -> Result<TypedBody> {
    Ok(TypedBody {
        content_type: "text/css; charset=utf-8",
        body: include_str!("../../public/vendor/toastui-editor.min.css").to_string(),
    })
}

#[route(GET "/vendor/toastui-editor-all.min.js")]
pub async fn vendor_toast_js(_cx: &Cx) -> Result<TypedBody> {
    Ok(TypedBody {
        content_type: "application/javascript; charset=utf-8",
        body: include_str!("../../public/vendor/toastui-editor-all.min.js").to_string(),
    })
}

topcoat::router::path_param!(upload_name);

/// Runtime uploads (product images) from `UPLOAD_DIR`. Only serves files
/// whose names match the generated pattern (`[A-Za-z0-9_-]+.(png|jpg|jpeg|webp|gif)`);
/// everything else is 404 (no traversal possible).
#[route(GET "/uploads/{upload_name}")]
pub async fn uploaded_file(cx: &Cx) -> Result<BinBody> {
    let name = path_param::<UploadName>(cx);
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !name.contains("..")
        && ["png", "jpg", "jpeg", "webp", "gif"]
            .iter()
            .any(|ext| name.to_lowercase().ends_with(&format!(".{ext}")));
    if !valid {
        return Err(topcoat::router::error::not_found().into());
    }
    let lowered = name.to_lowercase();
    let content_type = if lowered.ends_with(".png") {
        "image/png"
    } else if lowered.ends_with(".gif") {
        "image/gif"
    } else if lowered.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };
    let path = std::path::Path::new(&crate::config::upload_dir()).join(name);
    let bytes = std::fs::read(&path).map_err(|_| topcoat::router::error::not_found())?;
    if bytes.len() > crate::services::uploads::MAX_UPLOAD_BYTES {
        return Err(topcoat::router::error::not_found().into());
    }
    Ok(BinBody {
        content_type,
        body: Box::leak(bytes.into_boxed_slice()),
    })
}

#[route(GET "/sw.js")]
pub async fn service_worker(_cx: &Cx) -> Result<TypedBody> {
    Ok(TypedBody {
        content_type: "application/javascript; charset=utf-8",
        body: service_worker_body(),
    })
}

fn robots_txt_body() -> String {
    let disallow = ROBOTS_DISALLOW
        .iter()
        .map(|p| format!("Disallow: {p}\n"))
        .collect::<String>();
    format!(
        "User-agent: *\nAllow: /\n{disallow}\nSitemap: {}/sitemap.xml\n",
        crate::config::site_url(),
    )
}

#[route(GET "/robots.txt")]
pub async fn robots_txt(_cx: &Cx) -> Result<TypedBody> {
    Ok(TypedBody {
        content_type: "text/plain; charset=utf-8",
        body: robots_txt_body(),
    })
}

async fn ads_txt_body(db: &crate::db::Db) -> String {
    let client_id = crate::services::site_settings::get_setting(db, "adsense_client").await;
    if client_id.is_empty() {
        return String::new();
    }
    format!("google.com, {client_id}, DIRECT, {ADSENSE_DIRECT_TAG}\n")
}

#[route(GET "/ads.txt")]
pub async fn ads_txt(cx: &Cx) -> Result<TypedBody> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let body = ads_txt_body(db).await;
    Ok(TypedBody {
        content_type: "text/plain; charset=utf-8",
        body,
    })
}

/// Liveness/readiness probe (no auth, single cheap query).
#[route(GET "/healthz")]
pub async fn healthz(cx: &Cx) -> Result<TypedBody> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let ok: bool = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(db)
        .await
        .map(|v| v == 1)
        .unwrap_or(false);
    Ok(TypedBody {
        content_type: "application/json; charset=utf-8",
        body: format!(
            "{{\"status\":\"{}\",\"version\":\"{}\"}}",
            if ok { "ok" } else { "degraded" },
            crate::config::app_version(),
        ),
    })
}

/// Trusted Web Activity asset links (`/.well-known/assetlinks.json`).
/// Served from `TWA_PACKAGE_NAME` + `TWA_SHA256_FINGERPRINTS`; empty array
/// when unconfigured (advertises no association).
#[route(GET "/.well-known/assetlinks.json")]
pub async fn assetlinks(_cx: &Cx) -> Result<TypedBody> {
    Ok(TypedBody {
        content_type: "application/json; charset=utf-8",
        body: assetlinks_body(
            &crate::config::twa_package_name(),
            &crate::config::twa_fingerprints(),
        ),
    })
}

fn assetlinks_body(package: &str, fingerprints: &[String]) -> String {
    let mut statements = String::from("[");
    if !package.is_empty() && !fingerprints.is_empty() {
        let fps: Vec<String> = fingerprints.iter().map(|f| format!("\"{f}\"")).collect();
        statements.push_str(&format!(
            "{{\"relation\":[\"delegate_permission/common.handle_all_urls\"],\"target\":{{\"namespace\":\"android_app\",\"package_name\":\"{package}\",\"sha256_cert_fingerprints\":[{}]}}}}",
            fps.join(","),
        ));
    }
    statements.push(']');
    statements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assetlinks_empty_when_unconfigured() {
        assert_eq!(assetlinks_body("", &[]), "[]");
        assert_eq!(assetlinks_body("kr.rustkorea.hub", &[]), "[]");
    }

    #[test]
    fn assetlinks_statement_shape() {
        let body = assetlinks_body(
            "kr.rustkorea.hub",
            &["AA:BB".to_string(), "CC:DD".to_string()],
        );
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            v[0]["relation"][0],
            "delegate_permission/common.handle_all_urls"
        );
        assert_eq!(v[0]["target"]["namespace"], "android_app");
        assert_eq!(v[0]["target"]["package_name"], "kr.rustkorea.hub");
        assert_eq!(v[0]["target"]["sha256_cert_fingerprints"][1], "CC:DD");
    }

    #[test]
    fn service_worker_is_versioned() {
        let sw = service_worker_body();
        assert!(sw.contains(&format!("rustkorea-{}", crate::config::app_version())));
        assert!(sw.contains("'/offline'"));
    }

    #[test]
    fn manifest_references_png_icons() {
        let m = manifest_json_body("T", "D");
        let v: serde_json::Value = serde_json::from_str(&m).unwrap();
        assert_eq!(v["display"], "standalone");
        let icons = v["icons"].as_array().unwrap();
        assert!(
            icons
                .iter()
                .any(|i| i["sizes"] == "512x512" && i["type"] == "image/png")
        );
        assert!(icons.iter().any(|i| i["purpose"] == "maskable"));
    }
}
