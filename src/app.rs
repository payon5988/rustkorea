use std::future::Future;
use topcoat::router::request::uri;
use topcoat::{
    Result,
    router::layout,
    tailwind,
    view::{View, view},
};

use crate::components::{cookie_banner::CookieBanner, footer::Footer, header::Header};

#[layout("/")]
pub async fn root_layout(
    cx: &topcoat::context::Cx,
    slot: topcoat::router::Slot<'_>,
) -> Result<impl View + '_> {
    let path = uri(cx).path();
    let bare = path.starts_with("/admin") || path.starts_with("/auth");

    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let site_title = crate::services::site_settings::get_setting(db, "site_title").await;
    let site_title = if site_title.is_empty() {
        "Rust Korea Hub".to_string()
    } else {
        site_title
    };
    let site_description =
        crate::services::site_settings::get_setting(db, "site_description").await;
    let site_description = if site_description.is_empty() {
        "한국 Rust 생태계의 단일 진실 공급원".to_string()
    } else {
        site_description
    };
    let site_keywords = crate::services::site_settings::get_setting(db, "site_keywords").await;
    let adsense_enabled = crate::services::site_settings::get_setting(db, "adsense_enabled").await;
    let adsense_client = crate::services::site_settings::get_setting(db, "adsense_client").await;

    let theme_color = "#2563EB";
    let apple_meta_name = "apple-mobile-web-app-capable";
    let canonical = format!("{}{}", crate::config::site_url(), path);
    let robots_content = if bare {
        "noindex, nofollow"
    } else {
        "index, follow"
    };

    Ok(view! {
        <!DOCTYPE html>
        <html lang="ko">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>(site_title.clone())</title>
                <meta name="description" content=(site_description.clone()) />
                <meta property="og:title" content=(site_title.clone()) />
                <meta property="og:description" content=(site_description.clone()) />
                <meta property="og:type" content="website" />
                <meta property="og:url" content=(canonical.clone()) />
                <meta property="og:site_name" content=(site_title.clone()) />
                <meta property="og:locale" content="ko_KR" />
                <meta name="twitter:card" content="summary" />
                <meta name="robots" content=(robots_content) />
                <meta name="keywords" content=(site_keywords.clone()) />
                <link rel="canonical" href=(canonical.clone()) />
                <link rel="alternate" type="application/rss+xml" title="Rust Korea Hub" href="/feed.xml" />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="stylesheet" href=(tailwind::stylesheet!()) />
                <link rel="manifest" href="/manifest.json" />
                <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
                <meta name="mobile-web-app-capable" content="yes" />
                <link rel="icon" href="/icon-192.png" type="image/png" />
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
                    .desktop-footer { display: block; }
                    @media (max-width: 768px) {
                        .desktop-nav { display: none !important; }
                        .desktop-actions { display: none !important; }
                        .mobile-actions { display: flex !important; }
                        .container { padding: 0 16px; }
                        .hero-title { font-size: 1.5rem !important; }
                        .bottom-tab-bar { display: flex; position: fixed; bottom: 0; left: 0; right: 0; z-index: 50; background: #fff; border-top: 1px solid #e5e7eb; box-shadow: 0 -2px 8px rgba(0,0,0,0.06); padding: 6px 0 max(6px, env(safe-area-inset-bottom)); justify-content: space-around; align-items: center; height: auto; }
                        html.dark .bottom-tab-bar { background: #12141c; border-top-color: #2a2d3a; box-shadow: 0 -2px 8px rgba(0,0,0,0.3); }
                        .bottom-tab { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 4px 0; font-size: 10px; font-weight: 500; text-decoration: none; min-width: 0; flex: 1; transition: color 0.15s; }
                        .bottom-tab svg { flex-shrink: 0; }
                        body { padding-bottom: 72px !important; }
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
