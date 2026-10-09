use topcoat::context::Cx;
use topcoat::router::request::uri;

use crate::auth::access::Access;
use crate::db::Db;
use crate::models::{Category, User};

pub const CATEGORIES: [(&str, &str); 5] = [
    ("general", "일반 토론"),
    ("qna", "질문과 답변"),
    ("showcase", "프로젝트 쇼케이스"),
    ("jobs", "구인구직"),
    ("announce", "공지사항"),
];

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
        .map(|a| a.can("posts.moderate"))
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
            .unwrap_or_else(|| "everyone".to_string());

    match permission.as_str() {
        "admin" => is_admin_user(access),
        "editor" => is_moderator(access),
        _ => true,
    }
}

pub fn category_label(category: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(key, _)| *key == category)
        .map(|(_, label)| *label)
        .unwrap_or("일반 토론")
}

pub fn category_color(category: &str) -> &'static str {
    match category {
        "qna" => "#2563EB",
        "showcase" => "#7C3AED",
        "jobs" => "#059669",
        "announce" => "#DC2626",
        _ => "#6b7280",
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
    redirect_to(cx, &format!("/auth/login?redirect={return_to}")).await
}
