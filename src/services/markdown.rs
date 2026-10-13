use pulldown_cmark::{Options, Parser, html};

/// Convert pasted rich HTML (Notion/Docs/blog export) into markdown, which
/// then flows through the normal [`render_markdown`] pipeline (sanitized
/// there). Scripts/styles/active content are stripped before conversion;
/// tables, images and headings survive as GFM markdown.
pub fn html_to_markdown(pasted: &str) -> String {
    let clean = ammonia::Builder::default()
        .add_tags(&[
            "p",
            "br",
            "hr",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "ul",
            "ol",
            "li",
            "pre",
            "code",
            "blockquote",
            "table",
            "thead",
            "tbody",
            "tr",
            "th",
            "td",
            "a",
            "img",
            "strong",
            "em",
            "b",
            "i",
            "u",
            "del",
            "sup",
            "sub",
            "figure",
            "figcaption",
        ])
        .add_tag_attributes("a", &["href", "title"])
        .add_tag_attributes("img", &["src", "alt", "title", "width", "height"])
        .url_schemes(std::collections::HashSet::from(["http", "https", "mailto"]))
        .clean(pasted)
        .to_string();
    // Convert the sanitized fragment to markdown (whole-fragment converter).
    htmd::convert(&format!("<div>{clean}</div>")).unwrap_or_default()
}

pub fn render_markdown(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_DEFINITION_LIST);

    // Staff-only rich embeds. `:::youtube[VIDEO_ID]` lines become a
    // placeholder before rendering and a sandboxed iframe afterwards.
    // The ID charset is strictly validated, so the emitted HTML is safe
    // by construction (ammonia would strip iframes otherwise).
    let (md_owned, embeds) = extract_youtube_embeds(md);
    let md = md_owned.as_str();

    let parser = Parser::new_ext(md, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // XSS 방지를 ammonia로 필터링 (스킴 제한 + 외부링크 rel)

    ammonia::Builder::default()
        .add_tags(&[
            "p",
            "br",
            "hr",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "ul",
            "ol",
            "li",
            "pre",
            "code",
            "blockquote",
            "table",
            "thead",
            "tbody",
            "tr",
            "th",
            "td",
            "a",
            "img",
            "strong",
            "em",
            "del",
            "sup",
            "sub",
            "dl",
            "dt",
            "dd",
            "figure",
            "figcaption",
            "input",
        ])
        .add_tag_attributes("a", &["href", "title", "target"])
        .add_tag_attributes("img", &["src", "alt", "title", "width", "height"])
        .add_tag_attributes("code", &["class"])
        .add_tag_attributes("pre", &["class"])
        .add_tag_attributes("input", &["type", "checked", "disabled"])
        .link_rel(Some("noopener noreferrer"))
        .url_schemes(std::collections::HashSet::from(["http", "https", "mailto"]))
        .clean(&html_output)
        .to_string()
        .lines()
        .map(|line| restore_youtube_embeds(line, &embeds))
        .collect::<Vec<_>>()
        .join("\n")
}

/// YouTube video IDs: exactly 11 chars of `[A-Za-z0-9_-]`.
pub const YOUTUBE_ID_LEN: usize = 11;
fn valid_video_id(id: &str) -> bool {
    id.len() == YOUTUBE_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Replace `:::youtube[ID]` lines with text placeholders; returns the
/// rewritten markdown plus the validated IDs in order.
fn extract_youtube_embeds(md: &str) -> (String, Vec<String>) {
    let mut embeds = Vec::new();
    let mut out = String::with_capacity(md.len());
    for line in md.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(":::youtube[") {
            if let Some(id) = rest.strip_suffix(']') {
                if valid_video_id(id) {
                    embeds.push(id.to_string());
                    out.push_str(&format!("YOUTUBEEMBED{}END\n", embeds.len() - 1));
                    continue;
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    (out, embeds)
}

/// Privacy-enhanced YouTube embed base.
pub const YOUTUBE_EMBED_BASE: &str = "https://www.youtube-nocookie.com/embed";

fn restore_youtube_embeds(line: &str, embeds: &[String]) -> String {
    let mut out = line.to_string();
    for (i, id) in embeds.iter().enumerate() {
        let token = format!("YOUTUBEEMBED{i}END");
        if out.contains(&token) {
            let iframe = format!(
                "<div class=\"youtube-embed\"><iframe src=\"{YOUTUBE_EMBED_BASE}/{id}\" \
                 title=\"YouTube video\" loading=\"lazy\" \
                 allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\" \
                 allowfullscreen></iframe></div>"
            );
            out = out.replace(&format!("<p>{token}</p>"), &iframe);
            out = out.replace(&token, &iframe);
        }
    }
    out
}

/// Shared stylesheet for rendered markdown (content pages + editor
/// preview via `previewClass`). Single source of truth — do not fork.
pub const MARKDOWN_CSS: &str = r#"
.markdown-body { font-size: 1rem; line-height: 1.8; color: #1f2937; word-wrap: break-word; }
.markdown-body > *:first-child { margin-top: 0; }
.markdown-body > *:last-child { margin-bottom: 0; }
.markdown-body h1, .markdown-body h2, .markdown-body h3, .markdown-body h4, .markdown-body h5, .markdown-body h6 { font-weight: 800; letter-spacing: -0.02em; line-height: 1.3; color: #111827; margin: 1.6em 0 0.8em; }
.markdown-body h1 { font-size: 1.75rem; padding-bottom: 0.4em; border-bottom: 1px solid #e5e7eb; }
.markdown-body h2 { font-size: 1.5rem; padding-bottom: 0.35em; border-bottom: 1px solid #e5e7eb; }
.markdown-body h3 { font-size: 1.25rem; }
.markdown-body h4 { font-size: 1.1rem; }
.markdown-body p { margin: 0 0 1em; }
.markdown-body a { color: #DE5B30; text-decoration: none; }
.markdown-body a:hover { text-decoration: underline; }
.markdown-body ul, .markdown-body ol { margin: 0 0 1em; padding-left: 1.5em; }
.markdown-body li { margin: 0.3em 0; }
.markdown-body li > ul, .markdown-body li > ol { margin-bottom: 0; }
.markdown-body ul.contains-task-list { list-style: none; padding-left: 0.2em; }
.markdown-body li.task-list-item { list-style: none; }
.markdown-body input[type="checkbox"] { margin-right: 0.5em; accent-color: #DE5B30; width: 1em; height: 1em; }
.markdown-body blockquote { margin: 0 0 1em; padding: 0.8em 1.2em; border-left: 4px solid #DE5B30; background: #fafaf9; color: #4b5563; border-radius: 0 8px 8px 0; }
.markdown-body blockquote > *:last-child { margin-bottom: 0; }
.markdown-body table { display: block; overflow-x: auto; border-collapse: collapse; margin: 0 0 1em; font-size: 0.9em; }
.markdown-body th, .markdown-body td { border: 1px solid #e5e7eb; padding: 8px 12px; text-align: left; }
.markdown-body th { background: #f9fafb; font-weight: 700; }
.markdown-body tr:nth-child(even) td { background: #fafafa; }
.markdown-body code { font-family: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace; font-size: 0.85em; background: #f3f4f6; color: #DE5B30; padding: 0.15em 0.4em; border-radius: 6px; }
.markdown-body pre { background: #1C1B1A; border-radius: 12px; padding: 20px; overflow-x: auto; margin: 0 0 1.2em; position: relative; }
.markdown-body pre code { background: transparent; color: #F5F1EA; padding: 0; font-size: 0.85rem; line-height: 1.7; }
.markdown-body img { max-width: 100%; height: auto; border-radius: 8px; }
.markdown-body hr { border: none; border-top: 1px solid #e5e7eb; margin: 2em 0; }
.markdown-body .youtube-embed { position: relative; width: 100%; aspect-ratio: 16 / 9; margin: 0 0 1.2em; border-radius: 12px; overflow: hidden; background: #000; }
.markdown-body .youtube-embed iframe { position: absolute; inset: 0; width: 100%; height: 100%; border: 0; }
.markdown-body .code-copy-btn { position: absolute; top: 10px; right: 10px; font-size: 12px; font-weight: 600; color: #d1d5db; background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.15); border-radius: 6px; padding: 4px 10px; cursor: pointer; }
.markdown-body .code-copy-btn:hover { color: #fff; background: rgba(255,255,255,0.15); }
html.dark .markdown-body { color: #e5e7eb; }
html.dark .markdown-body h1, html.dark .markdown-body h2, html.dark .markdown-body h3, html.dark .markdown-body h4, html.dark .markdown-body h5, html.dark .markdown-body h6 { color: #f9fafb; }
html.dark .markdown-body h1, html.dark .markdown-body h2 { border-bottom-color: #2a2d3a; }
html.dark .markdown-body blockquote { background: #1a1d27; border-left-color: #DE5B30; color: #9ca3af; }
html.dark .markdown-body th { background: #1a1d27; }
html.dark .markdown-body th, html.dark .markdown-body td { border-color: #2a2d3a; }
html.dark .markdown-body tr:nth-child(even) td { background: #14161d; }
html.dark .markdown-body code { background: #2a2d3a; }
html.dark .markdown-body hr { border-top-color: #2a2d3a; }
"#;

/// highlight.js assets (cdnjs, pinned + SRI). Used by content pages.
/// `HL_VERSION` is asserted against the URLs by `urls_pin_hl_version`.
#[allow(dead_code)]
pub const HL_VERSION: &str = "11.9.0";
pub const HL_CSS_URL: &str =
    "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css";
pub const HL_CSS_SRI: &str =
    "sha384-eFTL69TLRZTkNfYZOLM+G04821K1qZao/4QLJbet1pP4tcF+fdXq/9CdqAbWRl/L";
pub const HL_JS_URL: &str =
    "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js";
pub const HL_JS_SRI: &str =
    "sha384-F/bZzf7p3Joyp5psL90p/p89AZJsndkSoGwRpXcZhleCWhd8SnRuoYo4d0yirjJp";
pub const HL_RUST_URL: &str =
    "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js";
pub const HL_RUST_SRI: &str =
    "sha384-h29HWMYGysZp5v7HMfLgfj7PMRyj5l0/KqKdXWYxGfcfh4GxKPNGM/Ria90QFqoZ";

#[cfg(test)]
mod hl_version_tests {
    use super::{HL_CSS_URL, HL_JS_URL, HL_RUST_URL, HL_VERSION};

    #[test]
    fn urls_pin_hl_version() {
        for url in [HL_CSS_URL, HL_JS_URL, HL_RUST_URL] {
            assert!(
                url.contains(HL_VERSION),
                "highlight.js URL drifted from HL_VERSION: {url}"
            );
        }
    }
}

/// Shared EasyMDE initializer (single source; replaces 6 copies).
/// Defines `initEasyMDE(id)` and returns the editor instance.
/// No `<`, `>`, `&` characters (view! would HTML-escape them).
pub const EASYMDE_INIT_JS: &str = r#"function initEasyMDE(id){var el=document.getElementById(id);if(!el){return null;}var ed=new EasyMDE({element:el,minHeight:'300px',status:false,toolbar:['bold','italic','strikethrough','heading','|','quote','unordered-list','ordered-list','table','horizontal-rule','|','link','image','code','preview','side-by-side','fullscreen','guide'],previewClass:['markdown-body','editor-preview']});return ed;}"#;

/// Syntax highlighting + copy buttons for `pre code` blocks.
/// Call as `initHighlight()` after render. No `<`, `>`, `&` characters.
pub const HL_INIT_JS: &str = r#"function initHighlight(){var blocks=document.querySelectorAll('pre code');blocks.forEach(function(el){try{hljs.highlightElement(el);}catch(e){}var btn=document.createElement('button');btn.className='code-copy-btn';btn.textContent='복사';btn.setAttribute('type','button');(function(b,t){b.addEventListener('click',function(){function done(){b.textContent='완료';setTimeout(function(){b.textContent='복사';},1500);}if(navigator.clipboard){navigator.clipboard.writeText(t).then(done);}else{var ta=document.createElement('textarea');ta.value=t;document.body.appendChild(ta);ta.select();try{document.execCommand('copy');done();}catch(err){}document.body.removeChild(ta);}});})(btn,el.innerText);var pre=el.parentElement;if(pre){pre.style.position='relative';pre.appendChild(btn);}});}"#;

/// Table of contents builder for chapter pages: scans `.markdown-body`
/// h1-h3, assigns ids, and fills `#toc-list`. DOM API only (no `<`, `>`, `&`).
pub const TOC_JS: &str = r#"function buildToc(){var body=document.querySelector('.markdown-body');var nav=document.getElementById('toc-list');if(!body||!nav){return;}var hs=body.querySelectorAll('h1,h2,h3');if(!hs.length){var w=document.getElementById('toc-wrap');if(w){w.style.display='none';}return;}hs.forEach(function(h,i){var id='sec-'+i;h.setAttribute('id',id);var li=document.createElement('li');var a=document.createElement('a');a.setAttribute('href','#sec-'+i);a.textContent=h.textContent;var d=h.tagName==='H1'?0:(h.tagName==='H2'?1:2);li.style.paddingLeft=(d*12)+'px';li.appendChild(a);nav.appendChild(li);});}"#;
/// Call as `showEditorTab('md'|'html')`. No `<`, `>`, `&` characters.
pub const EDITOR_TAB_JS: &str = r#"function showEditorTab(n){var md=document.getElementById('tab-md');var ht=document.getElementById('tab-html');var bm=document.getElementById('tabbtn-md');var bh=document.getElementById('tabbtn-html');if(md){md.style.display=n==='md'?'block':'none';}if(ht){ht.style.display=n==='html'?'block':'none';}if(bm){bm.style.borderColor=n==='md'?'var(--brand)':'var(--border)';bm.style.color=n==='md'?'var(--text)':'var(--muted)';}if(bh){bh.style.borderColor=n==='html'?'var(--brand)':'var(--border)';bh.style.color=n==='html'?'var(--text)':'var(--muted)';}}"#;
/// Call as `bindMdUpload('md-file', easymde)`. No `<`, `>`, `&` characters.
/// Falls back to the raw textarea when EasyMDE failed to initialize, so a
/// CDN hiccup never silently disables uploads.
pub const MD_UPLOAD_JS: &str = r#"function bindMdUpload(inputId,ed){var inp=document.getElementById(inputId);if(!inp){return;}inp.addEventListener('change',function(){var f=inp.files[0];if(!f){return;}var r=new FileReader();r.onload=function(){var t=r.result;if(typeof t!=='string'){return;}var done=false;try{if(ed){ed.value(t);done=true;}}catch(e){}if(!done){var ta=document.getElementById('editor');if(ta){ta.value=t;try{ta.dispatchEvent(new Event('input',{bubbles:true}));}catch(e2){}}}};r.readAsText(f);});}"#;

/// Allowlist for avatar/thumbnail URLs stored from external providers.
/// Anything else is dropped at write time (defense in depth; renderers
/// also escape).
pub fn safe_http_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.starts_with("https://") && !url.contains(['<', '>', '"', '\'', ' ']) {
        Some(url.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_import_keeps_structure_drops_scripts() {
        let html = "<h1>제목</h1><p>본문 <b>굵게</b></p><table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table><script>alert(1)</script>";
        let md = html_to_markdown(html);
        let out = render_markdown(&md);
        assert!(out.contains("<h1>제목</h1>"), "heading lost: {md}");
        assert!(out.contains("<table"), "table lost: {md}");
        assert!(!out.contains("alert"), "script leaked");
    }

    #[test]
    fn test_basic_markdown() {
        let input = "# Hello\n\nThis is **bold** and *italic*.";
        let output = render_markdown(input);
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let output = render_markdown(input);
        assert!(output.contains("<code"));
    }

    #[test]
    fn test_xss_prevention() {
        let input = "<script>alert('xss')</script>";
        let output = render_markdown(input);
        assert!(!output.contains("<script>"));
    }

    #[test]
    fn test_task_list() {
        let input = "- [x] done\n- [ ] todo\n";
        let output = render_markdown(input);
        assert!(output.contains("checkbox"));
        assert!(output.contains("checked"));
    }

    #[test]
    fn test_youtube_embed() {
        let input = "intro\n\n:::youtube[dQw4w9WgXcQ]\n\noutro";
        let output = render_markdown(input);
        assert!(output.contains("youtube-nocookie.com/embed/dQw4w9WgXcQ"));
        assert!(!output.contains("YOUTUBEEMBED"));
    }

    #[test]
    fn test_youtube_embed_rejects_bad_id() {
        for bad in [
            ":::youtube[short]",
            ":::youtube[waytoolongvideoid123]",
            ":::youtube[abc\" onload=\"x]",
            ":::youtube[<script>]",
            "not an embed :::youtube[dQw4w9WgXcQ] trailing",
        ] {
            let output = render_markdown(bad);
            assert!(
                !output.contains("youtube-nocookie"),
                "rejected input leaked iframe: {bad}"
            );
            assert!(!output.contains("<script>"));
        }
    }

    #[test]
    fn test_iframe_still_stripped() {
        let input = "<iframe src=\"https://evil.example\"></iframe>";
        let output = render_markdown(input);
        assert!(!output.contains("<iframe"));
    }
}
