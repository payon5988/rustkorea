//! English (public site) regressions: language resolution, chrome strings,
//! DB content fallback, and admin English fields reaching the database.

use crate::db::Db;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Body, Router, RouterBuilderDiscoverExt, request::Request, to_bytes},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

fn router(pool: &Db) -> Router {
    let assets_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets");
    Router::builder()
        .app_context(pool.clone())
        .assets(
            AssetBundle::load_dir(assets_dir).expect("run cargo build and topcoat asset bundle"),
        )
        .cookies()
        .sessions(
            topcoat::session::SessionConfig::builder()
                .token_store(crate::auth::cookie_store::EnvCookieStore)
                .build(),
        )
        .runtime()
        .discover()
        .build()
}

/// (status, set-cookie headers joined, body)
async fn get(router: &Router, path: &str, cookie: &str) -> (u16, String, String) {
    let response = router
        .handle(
            Request::builder()
                .uri(path)
                .header("cookie", cookie)
                .body(Body::from(""))
                .unwrap(),
        )
        .await;
    let status = response.status().as_u16();
    let cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap_or("").to_string())
        .collect::<Vec<_>>()
        .join("; ");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, cookies, String::from_utf8_lossy(&body).into_owned())
}

#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; creates an isolated test database"]
async fn language_resolution_and_chrome(pool: Db) {
    let router = router(&pool);

    // No ?lang and no cookie: Korean (crawlers always get Korean here).
    let (status, _, html) = get(&router, "/", "").await;
    assert_eq!(status, 200);
    assert!(html.contains(r#"<html lang="ko""#), "default lang");
    assert!(html.contains("매뉴얼 시작하기"));
    assert!(html.contains(r#"hreflang="en""#) && html.contains("?lang=en"));

    // ?lang=en: English chrome, nav labels from migration 018, cookie saved.
    let (_, set_cookie, html) = get(&router, "/?lang=en", "").await;
    assert!(html.contains(r#"<html lang="en""#));
    assert!(html.contains("Start the manuals"));
    assert!(html.contains(">Manuals<"), "header nav label_en");
    let canonical = format!(
        r#"rel="canonical" href="{}/?lang=en""#,
        crate::config::site_url()
    );
    assert!(html.contains(&canonical), "English canonical");
    assert!(set_cookie.contains("locale=en"), "{set_cookie}");

    // Cookie alone keeps English on later pages; ?lang=ko switches back.
    let (_, _, html) = get(&router, "/community", "locale=en").await;
    assert!(html.contains(r#"<html lang="en""#) && html.contains("Community"));
    let (_, set_cookie, html) = get(&router, "/community?lang=ko", "locale=en").await;
    assert!(html.contains(r#"<html lang="ko""#));
    assert!(set_cookie.contains("locale=ko"));

    // Legal pages have a full English version with the governing notice.
    let (_, _, html) = get(&router, "/privacy?lang=en", "").await;
    assert!(html.contains("Privacy Policy") && html.contains("Korean version prevails"));
    assert!(!html.contains("&lt;strong&gt;"), "tags must not be escaped");
}

#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; creates an isolated test database"]
async fn content_uses_english_fields_with_korean_fallback(pool: Db) {
    sqlx::query(
        "INSERT INTO manual_series (id, slug, title, title_en) VALUES ('s1', 'i18n', '한국어 시리즈', 'English Series')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, title_en, content_md_en, content_html_en, chapter_index)
         VALUES ('s1', 'done', '번역된 장', '본문', '<p>한국어 본문</p>', 'Translated', 'body', '<p>English body</p>', 0),
                ('s1', 'todo', '번역 안 된 장', '본문', '<p>원문만 있음</p>', '', '', '', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let router = router(&pool);

    let (_, _, html) = get(&router, "/manuals/i18n?lang=en", "").await;
    assert!(html.contains("English Series"));
    assert!(html.contains("Translated") && html.contains("번역 안 된 장"));

    let (_, _, html) = get(&router, "/manuals/i18n/done?lang=en", "").await;
    assert!(html.contains("<p>English body</p>") && !html.contains("한국어 본문"));
    assert!(!html.contains("has not been translated"));

    let (_, _, html) = get(&router, "/manuals/i18n/todo?lang=en", "").await;
    assert!(html.contains("<p>원문만 있음</p>"));
    assert!(html.contains("has not been translated"));

    // Korean view is unchanged.
    let (_, _, html) = get(&router, "/manuals/i18n/done", "").await;
    assert!(html.contains("한국어 본문") && html.contains("번역된 장"));
}

#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; creates an isolated test database"]
async fn admin_blog_saves_english_fields(pool: Db) {
    sqlx::query(
        "INSERT INTO users (id, username, role_id) VALUES ('i18n-admin', 'i18n-admin', 'admin')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let token = topcoat::session::Token::random();
    let hash: String = token.hash().iter().map(|b| format!("{b:02x}")).collect();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, csrf_token, expires_at) VALUES ($1, 'i18n-admin', 'i18n-csrf', NOW() + INTERVAL '1 hour')")
        .bind(hash).execute(&pool).await.unwrap();
    let cookie = format!("__Host-session={}", token.encode());
    let router = router(&pool);

    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("csrf_token", "i18n-csrf"),
            ("slug", "hello-i18n"),
            ("title", "안녕"),
            ("summary", "요약"),
            ("content_md", "본문"),
            ("thumbnail_url", ""),
            ("source_url", ""),
            ("tags", ""),
            ("visibility", "public"),
            ("title_en", "Hello"),
            ("summary_en", "Summary"),
            ("content_md_en", "**Body**"),
        ])
        .finish();
    let response = router
        .handle(
            Request::builder()
                .method("POST")
                .uri("/admin/blog/create")
                .header("cookie", &cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status().as_u16(), 303);
    let row: (String, String, String) = sqlx::query_as(
        "SELECT title_en, summary_en, content_html_en FROM blog_posts WHERE slug = 'hello-i18n'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "Hello");
    assert_eq!(row.1, "Summary");
    assert!(row.2.contains("<strong>Body</strong>"), "{}", row.2);

    let (_, _, html) = get(&router, "/blog/hello-i18n?lang=en", "").await;
    assert!(html.contains("Hello") && html.contains("<strong>Body</strong>"));
}
