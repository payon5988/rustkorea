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

/// SQL fragment for the scheduled-publishing gate. The column must exist on
/// the (aliased) table; pass e.g. `"p"` / `"mc"` / `"ms"` / `""`.
pub fn published_clause(alias: &str) -> String {
    let col = if alias.is_empty() {
        "published_at".to_string()
    } else {
        format!("{alias}.published_at")
    };
    format!("({col} IS NULL OR {col}::timestamptz <= NOW())")
}

/// Parse an HTML `datetime-local` input (assumed KST) into UTC storage text
/// (`YYYY-MM-DD HH:MM:SS+00`). Empty input = publish immediately (None).
/// Returns `Err` on malformed input.
pub fn parse_schedule_kst(input: &str) -> Result<Option<String>, ()> {
    let t = input.trim();
    if t.is_empty() {
        return Ok(None);
    }
    let naive = chrono::NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M").map_err(|_| ())?;
    let utc = naive - chrono::Duration::hours(9);
    Ok(Some(utc.format("%Y-%m-%d %H:%M:%S+00").to_string()))
}

/// Normalize UTC storage text (`2026-10-12 01:20:30.779858+00`) to RFC3339.
fn normalize_storage_ts(s: &str) -> String {
    let mut t = s.replacen(' ', "T", 1);
    if let Some(pos) = t.rfind(['+', '-']) {
        if pos > 10 {
            let tz = &t[pos..];
            if tz.len() == 3 {
                t.push_str(":00");
            } else if tz.len() == 5 && !tz.contains(':') {
                t.insert(pos + 3, ':');
            }
        }
    }
    t
}

/// Whether a `published_at` value is released (unset or not in the future).
pub fn is_released(published_at: Option<&str>) -> bool {
    let s = published_at.unwrap_or("").trim();
    if s.is_empty() {
        return true;
    }
    chrono::DateTime::parse_from_rfc3339(&normalize_storage_ts(s))
        .map(|dt| dt <= chrono::Utc::now().fixed_offset())
        .unwrap_or(true)
}

/// Format UTC storage text back to KST `datetime-local` for edit forms.
/// Returns empty string when unset/unparseable.
pub fn format_schedule_kst(stored: Option<&str>) -> String {
    let s = stored.unwrap_or("").trim();
    if s.is_empty() {
        return String::new();
    }
    chrono::DateTime::parse_from_rfc3339(&normalize_storage_ts(s))
        .map(|dt| {
            (dt + chrono::Duration::hours(9))
                .format("%Y-%m-%dT%H:%M")
                .to_string()
        })
        .unwrap_or_default()
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

/// Promo banner visibility: enabled flag plus optional `YYYY-MM-DD` expiry
/// (inclusive, compared in KST). Pure for unit tests.
pub fn banner_visible_on(enabled: bool, until: &str, today_kst: &str) -> bool {
    if !enabled {
        return false;
    }
    let until = until.trim();
    if until.is_empty() {
        return true;
    }
    until >= today_kst.trim()
}

/// Today's date in KST (`YYYY-MM-DD`).
pub fn today_kst() -> String {
    (chrono::Utc::now() + chrono::Duration::hours(9))
        .format("%Y-%m-%d")
        .to_string()
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
    fn banner_expiry() {
        assert!(!banner_visible_on(false, "", "2026-10-14"));
        assert!(banner_visible_on(true, "", "2026-10-14"));
        assert!(banner_visible_on(true, "2026-10-14", "2026-10-14"));
        assert!(banner_visible_on(true, "2026-12-31", "2026-10-14"));
        assert!(!banner_visible_on(true, "2026-10-13", "2026-10-14"));
    }

    #[test]
    fn schedule_kst_roundtrip() {
        assert_eq!(parse_schedule_kst(""), Ok(None));
        assert!(parse_schedule_kst("not-a-date").is_err());
        // 2026-10-20 09:00 KST == 2026-10-20 00:00 UTC.
        assert_eq!(
            parse_schedule_kst("2026-10-20T09:00"),
            Ok(Some("2026-10-20 00:00:00+00".to_string()))
        );
        assert_eq!(
            format_schedule_kst(Some("2026-10-20 00:00:00+00")),
            "2026-10-20T09:00"
        );
        assert_eq!(format_schedule_kst(None), "");
        assert_eq!(format_schedule_kst(Some("garbage")), "");
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
