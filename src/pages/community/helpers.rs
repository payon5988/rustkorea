use topcoat::context::Cx;
use topcoat::router::request::uri;

use crate::auth::access::Access;
use crate::db::Db;
use crate::models::{Category, User};

/// Category slugs (single source; DB rows reference these).
pub mod category_slug {
    pub const GENERAL: &str = "general";
    pub const QNA: &str = "qna";
    pub const SHOWCASE: &str = "showcase";
    pub const JOBS: &str = "jobs";
    pub const ANNOUNCE: &str = "announce";
}

/// Category write permissions.
pub mod write_perm {
    pub const EVERYONE: &str = "everyone";
    pub const ADMIN: &str = "admin";
    pub const EDITOR: &str = "editor";
}

/// Category badge colors.
pub mod category_colors {
    pub const QNA: &str = "#2563EB";
    pub const SHOWCASE: &str = "#7C3AED";
    pub const JOBS: &str = "#059669";
    pub const ANNOUNCE: &str = "#DC2626";
    pub const DEFAULT: &str = "#6b7280";
}

pub async fn get_current_user(cx: &Cx) -> Option<User> {
    let db: &Db = topcoat::context::app_context(cx);
    crate::auth::session::current_user(db, cx).await
}

pub fn is_admin_user(access: &Option<Access>) -> bool {
    access.as_ref().map(|a| a.is_admin()).unwrap_or(false)
}

pub fn is_moderator(access: &Option<Access>) -> bool {
    access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::POSTS_MODERATE))
        .unwrap_or(false)
}

pub fn can_write_post(user: &Option<User>) -> bool {
    // Any authenticated user may open the editor; per-category permission
    // (`can_write_to_category`) is enforced at submit time.
    user.is_some()
}

pub fn can_edit_post(access: &Option<Access>, author_id: &str) -> bool {
    is_moderator(access)
        || access
            .as_ref()
            .map(|a| a.user.id == author_id)
            .unwrap_or(false)
}

pub async fn valid_category(db: &Db, category: &str) -> bool {
    sqlx::query_scalar::<_, String>("SELECT slug FROM categories WHERE slug = $1 AND visible = 1")
        .bind(category)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .is_some()
}

pub async fn can_write_to_category(db: &Db, access: &Option<Access>, category: &str) -> bool {
    let permission: String =
        sqlx::query_scalar("SELECT write_permission FROM categories WHERE slug = $1")
            .bind(category)
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| write_perm::EVERYONE.to_string());

    match permission.as_str() {
        write_perm::ADMIN => is_admin_user(access),
        write_perm::EDITOR => is_moderator(access),
        _ => true,
    }
}

/// Built-in category label in `loc` (DB categories carry their own `name_en`).
pub fn category_label_in(category: &str, loc: crate::services::i18n::Locale) -> &'static str {
    match category {
        category_slug::QNA => loc.pick("질문과 답변", "Q&A"),
        category_slug::SHOWCASE => loc.pick("프로젝트 쇼케이스", "Project Showcase"),
        category_slug::JOBS => loc.pick("구인구직", "Jobs"),
        category_slug::ANNOUNCE => loc.pick("공지사항", "Announcements"),
        _ => loc.pick("일반 토론", "General Discussion"),
    }
}

pub fn category_color(category: &str) -> &'static str {
    match category {
        category_slug::QNA => category_colors::QNA,
        category_slug::SHOWCASE => category_colors::SHOWCASE,
        category_slug::JOBS => category_colors::JOBS,
        category_slug::ANNOUNCE => category_colors::ANNOUNCE,
        _ => category_colors::DEFAULT,
    }
}

pub async fn load_categories(db: &Db) -> Vec<Category> {
    sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE visible = 1 ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default()
}

pub fn query_param(cx: &Cx, key: &str) -> Option<String> {
    let query = uri(cx).query()?;
    form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

pub async fn redirect_to(_cx: &Cx, target: &str) -> topcoat::Error {
    topcoat::router::error::redirect(target).into()
}

pub async fn login_redirect(cx: &Cx, return_to: &str) -> topcoat::Error {
    redirect_to(cx, &crate::auth::session::login_link(return_to)).await
}
