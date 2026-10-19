use super::*;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Body, Router, RouterBuilderDiscoverExt, request::Request, to_bytes},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

#[test]
fn links_and_active_paths() {
    for href in [
        "/",
        "/blog",
        "/search?q=rust",
        "https://example.com/path",
        "http://example.com",
    ] {
        assert!(valid_href(href), "{href}");
    }
    for href in [
        "",
        "//evil.example",
        "/\\evil.example",
        "javascript:alert(1)",
        "data:text/html,x",
        "https://",
        "https://example.com/\n",
        "relative",
    ] {
        assert!(!valid_href(href), "{href}");
    }
    assert!(is_active("/blog", "/blog/post"));
    assert!(is_active("/blog/", "/blog"));
    assert!(!is_active("/blog", "/blogger"));
    assert!(is_active("/", "/"));
    assert!(!is_active("/", "/blog"));
    assert!(!is_active("https://example.com/blog", "/blog"));
}

fn router(pool: &Db) -> Router {
    let assets = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets");
    Router::builder()
        .app_context(pool.clone())
        .assets(AssetBundle::load_dir(assets).expect("run cargo build and topcoat asset bundle"))
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

async fn session(pool: &Db, id: &str, role: &str) -> String {
    sqlx::query("INSERT INTO users (id, username, role_id) VALUES ($1,$1,$2)")
        .bind(id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let token = topcoat::session::Token::random();
    let hash: String = token.hash().iter().map(|b| format!("{b:02x}")).collect();
    sqlx::query("INSERT INTO sessions (token_hash,user_id,csrf_token,expires_at) VALUES ($1,$2,'menu-csrf',NOW()+INTERVAL '1 hour')")
        .bind(hash).bind(id).execute(pool).await.unwrap();
    format!("__Host-session={}", token.encode())
}

async fn request(
    router: &Router,
    cookie: &str,
    method: &str,
    path: &str,
    pairs: &[(&str, &str)],
) -> topcoat::router::response::Response {
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs.iter().copied())
        .finish();
    router
        .handle(
            Request::builder()
                .method(method)
                .uri(path)
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
}
async fn html(router: &Router, cookie: &str, path: &str) -> String {
    let response = request(router, cookie, "GET", path, &[]).await;
    assert_eq!(response.status(), 200);
    String::from_utf8(
        to_bytes(response.into_body(), 8 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}
async fn post(router: &Router, cookie: &str, action: &str, pairs: &[(&str, &str)], location: &str) {
    let mut pairs = pairs.to_vec();
    pairs.push(("csrf_token", "menu-csrf"));
    let response = request(
        router,
        cookie,
        "POST",
        &format!("/admin/menus/{action}"),
        &pairs,
    )
    .await;
    assert_eq!(response.status(), 303);
    assert_eq!(response.headers().get("location").unwrap(), location);
}
fn nav<'a>(html: &'a str, marker: &str) -> &'a str {
    html.split_once(marker)
        .expect(marker)
        .1
        .split_once("</nav>")
        .unwrap()
        .0
}

#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; uses an isolated test DB"]
async fn managed_menus_render_and_obey_admin_changes(pool: Db) {
    assert_eq!(visible_items(&pool, "mobile", None).await.unwrap().len(), 5);
    assert_eq!(
        visible_items(&pool, "footer", None).await.unwrap().len(),
        12
    );
    let admin = session(&pool, "menu-admin", "admin").await;
    let member = session(&pool, "menu-member", "member").await;
    let router = router(&pool);
    let page = html(&router, "", "/blog").await;
    for marker in [
        "class=\"desktop-nav\"",
        "aria-label=\"모바일 전체 메뉴\"",
        "aria-label=\"모바일 하단 탭\"",
        "aria-label=\"푸터 서비스\"",
        "aria-label=\"모바일 푸터\"",
    ] {
        assert!(nav(&page, marker).contains("href=\"/blog\""), "{marker}");
    }
    assert!(nav(&page, "aria-label=\"모바일 하단 탭\"").contains("aria-current=\"page\""));
    let manager = html(&router, &admin, "/admin/menus").await;
    assert!(manager.contains("모바일 하단 탭"));
    assert!(manager.contains("name=\"footer_group\""));

    for section in ["header", "footer", "mobile"] {
        post(
            &router,
            &admin,
            "add",
            &[
                ("label", "메뉴 회귀"),
                ("href", "/menu-regression"),
                ("section", section),
                ("position", "0"),
                ("footer_group", "links"),
            ],
            "/admin/menus?added=1",
        )
        .await;
        post(
            &router,
            &admin,
            "add",
            &[
                ("label", "중복"),
                ("href", "/menu-regression"),
                ("section", section),
            ],
            "/admin/menus?error=exists",
        )
        .await;
        let id: String = sqlx::query_scalar(
            "SELECT id FROM nav_items WHERE href='/menu-regression' AND section=$1",
        )
        .bind(section)
        .fetch_one(&pool)
        .await
        .unwrap();
        post(
            &router,
            &admin,
            "update",
            &[
                ("item_id", &id),
                ("label", "변경 메뉴"),
                ("href", "/menu-changed"),
                ("footer_group", "participate"),
            ],
            "/admin/menus?updated=1",
        )
        .await;
        let page = html(&router, "", "/blog").await;
        let marker = match section {
            "header" => "aria-label=\"모바일 전체 메뉴\"",
            "footer" => "aria-label=\"푸터 참여\"",
            _ => "aria-label=\"모바일 하단 탭\"",
        };
        assert!(nav(&page, marker).contains("변경 메뉴"));
        post(
            &router,
            &admin,
            "move",
            &[("item_id", &id), ("direction", "down")],
            "/admin/menus?moved=1",
        )
        .await;
        post(
            &router,
            &admin,
            "permission",
            &[("item_id", &id), ("required_permission", "settings.manage")],
            "/admin/menus?saved=1",
        )
        .await;
        assert!(!html(&router, "", "/blog").await.contains("/menu-changed"));
        assert!(
            !html(&router, &member, "/blog")
                .await
                .contains("/menu-changed")
        );
        assert!(
            html(&router, &admin, "/blog")
                .await
                .contains("/menu-changed")
        );
        post(
            &router,
            &admin,
            "toggle",
            &[("item_id", &id)],
            "/admin/menus?toggled=1",
        )
        .await;
        assert!(
            !html(&router, &admin, "/blog")
                .await
                .contains("/menu-changed")
        );
        post(
            &router,
            &admin,
            "delete",
            &[("item_id", &id)],
            "/admin/menus?deleted=1",
        )
        .await;
    }
    for (href, section, error) in [
        ("javascript:alert(1)", "footer", "href"),
        ("/ok", "invalid", "section"),
    ] {
        post(
            &router,
            &admin,
            "add",
            &[("label", "invalid"), ("href", href), ("section", section)],
            &format!("/admin/menus?error={error}"),
        )
        .await;
    }
    post(
        &router,
        &admin,
        "delete",
        &[("item_id", "missing")],
        "/admin/menus?error=missing",
    )
    .await;
    let response = request(
        &router,
        &member,
        "POST",
        "/admin/menus/delete",
        &[("item_id", "missing"), ("csrf_token", "menu-csrf")],
    )
    .await;
    assert_eq!(response.headers().get("location").unwrap(), "/admin/login");
    let response = request(
        &router,
        &admin,
        "POST",
        "/admin/menus/delete",
        &[("item_id", "missing"), ("csrf_token", "bad")],
    )
    .await;
    assert_eq!(response.headers().get("location").unwrap(), "/admin/login");
    sqlx::query(
        "ALTER TABLE nav_items ADD CONSTRAINT menu_write_failure CHECK(label <> 'reject-menu')",
    )
    .execute(&pool)
    .await
    .unwrap();
    post(
        &router,
        &admin,
        "add",
        &[
            ("label", "reject-menu"),
            ("href", "/reject"),
            ("section", "footer"),
        ],
        "/admin/menus?error=database",
    )
    .await;
    assert!(
        html(&router, &admin, "/admin/menus?error=database")
            .await
            .contains("저장하지 못했습니다")
    );
    sqlx::query("UPDATE nav_items SET visible=0")
        .execute(&pool)
        .await
        .unwrap();
    let hidden = html(&router, "", "/blog").await;
    assert!(
        !hidden.contains("href=\"/manuals\""),
        "hidden menus must not fall back to fixed links"
    );
    sqlx::query("DELETE FROM nav_items")
        .execute(&pool)
        .await
        .unwrap();
    crate::services::seed::seed_database(&pool).await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM nav_items")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[sqlx::test(migrations = false)]
#[ignore = "requires PostgreSQL DATABASE_URL; uses an isolated test DB"]
async fn migration_preserves_existing_menus_and_tied_order_can_move(pool: Db) {
    for migration in sqlx::migrate!("./migrations")
        .iter()
        .filter(|m| m.version < 17)
    {
        sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO nav_items (id,label,href,section,visible) VALUES ('old-footer','기존','/old','footer',0),('old-header','기존 상단','/custom','header',0)").execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!("../../migrations/017_managed_navigation.sql"))
        .execute(&pool)
        .await
        .unwrap();
    // 018 only adds English columns; NavItem reads them.
    sqlx::raw_sql(include_str!("../../migrations/018_english_content.sql"))
        .execute(&pool)
        .await
        .unwrap();
    let footer = sqlx::query_as::<_, NavItem>("SELECT * FROM nav_items WHERE section='footer'")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(footer.len(), 1);
    assert_eq!(footer[0].href, "/old");
    assert_eq!(footer[0].visible, 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM nav_items WHERE section='header'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    sqlx::query("INSERT INTO nav_items (id,label,href,section,position) VALUES ('a','A','/a','footer',0),('b','B','/b','footer',0)").execute(&pool).await.unwrap();
    assert!(move_item(&pool, "b", "up").await.unwrap());
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM nav_items WHERE section='footer' ORDER BY position,id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(ids, vec!["b", "a", "old-footer"]);
}
