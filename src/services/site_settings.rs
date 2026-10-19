use crate::db::Db;

/// Public-site setting in `loc`: `<key>_en` for English when set, else `<key>`.
pub async fn get_localized(db: &Db, key: &str, loc: crate::services::i18n::Locale) -> String {
    if loc == crate::services::i18n::Locale::En {
        let en = get_setting(db, &format!("{key}_en")).await;
        if !en.trim().is_empty() {
            return en;
        }
    }
    get_setting(db, key).await
}

pub async fn get_setting(db: &Db, key: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM site_settings WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}
