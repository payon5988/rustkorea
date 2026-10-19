use crate::{auth::access::Access, db::Db, models::NavItem};

pub const SECTIONS: [(&str, &str); 3] = [
    ("header", "상단 메뉴"),
    ("footer", "푸터 메뉴"),
    ("mobile", "모바일 하단 탭"),
];
pub const FOOTER_GROUPS: [(&str, &str); 3] = [
    ("service", "서비스"),
    ("links", "링크"),
    ("participate", "참여"),
];

/// Public footer heading for a group in `loc` (admin uses `FOOTER_GROUPS`).
pub fn footer_group_label(group: &str, loc: crate::services::i18n::Locale) -> &'static str {
    match group {
        "links" => loc.pick("링크", "Links"),
        "participate" => loc.pick("참여", "Get Involved"),
        _ => loc.pick("서비스", "Service"),
    }
}

/// Only same-site absolute paths and explicit HTTP(S) links are navigable.
pub fn valid_href(href: &str) -> bool {
    if href.is_empty()
        || href.chars().any(|c| c.is_whitespace() || c.is_control())
        || href.contains('\\')
    {
        return false;
    }
    if href.starts_with('/') {
        return !href.starts_with("//");
    }
    reqwest::Url::parse(href)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .unwrap_or(false)
}

pub fn is_active(href: &str, path: &str) -> bool {
    if !href.starts_with('/') || href.starts_with("//") {
        return false;
    }
    let target = href.split(['?', '#']).next().unwrap_or(href);
    let target = if target == "/" {
        target
    } else {
        target.trim_end_matches('/')
    };
    path == target
        || (target != "/"
            && path
                .strip_prefix(target)
                .is_some_and(|rest| rest.starts_with('/')))
}

pub async fn visible_items(
    db: &Db,
    section: &str,
    access: Option<&Access>,
) -> Result<Vec<NavItem>, sqlx::Error> {
    let items = sqlx::query_as::<_, NavItem>(
        "SELECT * FROM nav_items WHERE section = $1 AND visible = 1 ORDER BY position, id",
    )
    .bind(section)
    .fetch_all(db)
    .await?;
    Ok(items
        .into_iter()
        .filter(|item| {
            valid_href(&item.href)
                && (item.required_permission.is_empty()
                    || access.is_some_and(|a| a.can(&item.required_permission)))
        })
        .collect())
}

/// Serialize ordering changes per section and normalize ties before swapping.
pub async fn move_item(db: &Db, id: &str, direction: &str) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    let section: Option<String> = sqlx::query_scalar("SELECT section FROM nav_items WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(section) = section else {
        return Ok(false);
    };
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(&section)
        .execute(&mut *tx)
        .await?;
    let mut ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM nav_items WHERE section = $1 ORDER BY position, id FOR UPDATE",
    )
    .bind(section)
    .fetch_all(&mut *tx)
    .await?;
    let Some(index) = ids.iter().position(|item| item == id) else {
        return Ok(false);
    };
    let neighbor = match direction {
        "up" => index.checked_sub(1),
        "down" if index + 1 < ids.len() => Some(index + 1),
        _ => None,
    };
    if let Some(neighbor) = neighbor {
        ids.swap(index, neighbor);
    }
    for (position, id) in ids.iter().enumerate() {
        sqlx::query("UPDATE nav_items SET position = $1, updated_at = NOW()::text WHERE id = $2")
            .bind(position as i32)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(true)
}

#[cfg(test)]
#[path = "navigation_tests.rs"]
mod tests;
