use std::collections::HashSet;

use topcoat::context::Cx;

use crate::db::Db;
use crate::models::{Role, User};

/// Authenticated identity with resolved role + permission set.
/// All authorization decisions go through this; role-name string
/// comparisons elsewhere are legacy and must not be extended.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Access {
    pub user: User,
    pub role_id: String,
    pub role_name: String,
    pub is_superuser: bool,
    pub permissions: HashSet<String>,
}

impl Access {
    pub fn is_admin(&self) -> bool {
        self.is_superuser
    }

    pub fn can(&self, permission: &str) -> bool {
        self.is_superuser || self.permissions.contains(permission)
    }
}

/// Load the request's access context. Returns `None` for anonymous,
/// suspended, or pending users (and unknown/expired sessions).
pub async fn load_access(db: &Db, cx: &Cx) -> Option<Access> {
    let user = crate::auth::session::current_user(db, cx).await?;
    load_access_for_user(db, &user).await
}

/// Access context for a known user (no session required).
/// Returns `None` for suspended/pending users or unknown roles.
pub async fn load_access_for_user(db: &Db, user: &User) -> Option<Access> {
    if user.status != "active" {
        return None;
    }
    let role: Option<Role> = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
        .bind(&user.role_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    let role = role?;
    let perms: Vec<String> = sqlx::query_scalar(
        "SELECT p.key FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         WHERE rp.role_id = $1",
    )
    .bind(&role.id)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    Some(Access {
        user: user.clone(),
        role_id: role.id.clone(),
        role_name: role.name.clone(),
        is_superuser: role.is_superuser == 1,
        permissions: perms.into_iter().collect(),
    })
}

/// Visibility gate shared by manuals and videos.
/// `tier`: public | members | staff | premium.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Allow,
    LoginRequired,
    Forbidden,
}

/// Pure visibility decision (unit-tested).
/// `staff_perm` / `premium_perm` are the permissions guarding those tiers
/// (e.g. `manuals.view_staff` / `premium.access`).
pub fn visibility_gate(
    access: Option<&Access>,
    tier: &str,
    staff_perm: &str,
    premium_perm: &str,
) -> Gate {
    match tier {
        "members" => {
            if access.is_some() {
                Gate::Allow
            } else {
                Gate::LoginRequired
            }
        }
        "staff" => match access {
            Some(a) if a.can(staff_perm) => Gate::Allow,
            Some(_) => Gate::Forbidden,
            None => Gate::LoginRequired,
        },
        "premium" => match access {
            Some(a) if a.can(premium_perm) => Gate::Allow,
            Some(_) => Gate::Forbidden,
            None => Gate::LoginRequired,
        },
        _ => Gate::Allow,
    }
}

/// Tiers visible to `access`, for `visibility = ANY($n)` list queries.
/// (Kept for tier-filtered listings; manuals index now lists all titles.)
#[allow(dead_code)]
pub fn visible_tiers(access: Option<&Access>, staff_perm: &str, premium_perm: &str) -> Vec<String> {
    let mut tiers = vec!["public".to_string()];
    if access.is_some() {
        tiers.push("members".to_string());
    }
    if access.map(|a| a.can(staff_perm)).unwrap_or(false) {
        tiers.push("staff".to_string());
    }
    if access.map(|a| a.can(premium_perm)).unwrap_or(false) {
        tiers.push("premium".to_string());
    }
    tiers
}

/// Well-known role keys (single source; DB rows reference these).
pub mod role_keys {
    pub const ADMIN: &str = "admin";
    pub const EDITOR: &str = "editor";
    pub const MEMBER: &str = "member";
}

/// Default role for OAuth-provisioned accounts.
pub const DEFAULT_ROLE: &str = role_keys::MEMBER;

/// User account statuses.
pub mod user_status {
    pub const ACTIVE: &str = "active";
    pub const PENDING: &str = "pending";
    #[allow(dead_code)]
    pub const SUSPENDED: &str = "suspended";
}

/// Display label for well-known roles (custom roles show their key).
pub fn role_label(name: &str) -> &str {
    match name {
        role_keys::ADMIN => "관리자",
        role_keys::EDITOR => "에디터",
        role_keys::MEMBER => "일반",
        other => other,
    }
}

/// Content visibility tiers. `premium` is dormant until the paid policy
/// arrives (requires `premium.access`, held by no role by default).
pub const VISIBILITY_TIERS: [(&str, &str); 4] = [
    ("public", "전체공개"),
    ("members", "회원전용"),
    ("staff", "스태프전용"),
    ("premium", "유료전용(준비중)"),
];

pub fn valid_visibility(v: &str) -> bool {
    VISIBILITY_TIERS.iter().any(|(k, _)| *k == v)
}

pub fn visibility_label(tier: &str) -> &str {
    VISIBILITY_TIERS
        .iter()
        .find(|(k, _)| *k == tier)
        .map(|(_, label)| *label)
        .unwrap_or("전체공개")
}

/// Permission keys (single source; call sites use `perms::*`, never literals).
pub mod perms {
    pub const MANUALS_VIEW_STAFF: &str = "manuals.view_staff";
    pub const MANUALS_MANAGE: &str = "manuals.manage";
    pub const VIDEOS_VIEW_STAFF: &str = "videos.view_staff";
    pub const YOUTUBE_MANAGE: &str = "youtube.manage";
    pub const POSTS_MODERATE: &str = "posts.moderate";
    pub const COMMUNITY_MANAGE: &str = "community.manage";
    pub const USERS_MANAGE: &str = "users.manage";
    pub const ROLES_MANAGE: &str = "roles.manage";
    pub const EVENTS_MANAGE: &str = "events.manage";
    pub const SETTINGS_MANAGE: &str = "settings.manage";
    pub const PREMIUM_ACCESS: &str = "premium.access";
}

/// Permissions granting access to the admin console (any one suffices;
/// superusers always pass). `premium.access` is deliberately excluded:
/// it gates content, not administration.
pub const ADMIN_PERMS: [&str; 10] = [
    perms::MANUALS_VIEW_STAFF,
    perms::MANUALS_MANAGE,
    perms::VIDEOS_VIEW_STAFF,
    crate::auth::access::perms::YOUTUBE_MANAGE,
    crate::auth::access::perms::POSTS_MODERATE,
    crate::auth::access::perms::COMMUNITY_MANAGE,
    crate::auth::access::perms::USERS_MANAGE,
    crate::auth::access::perms::ROLES_MANAGE,
    crate::auth::access::perms::EVENTS_MANAGE,
    crate::auth::access::perms::SETTINGS_MANAGE,
];

/// Whether this identity may enter the admin console at all.
pub fn can_access_admin(access: &Access) -> bool {
    access.is_superuser || ADMIN_PERMS.iter().any(|p| access.can(p))
}

/// Load access for anyone allowed into the admin console at all.
pub async fn require_admin_area(db: &Db, cx: &Cx) -> Option<Access> {
    let access = load_access(db, cx).await?;
    if can_access_admin(&access) {
        Some(access)
    } else {
        None
    }
}

/// Load access and require one permission (superusers bypass).
/// Returns `None` for anonymous/inactive/unauthorized callers.
pub async fn require_perm(db: &Db, cx: &Cx, permission: &str) -> Option<Access> {
    let access = load_access(db, cx).await?;
    if access.can(permission) {
        Some(access)
    } else {
        None
    }
}

/// Fire-and-forget audit record. Never fails the caller.
pub async fn audit(
    db: &Db,
    cx: &Cx,
    actor_id: &str,
    action: &str,
    target_type: &str,
    target_id: &str,
    detail: &str,
) {
    let ip = crate::auth::session::client_ip(cx);
    let actor: Option<&str> = if actor_id.is_empty() {
        None
    } else {
        Some(actor_id)
    };
    sqlx::query(
        "INSERT INTO audit_logs (actor_id, action, target_type, target_id, detail, ip)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(actor)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(detail)
    .bind(&ip)
    .execute(db)
    .await
    .ok();
}

/// Account-status gate for all login paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginDecision {
    Allow,
    Pending,
    Suspended,
}

pub fn login_decision(user: &User) -> LoginDecision {
    match user.status.as_str() {
        user_status::ACTIVE => LoginDecision::Allow,
        user_status::PENDING => LoginDecision::Pending,
        _ => LoginDecision::Suspended,
    }
}

/// Status for freshly registered users (approval workflow when enabled).
pub async fn initial_status(db: &Db) -> String {
    let required =
        crate::services::site_settings::get_setting(db, "require_approval").await == "true";
    if required {
        user_status::PENDING.to_string()
    } else {
        user_status::ACTIVE.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn access_with(perms: &[&str]) -> Access {
        Access {
            user: User {
                id: "u".into(),
                email: None,
                password_hash: None,
                github_id: None,
                google_id: None,
                username: "u".into(),
                display_name: None,
                avatar_url: None,
                role_id: "member".into(),
                status: "active".into(),
                totp_secret: String::new(),
                totp_enabled: 0,
                bio: String::new(),
                theme_preference: "light".into(),
                points: 0,
                locale: "ko".into(),
                last_login_at: None,
                created_at: String::new(),
                updated_at: String::new(),
            },
            role_id: "member".into(),
            role_name: "member".into(),
            is_superuser: false,
            permissions: perms.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn gate_matrix() {
        use Gate::*;
        let anon: Option<&Access> = None;
        let member = access_with(&[]);
        let staff = access_with(&[crate::auth::access::perms::MANUALS_VIEW_STAFF]);
        let m = Some(&member);
        let s = Some(&staff);
        assert_eq!(
            visibility_gate(
                anon,
                "public",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Allow
        );
        assert_eq!(
            visibility_gate(
                anon,
                "members",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            LoginRequired
        );
        assert_eq!(
            visibility_gate(
                m,
                "members",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Allow
        );
        assert_eq!(
            visibility_gate(
                anon,
                "staff",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            LoginRequired
        );
        assert_eq!(
            visibility_gate(
                m,
                "staff",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Forbidden
        );
        assert_eq!(
            visibility_gate(
                s,
                "staff",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Allow
        );
        assert_eq!(
            visibility_gate(
                m,
                "premium",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Forbidden
        );
        assert_eq!(
            visibility_gate(
                anon,
                "weird-tier",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Allow
        );
    }

    #[test]
    fn tier_lists() {
        let anon: Option<&Access> = None;
        let member = access_with(&[]);
        let staff = access_with(&[crate::auth::access::perms::MANUALS_VIEW_STAFF]);
        assert_eq!(
            visible_tiers(
                anon,
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            vec!["public"]
        );
        assert_eq!(
            visible_tiers(
                Some(&member),
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            vec!["public".to_string(), "members".to_string()]
        );
        assert_eq!(
            visible_tiers(
                Some(&staff),
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            vec![
                "public".to_string(),
                "members".to_string(),
                "staff".to_string()
            ]
        );
    }

    #[test]
    fn superuser_bypasses() {
        let mut a = access_with(&[]);
        a.is_superuser = true;
        assert!(a.can("anything.at.all"));
        assert_eq!(
            visibility_gate(
                Some(&a),
                "premium",
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS
            ),
            Gate::Allow
        );
    }
}
