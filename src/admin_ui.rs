/// Shared admin UI assets: single source of truth for the admin
/// stylesheet and third-party editor CDN URLs.
/// Previously each admin page embedded its own `ADMIN_STYLE` copy.
pub const ADMIN_STYLE: &str = r#"
:root { --brand: #DE5B30; color-scheme: dark; --bg: #0f1117; --bg2: #12141c; --card: #1a1d27; --border: #2a2d3a; --border-soft: #1e2030; --thumb: #27262b; --text: #f9fafb; --text2: #e5e7eb; --text3: #d1d5db; --muted: #9ca3af; --muted2: #6b7280; --dim: #4b5563; --faint: #3f3f46; --line: rgba(255,255,255,0.03); --line-strong: rgba(255,255,255,0.08); --red: #f87171; --red-soft: #fca5a5; --amber: #fbbf24; --green: #34d399; --blue: #60a5fa; --purple: #a78bfa; --blue-soft: #93c5fd; }
:root[data-theme='light'] { color-scheme: light; --bg: #f1f2f4; --bg2: #ffffff; --card: #ffffff; --border: #e2e4e9; --border-soft: #e8eaee; --thumb: #e8eaee; --text: #111827; --text2: #1f2937; --text3: #4b5563; --muted: #5b6472; --muted2: #6b7280; --dim: #9ca3af; --faint: #d1d5db; --line: rgba(17,24,39,0.06); --line-strong: rgba(17,24,39,0.12); --red: #dc2626; --red-soft: #b91c1c; --amber: #b45309; --green: #059669; --blue: #2563eb; --purple: #7c3aed; --blue-soft: #1d4ed8; }
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans KR', system-ui, sans-serif; background: var(--bg); color: var(--text2); min-height: 100vh; }
a { color: inherit; text-decoration: none; }
.btn { background: var(--brand); color: #fff; padding: 10px 20px; border-radius: 8px; font-weight: 600; font-size: 14px; display: inline-flex; align-items: center; gap: 8px; transition: background 0.15s; border: none; cursor: pointer; }
.btn:hover { background: #c94e28; }
.badge { font-size: 12px; font-weight: 500; padding: 4px 10px; border-radius: 9999px; display: inline-flex; }
.input { width: 100%; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px; font-size: 14px; color: var(--text); outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
.input:focus { border-color: var(--brand); box-shadow: 0 0 0 3px rgba(222,91,48,0.15); }
.input::placeholder { color: var(--dim); }
.atheme-btn { flex: 1; background: transparent; border: 1px solid var(--border); color: var(--muted); border-radius: 6px; font-size: 11px; font-weight: 600; padding: 6px 4px; cursor: pointer; font-family: inherit; transition: all 0.15s; }
.atheme-btn:hover { color: var(--text); border-color: var(--dim); }
.atheme-btn[data-active='1'] { background: rgba(222,91,48,0.12); border-color: var(--brand); color: var(--text); }
.toastui-editor-defaultUI { border-color: #d1d5db !important; }
.wrap { max-width: 420px; margin: 0 auto; padding: 80px 24px; }
.logo { text-align: center; margin-bottom: 40px; }
.logo-icon { width: 48px; height: 48px; border-radius: 12px; background: rgba(222,91,48,0.1); display: inline-flex; align-items: center; justify-content: center; margin-bottom: 16px; }
.logo h1 { font-size: 22px; font-weight: 800; color: var(--text); letter-spacing: -0.02em; }
.logo p { font-size: 14px; color: var(--muted2); margin-top: 6px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 32px; }
.card h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; color: var(--text); }
.card .sub { font-size: 13px; color: var(--muted2); margin-bottom: 24px; }
.field { margin-bottom: 20px; }
.field label { display: block; font-size: 13px; font-weight: 600; color: var(--muted); margin-bottom: 6px; }
.field input { width: 100%; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 12px 14px; font-size: 15px; color: var(--text); outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
.field input:focus { border-color: var(--brand); box-shadow: 0 0 0 3px rgba(222,91,48,0.15); }
.field input::placeholder { color: var(--dim); }
.site-link { text-align: center; margin-top: 28px; }
.site-link a { font-size: 12px; color: var(--dim); display: inline-flex; align-items: center; gap: 6px; transition: color 0.15s; }
.site-link a:hover { color: var(--muted2); }
.error { background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.2); border-radius: 8px; padding: 12px; margin-bottom: 20px; font-size: 13px; color: var(--red-soft); display: none; }
.btn-block { width: 100%; }
"#;

/// Admin theme (dark / light / system) bootstrap + setter.
/// Stored in `localStorage` under `rk-admin-theme`; `system` follows
/// `prefers-color-scheme`. Include via `<script>{(ADMIN_THEME_JS)}</script>`.
pub const ADMIN_THEME_JS: &str = r#"(function(){var K="rk-admin-theme";var cur="dark";try{cur=localStorage.getItem(K)||"dark"}catch(e){}function mark(m){try{var bs=document.querySelectorAll("[data-admin-theme-btn]");if(bs.forEach){bs.forEach(function(b){if(b.getAttribute("data-admin-theme-btn")===m){b.setAttribute("data-active","1")}else{b.removeAttribute("data-active")}})}}catch(e){}}function apply(m){var t=m;if(m==="system"){var isLight=false;try{var mq=window.matchMedia("(prefers-color-scheme: light)");if(mq){isLight=mq.matches}}catch(e){}if(isLight){t="light"}else{t="dark"}}document.documentElement.setAttribute("data-theme",t);mark(m)}try{window.__rkSetAdminTheme=function(m){try{localStorage.setItem(K,m)}catch(e){}apply(m)}}catch(e){}apply(cur);try{if(window.matchMedia){window.matchMedia("(prefers-color-scheme: light)").addEventListener("change",function(){var v="dark";try{v=localStorage.getItem(K)||"dark"}catch(e){}if(v==="system"){apply("system")}})}}catch(e){}})();"#;

/// Toast UI Editor (NHN) 3.2.2, self-hosted under `/vendor/` (no CDN
/// dependency). Markdown/WYSIWYG dual mode with table GUI editing and
/// paste-to-upload via `addImageBlobHook`.
pub const TOAST_CSS_URL: &str = "/vendor/toastui-editor.min.css";
pub const TOAST_JS_URL: &str = "/vendor/toastui-editor-all.min.js";

#[cfg(test)]
mod tests {
    use super::{ADMIN_STYLE, ADMIN_THEME_JS};

    #[test]
    fn theme_js_escape_safe() {
        assert!(
            !ADMIN_THEME_JS.contains(['&', '<', '>']),
            "view! would escape these and kill the theme script"
        );
    }

    #[test]
    fn admin_style_escape_safe() {
        assert!(
            !ADMIN_STYLE.contains(['&', '<', '>']),
            "view! would escape these and break admin CSS"
        );
    }
}
