//! Public-site language (ko default, en optional). The admin console stays
//! Korean.
//!
//! Resolution per request: `?lang=ko|en` > `locale` cookie > Korean.
//! `Accept-Language` is deliberately ignored so a URL without `?lang`
//! always serves Korean to crawlers; English pages are reachable (and
//! advertised via hreflang) as `?lang=en`.
//!
//! UI strings are written in place as `loc.pick("한국어", "English")`;
//! DB content uses [`Locale::text`], which falls back to Korean when the
//! English field is empty.

use topcoat::{context::Cx, cookie::Cookies};

pub const LOCALE_COOKIE: &str = "locale";
pub const LANG_PARAM: &str = "lang";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Ko,
    En,
}

impl Locale {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ko" => Some(Locale::Ko),
            "en" => Some(Locale::En),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Locale::Ko => "ko",
            Locale::En => "en",
        }
    }

    /// Open Graph `og:locale`.
    pub fn og(self) -> &'static str {
        match self {
            Locale::Ko => "ko_KR",
            Locale::En => "en_US",
        }
    }

    pub fn other(self) -> Self {
        match self {
            Locale::Ko => Locale::En,
            Locale::En => Locale::Ko,
        }
    }

    /// Fixed UI string in this locale.
    pub fn pick<'a>(self, ko: &'a str, en: &'a str) -> &'a str {
        match self {
            Locale::Ko => ko,
            Locale::En => en,
        }
    }

    /// DB content: the English field when present, else the Korean original.
    pub fn text<'a>(self, ko: &'a str, en: &'a str) -> &'a str {
        if self == Locale::En && !en.trim().is_empty() {
            en
        } else {
            ko
        }
    }

    /// Like [`Locale::text`], owned.
    pub fn text_owned(self, ko: &str, en: &str) -> String {
        self.text(ko, en).to_string()
    }
}

/// Pure resolution (unit-tested): query value, then cookie, else Korean.
pub fn resolve(query: Option<&str>, cookie: Option<&str>) -> Locale {
    query
        .and_then(Locale::parse)
        .or_else(|| cookie.and_then(Locale::parse))
        .unwrap_or(Locale::Ko)
}

/// This request's public-site locale.
pub fn locale(cx: &Cx) -> Locale {
    let query = crate::util::query_param(cx, LANG_PARAM);
    let cookie = topcoat::cookie::cookies(cx)
        .get(LOCALE_COOKIE)
        .map(|c| c.value().to_string());
    resolve(query.as_deref(), cookie.as_deref())
}

/// Persist an explicit `?lang=` choice so later navigation keeps it.
pub fn remember_locale(cx: &Cx) {
    let Some(loc) = crate::util::query_param(cx, LANG_PARAM)
        .as_deref()
        .and_then(Locale::parse)
    else {
        return;
    };
    let mut cookie = topcoat::cookie::Cookie::new(LOCALE_COOKIE, loc.as_str());
    cookie.set_path("/");
    cookie.set_same_site(topcoat::cookie::SameSite::Lax);
    cookie.set_max_age(topcoat::cookie::time::Duration::days(365));
    if crate::auth::cookie_store::secure_cookies() {
        cookie.set_secure(true);
    }
    topcoat::cookie::cookies(cx).add(cookie);
}

/// `path` + `query` with `lang` set to `loc` (other parameters kept).
pub fn lang_href(path: &str, query: Option<&str>, loc: Locale) -> String {
    let mut s = form_urlencoded::Serializer::new(String::new());
    for (k, v) in form_urlencoded::parse(query.unwrap_or("").as_bytes()) {
        if k != LANG_PARAM {
            s.append_pair(&k, &v);
        }
    }
    s.append_pair(LANG_PARAM, loc.as_str());
    format!("{path}?{}", s.finish())
}

/// Current request URL switched to `loc` (relative).
pub fn switch_href(cx: &Cx, loc: Locale) -> String {
    let uri = topcoat::router::request::uri(cx);
    lang_href(uri.path(), uri.query(), loc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_order() {
        assert_eq!(resolve(None, None), Locale::Ko);
        assert_eq!(resolve(None, Some("en")), Locale::En);
        assert_eq!(resolve(Some("ko"), Some("en")), Locale::Ko);
        assert_eq!(resolve(Some("en"), None), Locale::En);
        assert_eq!(resolve(Some("fr"), Some("en")), Locale::En);
        assert_eq!(resolve(Some("fr"), Some("xx")), Locale::Ko);
    }

    #[test]
    fn content_falls_back_to_korean() {
        assert_eq!(Locale::En.text("매뉴얼", "Manuals"), "Manuals");
        assert_eq!(Locale::En.text("매뉴얼", "  "), "매뉴얼");
        assert_eq!(Locale::Ko.text("매뉴얼", "Manuals"), "매뉴얼");
    }

    #[test]
    fn lang_href_keeps_other_params() {
        assert_eq!(
            lang_href("/community", None, Locale::En),
            "/community?lang=en"
        );
        assert_eq!(
            lang_href("/community", Some("page=2&lang=en&q=a+b"), Locale::Ko),
            "/community?page=2&q=a+b&lang=ko"
        );
    }
}
