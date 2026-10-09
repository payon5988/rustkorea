use crate::db::Db;

pub async fn get_setting(db: &Db, key: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM site_settings WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}
