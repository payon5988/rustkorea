use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub github_id: Option<i64>,
    pub google_id: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role_id: String,
    pub status: String,
    pub totp_secret: String,
    pub totp_enabled: i32,
    pub bio: String,
    pub theme_preference: String,
    pub points: i64,
    pub locale: String,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ManualSeries {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    pub difficulty: String,
    pub estimated_minutes: i32,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ManualChapter {
    pub id: String,
    pub series_id: String,
    pub slug: String,
    pub title: String,
    pub content_md: String,
    pub content_html: String,
    pub chapter_index: i32,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingProgress {
    pub id: String,
    pub user_id: String,
    pub chapter_id: String,
    pub scroll_pct: i32,
    pub completed: i32,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct YoutubeVideo {
    pub id: String,
    pub video_id: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub published_at: String,
    pub tags: String,
    pub created_at: String,
}

impl YoutubeVideo {
    #[allow(dead_code)]
    pub fn tags_parsed(&self) -> Vec<String> {
        serde_json::from_str(&self.tags).unwrap_or_default()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub content_md: String,
    pub content_html: String,
    pub tags: String,
    pub category: String,
    pub view_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl Post {
    #[allow(dead_code)]
    pub fn tags_parsed(&self) -> Vec<String> {
        serde_json::from_str(&self.tags).unwrap_or_default()
    }
}

/// Post row with resolved author display name for list surfaces.
/// `username` falls back to `author_id` when the user row is gone.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PostListItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub content_md: String,
    pub view_count: i64,
    pub created_at: String,
    pub username: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub detail: String,
    pub ip: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NavItem {
    pub id: String,
    pub label: String,
    pub href: String,
    pub position: i32,
    pub visible: i32,
    pub section: String,
    pub required_permission: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_system: i32,
    pub is_superuser: i32,
    pub position: i32,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: String,
    pub key: String,
    pub category: String,
    pub description: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SiteSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub position: i64,
    pub visible: i64,
    pub write_permission: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PostLike {
    pub id: String,
    pub post_id: String,
    pub user_id: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bookmark {
    pub id: String,
    pub post_id: String,
    pub user_id: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub noti_type: String,
    pub from_user_id: Option<String>,
    pub post_id: Option<String>,
    pub comment_id: Option<String>,
    pub message: String,
    pub is_read: i32,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub subject: String,
    pub content: String,
    pub is_read: i32,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub points_required: i64,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserBadge {
    pub user_id: String,
    pub badge_id: String,
    pub awarded_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PointHistory {
    pub id: String,
    pub user_id: String,
    pub points: i64,
    pub reason: String,
    pub reference_id: Option<String>,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Report {
    pub id: String,
    pub reporter_id: String,
    pub target_type: String,
    pub target_id: String,
    pub reason: String,
    pub status: String,
    pub admin_note: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: String,
    pub creator_id: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub event_date: String,
    pub event_time: String,
    pub max_participants: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRegistration {
    pub event_id: String,
    pub user_id: String,
    pub registered_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Upload {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailSubscription {
    pub id: String,
    pub email: String,
    pub sub_type: String,
    pub is_active: i32,
    pub token: String,
    pub created_at: String,
}
