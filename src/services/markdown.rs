use pulldown_cmark::{Options, Parser, html};

pub fn render_markdown(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);

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
        ])
        .add_tag_attributes("a", &["href", "title", "target"])
        .add_tag_attributes("img", &["src", "alt", "title", "width", "height"])
        .add_tag_attributes("code", &["class"])
        .add_tag_attributes("pre", &["class"])
        .link_rel(Some("noopener noreferrer"))
        .url_schemes(std::collections::HashSet::from(["http", "https", "mailto"]))
        .clean(&html_output)
        .to_string()
}

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
}
