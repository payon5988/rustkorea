/// Shared admin UI assets: single source of truth for the admin
/// stylesheet and third-party editor CDN URLs.
/// Previously each admin page embedded its own `ADMIN_STYLE` copy.
pub const ADMIN_STYLE: &str = r#"
:root { --brand: #DE5B30; --bg: #0f1117; --card: #1a1d27; --border: #2a2d3a; --text: #f9fafb; --muted: #6b7280; --dim: #4b5563; }
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans KR', system-ui, sans-serif; background: var(--bg); color: #e5e7eb; min-height: 100vh; }
a { color: inherit; text-decoration: none; }
.btn { background: var(--brand); color: #fff; padding: 10px 20px; border-radius: 8px; font-weight: 600; font-size: 14px; display: inline-flex; align-items: center; gap: 8px; transition: background 0.15s; border: none; cursor: pointer; }
.btn:hover { background: #c94e28; }
.badge { font-size: 12px; font-weight: 500; padding: 4px 10px; border-radius: 9999px; display: inline-flex; }
.input { width: 100%; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px; font-size: 14px; color: var(--text); outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
.input:focus { border-color: var(--brand); box-shadow: 0 0 0 3px rgba(222,91,48,0.15); }
.input::placeholder { color: var(--dim); }
.EasyMDEEditor, .EasyMDEEditor .editor-toolbar, .EasyMDEEditor textarea, .EasyMDEEditor .CodeMirror, .editor-toolbar, .CodeMirror { background: #fff !important; color: #333 !important; border-color: #d1d5db !important; }
.editor-toolbar button { color: #555 !important; background: none !important; }
.editor-toolbar button.active { background: #ddd !important; }
.editor-toolbar i.separator { border-color: #d1d5db !important; }
.wrap { max-width: 420px; margin: 0 auto; padding: 80px 24px; }
.logo { text-align: center; margin-bottom: 40px; }
.logo-icon { width: 48px; height: 48px; border-radius: 12px; background: rgba(222,91,48,0.1); display: inline-flex; align-items: center; justify-content: center; margin-bottom: 16px; }
.logo h1 { font-size: 22px; font-weight: 800; color: var(--text); letter-spacing: -0.02em; }
.logo p { font-size: 14px; color: var(--muted); margin-top: 6px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 32px; }
.card h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; color: var(--text); }
.card .sub { font-size: 13px; color: var(--muted); margin-bottom: 24px; }
.field { margin-bottom: 20px; }
.field label { display: block; font-size: 13px; font-weight: 600; color: #9ca3af; margin-bottom: 6px; }
.field input { width: 100%; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 12px 14px; font-size: 15px; color: var(--text); outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
.field input:focus { border-color: var(--brand); box-shadow: 0 0 0 3px rgba(222,91,48,0.15); }
.field input::placeholder { color: var(--dim); }
.site-link { text-align: center; margin-top: 28px; }
.site-link a { font-size: 12px; color: var(--dim); display: inline-flex; align-items: center; gap: 6px; transition: color 0.15s; }
.site-link a:hover { color: var(--muted); }
.error { background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.2); border-radius: 8px; padding: 12px; margin-bottom: 20px; font-size: 13px; color: #fca5a5; display: none; }
.btn-block { width: 100%; }
"#;

/// EasyMDE markdown editor CDN (jsdelivr, pinned version).
pub const EASYMDE_CSS_URL: &str = "https://cdn.jsdelivr.net/npm/easymde/dist/easymde.min.css";
pub const EASYMDE_JS_URL: &str = "https://cdn.jsdelivr.net/npm/easymde/dist/easymde.min.js";
