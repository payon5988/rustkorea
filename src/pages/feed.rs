use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use topcoat::{Result, context::Cx, router::route};

use crate::services::TypedBody;

/// Feed caps (single place).
pub const FEED_LIMIT: i64 = 20;
pub const FEED_DESC_MAX: usize = 500;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn to_rfc822(dt_str: &str) -> String {
    NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S")
        .map(|ndt| {
            let dt = ndt.and_utc();
            let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            let months = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            format!(
                "{}, {:02} {} {} {:02}:{:02}:{:02} +0900",
                days[dt.weekday().num_days_from_sunday() as usize],
                dt.day(),
                months[dt.month0() as usize],
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second(),
            )
        })
        .unwrap_or_else(|_| Utc::now().to_rfc3339())
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct FeedPost {
    id: String,
    title: String,
    content_html: String,
    author_id: String,
    created_at: String,
    username: String,
}

#[route(GET "/feed.xml")]
pub async fn feed_rss(cx: &Cx) -> Result<TypedBody> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);

    let site_title = crate::services::site_settings::get_setting(db, "site_title").await;
    let site_title = if site_title.is_empty() {
        "Rust Korea Hub".to_string()
    } else {
        site_title
    };
    let site_description =
        crate::services::site_settings::get_setting(db, "site_description").await;
    let site_description = if site_description.is_empty() {
        "한국 Rust 생태계의 단일 진실 공급원".to_string()
    } else {
        site_description
    };

    let domain = crate::config::site_url();

    let now_rfc822 = {
        let now = Utc::now();
        let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        format!(
            "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
            days[now.weekday().num_days_from_sunday() as usize],
            now.day(),
            months[now.month0() as usize],
            now.year(),
            now.hour(),
            now.minute(),
            now.second(),
        )
    };

    let posts: Vec<FeedPost> = sqlx::query_as::<_, FeedPost>(
        "SELECT p.id, p.title, p.content_html, p.author_id, p.created_at,
                COALESCE(u.username, p.author_id) AS username
         FROM posts p LEFT JOIN users u ON u.id = p.author_id
         ORDER BY p.created_at DESC LIMIT $1",
    )
    .bind(FEED_LIMIT)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let items: String = posts
        .iter()
        .map(|p| {
            let desc = xml_escape(&p.content_html)
                .chars()
                .take(FEED_DESC_MAX)
                .collect::<String>();
            let title = xml_escape(&p.title);
            let author = xml_escape(&p.username);
            let pub_date = to_rfc822(&p.created_at);
            let mail_domain = crate::config::mail_domain();
            format!(
                r#"    <item>
      <title>{title}</title>
      <link>{domain}/community/{id}</link>
      <description>{desc}</description>
      <author>{author}@{mail_domain}</author>
      <guid isPermaLink="true">{domain}/community/{id}</guid>
      <pubDate>{pub_date}</pubDate>
    </item>"#,
                title = title,
                domain = domain,
                id = p.id,
                desc = desc,
                author = author,
                mail_domain = mail_domain,
                pub_date = pub_date,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let channel_title = xml_escape(&site_title);
    let channel_desc = xml_escape(&site_description);

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>{channel_title}</title>
  <link>{domain}</link>
  <description>{channel_desc}</description>
  <language>ko</language>
  <lastBuildDate>{now_rfc822}</lastBuildDate>
  <atom:link href="{domain}/feed.xml" rel="self" type="application/rss+xml" />
{items}
</channel>
</rss>"#,
        channel_title = channel_title,
        domain = domain,
        channel_desc = channel_desc,
        now_rfc822 = now_rfc822,
        items = items,
    );

    Ok(TypedBody {
        content_type: "application/rss+xml; charset=utf-8",
        body: xml,
    })
}
