#![allow(dead_code)] // translation maps reserved for full i18n rollout
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Ko,
    En,
}

impl Locale {
    pub fn from_str(s: &str) -> Self {
        match s {
            "en" => Locale::En,
            _ => Locale::Ko,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::Ko => "ko",
            Locale::En => "en",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Locale::Ko => "한국어",
            Locale::En => "English",
        }
    }
}

fn ko() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("nav.home", "홈");
        m.insert("nav.manuals", "매뉴얼");
        m.insert("nav.youtube", "영상");
        m.insert("nav.community", "커뮤니티");
        m.insert("nav.events", "이벤트");
        m.insert("nav.login", "로그인");
        m.insert("nav.register", "회원가입");
        m.insert("nav.logout", "로그아웃");
        m.insert("nav.notifications", "알림");
        m.insert("nav.profile", "프로필");
        m.insert("nav.admin", "관리자");
        m.insert("nav.bookmarks", "북마크");
        m.insert("nav.messages", "쪽지");
        m.insert("nav.ranking", "랭킹");
        m.insert("nav.search", "검색");
        m.insert("site.title", "Rust Korea");
        m.insert("site.desc", "Rust 한국어 커뮤니티");
        m
    })
}

fn en() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("nav.home", "Home");
        m.insert("nav.manuals", "Manuals");
        m.insert("nav.youtube", "Videos");
        m.insert("nav.community", "Community");
        m.insert("nav.events", "Events");
        m.insert("nav.login", "Login");
        m.insert("nav.register", "Sign Up");
        m.insert("nav.logout", "Logout");
        m.insert("nav.notifications", "Notifications");
        m.insert("nav.profile", "Profile");
        m.insert("nav.admin", "Admin");
        m.insert("nav.bookmarks", "Bookmarks");
        m.insert("nav.messages", "Messages");
        m.insert("nav.ranking", "Ranking");
        m.insert("nav.search", "Search");
        m.insert("site.title", "Rust Korea");
        m.insert("site.desc", "Rust Korean Community");
        m
    })
}

pub fn t(locale: Locale, key: &str) -> String {
    let map = match locale {
        Locale::Ko => ko(),
        Locale::En => en(),
    };
    map.get(key).unwrap_or(&key).to_string()
}

pub fn detect_locale(cookie_value: Option<&str>, accept_lang: Option<&str>) -> Locale {
    if let Some(c) = cookie_value {
        return Locale::from_str(c);
    }
    if let Some(lang) = accept_lang {
        let lower = lang.to_lowercase();
        if lower.starts_with("ko") {
            return Locale::Ko;
        }
        if lower.starts_with("en") {
            return Locale::En;
        }
    }
    Locale::Ko
}
