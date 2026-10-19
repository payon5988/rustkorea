use std::future::Future;
use topcoat::router::request::uri;
use topcoat::{
    Result,
    router::layout,
    tailwind,
    view::{View, view},
};

use crate::components::{cookie_banner::CookieBanner, footer::Footer, header::Header};

/// Paths rendered without the public chrome (own layouts instead).
pub const BARE_PREFIXES: [&str; 2] = ["/admin", "/auth"];

/// Robots directives for the layout head.
pub const ROBOTS_INDEX: &str = "index, follow";
pub const ROBOTS_NOINDEX: &str = "noindex, nofollow";

#[layout("/")]
pub async fn root_layout(
    cx: &topcoat::context::Cx,
    slot: topcoat::router::Slot<'_>,
) -> Result<impl View + '_> {
    let path = uri(cx).path();
    let bare = BARE_PREFIXES.iter().any(|p| path.starts_with(p));

    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let loc = crate::services::i18n::locale(cx);
    crate::services::i18n::remember_locale(cx);
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
    let site_keywords = crate::services::site_settings::get_setting(db, "site_keywords").await;
    let adsense_enabled = crate::services::site_settings::get_setting(db, "adsense_enabled").await;
    let adsense_client = crate::services::site_settings::get_setting(db, "adsense_client").await;
    // Search-console ownership tokens (empty until the owner verifies in each
    // console and sets them via env; meta-tag method on both engines).
    let google_verification = crate::config::env_or("GOOGLE_SITE_VERIFICATION", "");
    let naver_verification = crate::config::env_or("NAVER_SITE_VERIFICATION", "");

    let theme_color = "#2563EB";
    let apple_meta_name = "apple-mobile-web-app-capable";
    // Korean lives at the bare URL, English at `?lang=en` (hreflang pair).
    let href_ko = format!("{}{}", crate::config::site_url(), path);
    let href_en = format!("{href_ko}?lang=en");
    let canonical = match loc {
        crate::services::i18n::Locale::Ko => href_ko.clone(),
        crate::services::i18n::Locale::En => href_en.clone(),
    };
    let robots_content = if bare { ROBOTS_NOINDEX } else { ROBOTS_INDEX };

    // Per-URL Open Graph: community posts and manual chapters expose their
    // own title/summary so shares match the content (KakaoTalk etc.).
    // Falls back to the site defaults on any failure.
    let (og_title, og_description, og_kind) =
        resolve_page_og(db, path, &site_title, &site_description, loc).await;
    let og_image = format!("{}/og-image.png", crate::config::site_url());

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(loc.as_str())>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>(site_title.clone())</title>
                <meta name="description" content=(site_description.clone()) />
                <meta property="og:title" content=(og_title.clone()) />
                <meta property="og:description" content=(og_description.clone()) />
                <meta property="og:type" content=(og_kind.clone()) />
                <meta property="og:url" content=(canonical.clone()) />
                <meta property="og:image" content=(og_image.clone()) />
                <meta property="og:site_name" content=(site_title.clone()) />
                <meta property="og:locale" content=(loc.og()) />
                <meta property="og:locale:alternate" content=(loc.other().og()) />
                <meta name="twitter:card" content="summary_large_image" />
                <meta name="twitter:image" content=(og_image.clone()) />
                <meta name="robots" content=(robots_content) />
                if !google_verification.is_empty() {
                    <meta name="google-site-verification" content=(google_verification.clone()) />
                }
                if !naver_verification.is_empty() {
                    <meta name="naver-site-verification" content=(naver_verification.clone()) />
                }
                <meta name="keywords" content=(site_keywords.clone()) />
                <link rel="canonical" href=(canonical.clone()) />
                if !bare {
                    <link rel="alternate" hreflang="ko" href=(href_ko.clone()) />
                    <link rel="alternate" hreflang="en" href=(href_en.clone()) />
                    <link rel="alternate" hreflang="x-default" href=(href_ko.clone()) />
                }
                <link rel="alternate" type="application/rss+xml" title="Rust Korea Hub" href="/feed.xml" />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="stylesheet" href=(tailwind::stylesheet!()) />
                <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
                <link rel="manifest" href="/manifest.json" />
                <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
                <meta name="mobile-web-app-capable" content="yes" />
                <link rel="icon" href="/logo.svg" type="image/svg+xml" />
                <meta name="theme-color" content=(theme_color) />
                <meta name=(apple_meta_name) content="yes" />
                if adsense_enabled == "true" && !adsense_client.is_empty() {
                    <meta name="google-adsense-account" content=(adsense_client.clone()) />
                }
                topcoat::dev::script()
                topcoat::runtime::script()
                <script>
                    {r#"function toggleTheme(){var t=document.cookie.split(';').find(function(c){return c.trim().startsWith('theme=')});var m=t?t.split('=')[1]:'light';var n=m==='dark'?'light':'dark';document.cookie='theme='+n+';path=/;max-age=31536000;SameSite=Lax'+(location.protocol==='https:'?';Secure':'');if(n==='dark'){document.documentElement.classList.add('dark')}else{document.documentElement.classList.remove('dark')}}(function(){var t=document.cookie.split(';').find(function(c){return c.trim().startsWith('theme=')});if(t){if(t.split('=')[1]==='dark'){document.documentElement.classList.add('dark')}}})();function toggleAdminSidebar(){var s=document.getElementById('admin-sidebar');var o=document.getElementById('admin-sidebar-overlay');if(s){if(s.style.transform==='translateX(-100%)'){s.style.transform='translateX(0)';if(o)o.style.display='block'}else{s.style.transform='translateX(-100%)';if(o)o.style.display='none'}}}function closeAdminSidebar(){var s=document.getElementById('admin-sidebar');var o=document.getElementById('admin-sidebar-overlay');if(s)s.style.transform='translateX(-100%)';if(o)o.style.display='none'}if('serviceWorker' in navigator){navigator.serviceWorker.register('/sw.js')}window.adsbygoogle=window.adsbygoogle||[];"#}
                </script>
                <style>
                    {r#":root {
                        --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans KR', system-ui, sans-serif;
                        --color-brand: #2563EB;
                        --color-brand-hover: #1D4ED8;
                        --color-bg: #FFFFFF;
                        --color-bg-secondary: #f8f9fa;
                        --color-border: #E4E4E7;
                        --color-text: #16181d;
                        --color-text-secondary: #6b7280;
                        --color-text-muted: #9ca3af;
                    }
                    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
                    body { font-family: var(--font-sans); background: var(--color-bg); color: var(--color-text); -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; line-height: 1.6; font-size: 16px; transition: background 0.2s, color 0.2s; }
                    .container { max-width: 1280px; margin: 0 auto; padding: 0 24px; }
                    .container-narrow { max-width: 960px; margin: 0 auto; padding: 0 24px; }
                    ::selection { background: rgba(37,99,235,0.15); color: #16181d; }
                    a { color: inherit; text-decoration: none; }
                    .text-accent { color: var(--color-brand); }
                    .section-heading { font-size: 1.75rem; font-weight: 700; letter-spacing: -0.02em; line-height: 1.2; color: var(--color-text); }
                    .section-sub { font-size: 1rem; color: var(--color-text-secondary); line-height: 1.7; }
                    .card { background: #fff; border: 1px solid var(--color-border); border-radius: 8px; transition: border-color 0.15s, box-shadow 0.15s; }
                    .card:hover { border-color: #d1d5db; box-shadow: 0 4px 6px rgba(0,0,0,0.04); }
                    .btn-primary { background: var(--color-brand); color: #fff; padding: 10px 20px; border-radius: 8px; font-weight: 600; font-size: 15px; display: inline-flex; align-items: center; gap: 8px; transition: background 0.15s; border: none; cursor: pointer; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
                    .btn-primary:hover { background: var(--color-brand-hover); }
                    .btn-secondary { background: #fff; color: var(--color-text); padding: 10px 20px; border-radius: 8px; font-weight: 500; font-size: 15px; display: inline-flex; align-items: center; gap: 8px; transition: background 0.15s; border: 1px solid var(--color-border); cursor: pointer; }
                    .btn-secondary:hover { background: #f9fafb; }
                    .badge { font-size: 12px; font-weight: 500; padding: 4px 8px; border-radius: 9999px; display: inline-flex; align-items: center; gap: 6px; }
                    .input { width: 100%; background: #fff; border: 1px solid #d1d5db; border-radius: 6px; padding: 10px 12px; font-size: 16px; color: #111827; outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
                    .input:focus { border-color: var(--color-brand); box-shadow: 0 0 0 3px rgba(37,99,235,0.1); }
                    .input::placeholder { color: #9ca3af; }
                    textarea.input { resize: vertical; font-family: 'SF Mono', 'Fira Code', monospace; line-height: 1.6; }
                    .divider { height: 1px; background: var(--color-border); }
                    .table { width: 100%; border-collapse: collapse; font-size: 14px; }
                    .table th { padding: 12px 16px; text-align: left; font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase; letter-spacing: 0.04em; border-bottom: 1px solid var(--color-border); background: #f9fafb; }
                    .table td { padding: 14px 16px; border-bottom: 1px solid #f3f4f6; }
                    .table tr:hover td { background: #f9fafb; }
                    .nav-active { color: var(--color-brand) !important; border-bottom-color: var(--color-brand) !important; }
                    @keyframes fadeUp { from { opacity: 0; transform: translateY(16px); } to { opacity: 1; transform: translateY(0); } }
                    .fade-up { animation: fadeUp 0.5s ease-out both; }
                    .delay-1 { animation-delay: 0.1s; }
                    .delay-2 { animation-delay: 0.2s; }
                    .delay-3 { animation-delay: 0.3s; }
                    html.dark body { background: #0f1117; color: #e5e7eb; }
                    html.dark .card { background: #1a1d27; border-color: #2a2d3a; }
                    html.dark .card:hover { border-color: #3a3d4a; box-shadow: 0 4px 6px rgba(0,0,0,0.2); }
                    html.dark a { color: #e5e7eb; }
                    html.dark .input { background: #0f1117; border-color: #2a2d3a; color: #e5e7eb; }
                    html.dark .btn-secondary { background: #1a1d27; border-color: #2a2d3a; color: #e5e7eb; }
                    html.dark .table th { background: #1a1d27; color: #9ca3af; border-color: #2a2d3a; }
                    html.dark .table td { border-color: #2a2d3a; }
                    html.dark .table tr:hover td { background: #1a1d27; }
                    html.dark ::selection { background: rgba(37,99,235,0.3); color: #e5e7eb; }
                    html.dark .desktop-footer { background: #0f1117; border-top-color: #2a2d3a; }
                    html.dark .desktop-footer h4 { color: #9ca3af; }
                    html.dark header { background: #0f1117 !important; border-color: #2a2d3a !important; }
                    html.dark header a, html.dark header span, html.dark header button { color: #e5e7eb !important; }
                    html.dark .EasyMDEEditor .editor-toolbar,
                    html.dark .EasyMDEEditor .editor-preview,
                    html.dark .EasyMDEEditor .editor-preview-side,
                    html.dark .EasyMDEEditor textarea,
                    html.dark .EasyMDEEditor .CodeMirror,
                    html.dark .editor-toolbar,
                    html.dark .EasyMDEEditor { background: #fff !important; color: #111827 !important; border-color: #d1d5db !important; }
                    html.dark .editor-toolbar button { color: #555 !important; }
                    html.dark .editor-toolbar button.active { background: #ddd !important; }
                    html.dark .CodeMirror { color: #333 !important; background: #fff !important; }
                    .mobile-actions { display: none; align-items: center; gap: 8px; }
                    .bottom-tab-bar { display: none; }
                    .mobile-menu-dialog { width:min(400px,calc(100vw - 32px)); max-height:80dvh; margin:auto; padding:20px; border:1px solid #e5e7eb; border-radius:14px; background:#fff; color:#16181d; }
                    .mobile-menu-dialog::backdrop { background:rgba(0,0,0,.45); }
                    .mobile-menu-dialog a { padding:14px 12px; border-radius:8px; color:inherit; }
                    .mobile-menu-dialog a[aria-current=page] { color:#2563EB; background:rgba(37,99,235,.1); }
                    .mobile-menu-dialog button { padding:8px 12px; border:1px solid #9ca3af; border-radius:8px; background:transparent; color:inherit; cursor:pointer; }
                    html.dark .mobile-menu-dialog { background:#12141c; color:#e5e7eb; border-color:#2a2d3a; }
                    html.dark .mobile-footer { background:#0f1117 !important; border-top-color:#2a2d3a !important; }
                    .desktop-footer { display: block; }
                    @media (max-width: 768px) {
                        .desktop-nav { display: none !important; }
                        .desktop-actions { display: none !important; }
                        .mobile-actions { display: flex !important; }
                        .container { padding: 0 16px; }
                        .hero-title { font-size: 1.5rem !important; }
                        .bottom-tab-bar { display: flex; position: fixed; bottom: 0; left: 0; right: 0; z-index: 50; background: #fff; border-top: 1px solid #e5e7eb; box-shadow: 0 -2px 8px rgba(0,0,0,0.06); padding: 6px 0 max(6px, env(safe-area-inset-bottom)); justify-content: space-around; align-items: center; height: calc(64px + env(safe-area-inset-bottom)); box-sizing:border-box; }
                        #cookie-banner { bottom:calc(64px + env(safe-area-inset-bottom)) !important; padding:12px 16px !important; }
                        #cookie-banner p { min-width:0 !important; }
                        html.dark .bottom-tab-bar { background: #12141c; border-top-color: #2a2d3a; box-shadow: 0 -2px 8px rgba(0,0,0,0.3); }
                        .bottom-tab { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 4px 0; font-size: 10px; font-weight: 500; text-decoration: none; min-width: 0; flex: 1; transition: color 0.15s; }
                        .bottom-tab-scroll { display:flex; flex:1; min-width:0; overflow-x:auto; overscroll-behavior-x:contain; }
                        .bottom-tab-scroll .bottom-tab { flex:1 0 58px; min-width:58px; max-width:none; }
                        .bottom-tab span { max-width:100%; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; padding:0 3px; }
                        .bottom-tab.account-tab { flex:0 0 60px; }
                        .bottom-tab svg { flex-shrink: 0; }
                        body { padding-bottom: calc(76px + env(safe-area-inset-bottom)) !important; }
                        .desktop-footer { display: none !important; }
                        .grid-3 { grid-template-columns: 1fr !important; }
                        .grid-4 { grid-template-columns: 1fr 1fr !important; }
                        .post-author, .post-view-count { display: none !important; }
                        .mobile-footer { display: block !important; }
                    }
                    @media (max-width: 480px) {
                        .container { padding: 0 12px; }
                        .grid-4 { grid-template-columns: 1fr !important; }
                        .mobile-footer-links { flex-direction: column !important; gap: 24px !important; }
                    }"#}
                </style>
            </head>
            <body>
                if bare {
                    (slot)
                } else {
                    {Header()}
                    <main>
                        (slot)
                    </main>
                    {Footer()}
                    {CookieBanner()}
                }
            </body>
        </html>
    })
}

/// Per-URL Open Graph (title, description, type). Community posts and manual
/// chapters resolve their own metadata (released rows only, so scrapers see
/// what anonymous visitors see); everything else uses the site defaults.
async fn resolve_page_og(
    db: &crate::db::Db,
    path: &str,
    site_title: &str,
    site_description: &str,
    loc: crate::services::i18n::Locale,
) -> (String, String, String) {
    // English fields fall back to Korean when empty.
    let en = loc == crate::services::i18n::Locale::En;
    let col = |table: &str, name: &str| {
        if en {
            format!("COALESCE(NULLIF({table}{name}_en, ''), {table}{name})")
        } else {
            format!("{table}{name}")
        }
    };
    fn summary(md: &str) -> String {
        let one_line: String = md
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(140)
            .collect();
        // Strip leading markdown markers for a clean preview.
        one_line
            .trim_start_matches(['#', '>', '-', '*', ' '])
            .to_string()
    }
    let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segs.as_slice() {
        ["community", id] if *id != "new" => {
            let row: Option<(String, String)> = sqlx::query_as(&format!(
                "SELECT title, content_md FROM posts WHERE id = $1 AND {}",
                crate::util::published_clause("")
            ))
            .bind(*id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
            match row {
                Some((title, md)) => (title, summary(&md), "article".to_string()),
                None => (
                    site_title.to_string(),
                    site_description.to_string(),
                    "website".to_string(),
                ),
            }
        }
        ["blog", slug] => {
            let row: Option<(String, String)> = sqlx::query_as(&format!(
                "SELECT {}, {} FROM blog_posts WHERE slug = $1 AND {}",
                col("", "title"),
                col("", "content_md"),
                crate::util::published_clause("")
            ))
            .bind(*slug)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
            match row {
                Some((title, md)) => (title, summary(&md), "article".to_string()),
                None => (
                    site_title.to_string(),
                    site_description.to_string(),
                    "website".to_string(),
                ),
            }
        }
        ["manuals", series, chapter] => {
            let row: Option<(String, String, String)> = sqlx::query_as(&format!(
                "SELECT {}, {}, {} FROM manual_chapters mc
                 JOIN manual_series ms ON ms.id = mc.series_id
                 WHERE ms.slug = $1 AND mc.slug = $2 AND {} AND {}",
                col("mc.", "title"),
                col("mc.", "content_md"),
                col("ms.", "title"),
                crate::util::published_clause("mc"),
                crate::util::published_clause("ms"),
            ))
            .bind(*series)
            .bind(*chapter)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
            match row {
                Some((title, md, series_title)) => (
                    format!("{title} - {series_title}"),
                    summary(&md),
                    "article".to_string(),
                ),
                None => (
                    site_title.to_string(),
                    site_description.to_string(),
                    "website".to_string(),
                ),
            }
        }
        _ => (
            site_title.to_string(),
            site_description.to_string(),
            "website".to_string(),
        ),
    }
}
