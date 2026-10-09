use topcoat::{Result, context::Cx, router::route};

use crate::services::TypedBody;

fn to_iso_date(dt_str: &str) -> String {
    dt_str.split_whitespace().next().unwrap_or("").to_string()
}

#[route(GET "/sitemap.xml")]
pub async fn sitemap_xml(cx: &Cx) -> Result<TypedBody> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let domain = crate::config::site_url();

    let mut entries: Vec<String> = vec![
        format!(
            r#"  <url><loc>{domain}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/community</loc><changefreq>daily</changefreq><priority>0.9</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/manuals</loc><changefreq>weekly</changefreq><priority>0.9</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/youtube</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/ranking</loc><changefreq>weekly</changefreq><priority>0.6</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/events</loc><changefreq>weekly</changefreq><priority>0.7</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/search</loc><changefreq>monthly</changefreq><priority>0.5</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/privacy</loc><changefreq>yearly</changefreq><priority>0.3</priority></url>"#
        ),
        format!(
            r#"  <url><loc>{domain}/terms</loc><changefreq>yearly</changefreq><priority>0.3</priority></url>"#
        ),
    ];

    let posts: Vec<(String, String)> =
        sqlx::query_as("SELECT id, created_at FROM posts ORDER BY created_at DESC LIMIT 500")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    for (id, date) in &posts {
        let lastmod = to_iso_date(date);
        if lastmod.is_empty() {
            entries.push(format!(r#"  <url><loc>{domain}/community/{id}</loc><changefreq>monthly</changefreq><priority>0.7</priority></url>"#));
        } else {
            entries.push(format!(r#"  <url><loc>{domain}/community/{id}</loc><lastmod>{lastmod}</lastmod><changefreq>monthly</changefreq><priority>0.7</priority></url>"#));
        }
    }

    let slugs: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM manual_series WHERE visibility = 'public'")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    for slug in &slugs {
        entries.push(format!(r#"  <url><loc>{domain}/manuals/{slug}</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>"#));
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{}
</urlset>"#,
        entries.join("\n")
    );

    Ok(TypedBody {
        content_type: "application/xml; charset=utf-8",
        body: xml,
    })
}
