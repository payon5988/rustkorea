//! Shared web helpers (query parsing, pagination).

use topcoat::context::Cx;

/// Read a query-string parameter (canonical; replaces per-file copies).
pub fn query_param(cx: &Cx, key: &str) -> Option<String> {
    let query = topcoat::router::request::uri(cx).query()?;
    form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

/// Clamp `page` and compute `(total_pages, current, offset)`.
pub fn paginate(total: i64, page: i64, per_page: i64) -> (i64, i64, i64) {
    let per_page = per_page.max(1);
    let total_pages = ((total + per_page - 1) / per_page).max(1);
    let current = page.max(1).min(total_pages);
    (total_pages, current, (current - 1) * per_page)
}

/// Parse a comma-separated tag input into a JSON array string.
/// Always valid JSON (proper escaping); at most [`MAX_TAGS`] tags.
pub const MAX_TAGS: usize = 5;

/// Default list page size (overridable per surface via settings).
pub const DEFAULT_PER_PAGE: i64 = 20;

/// External links (footer). Single source; no inline URLs in templates.
pub mod external_links {
    pub const RUST_OFFICIAL: &str = "https://www.rust-lang.org";
    pub const RUST_GITHUB: &str = "https://github.com/rust-lang";
    pub const TOPCOAT_DOCS: &str = "https://docs.rs/topcoat";
    pub const GOOGLE_ADS_POLICIES: &str = "https://policies.google.com/technologies/ads";
}

/// YouTube watch URL builder (canonical www host everywhere).
pub fn youtube_watch_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={video_id}")
}

pub fn tags_json(input: Option<String>) -> String {
    let tags: Vec<String> = input
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .take(MAX_TAGS)
        .collect();
    serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_math() {
        assert_eq!(paginate(0, 1, 20), (1, 1, 0));
        assert_eq!(paginate(45, 1, 20), (3, 1, 0));
        assert_eq!(paginate(45, 9, 20), (3, 3, 40));
        assert_eq!(paginate(45, -2, 20), (3, 1, 0));
        assert_eq!(paginate(45, 2, 0), (45, 2, 1));
    }

    #[test]
    fn tags_json_safe() {
        assert_eq!(tags_json(None), "[]");
        assert_eq!(tags_json(Some("".into())), "[]");
        assert_eq!(tags_json(Some("a,b,c".into())), "[\"a\",\"b\",\"c\"]");
        // quotes/backslashes are escaped, output always parses
        let out = tags_json(Some("a\"b,c\\d, e ".into()));
        let v: Vec<String> = serde_json::from_str(&out).unwrap();
        assert_eq!(v, vec!["a\"b", "c\\d", "e"]);
        let out = tags_json(Some("1,2,3,4,5,6,7".into()));
        assert_eq!(serde_json::from_str::<Vec<String>>(&out).unwrap().len(), 5);
    }
}
