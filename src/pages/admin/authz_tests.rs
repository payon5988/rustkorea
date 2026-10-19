//! Security regressions: admin mutations require a permission (not just a
//! CSRF token every member holds), and TOTP accounts cannot skip the second
//! factor through the public login form.

use crate::db::Db;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Body, Router, RouterBuilderDiscoverExt, request::Request},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

const CSRF: &str = "authz-test-csrf";

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

async fn post(router: &Router, cookie: &str, path: &str, body: String) -> String {
    let response = router
        .handle(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await;
    response
        .headers()
        .get("location")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_else(|| format!("status {}", response.status()))
}

fn form(pairs: &[(&str, &str)]) -> String {
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish()
}

/// Insert a user with a live full session; returns the session cookie.
async fn user_with_session(pool: &Db, id: &str, role: &str) -> String {
    sqlx::query("INSERT INTO users (id, username, role_id) VALUES ($1, $1, $2)")
        .bind(id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let token = topcoat::session::Token::random();
    let hash: String = token.hash().iter().map(|b| format!("{b:02x}")).collect();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, csrf_token, expires_at) VALUES ($1, $2, $3, NOW() + INTERVAL '1 hour')")
        .bind(hash).bind(id).bind(CSRF).execute(pool).await.unwrap();
    format!("__Host-session={}", token.encode())
}

async fn count(pool: &Db, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

/// Run with DATABASE_URL pointing at a PostgreSQL instance whose user can
/// create databases, after `cargo build` and `topcoat asset bundle`.
#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; creates an isolated test database"]
async fn members_cannot_run_admin_mutations(pool: Db) {
    let member = user_with_session(&pool, "authz-member", "member").await;
    sqlx::query(
        "INSERT INTO posts (id, author_id, title) VALUES ('authz-post', 'authz-member', 'keep')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO events (id, creator_id, title, event_date) VALUES ('authz-event', 'authz-member', 'keep', '2099-01-01')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO reports (id, reporter_id, target_type, target_id, reason) VALUES ('authz-report', 'authz-member', 'post', 'authz-post', 'spam')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO manual_series (id, slug, title) VALUES ('authz-series', 'authz-series', 'keep')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO manual_chapters (id, series_id, slug, title, content_md) VALUES ('authz-chapter', 'authz-series', 'ch', 'keep', 'keep')")
        .execute(&pool).await.unwrap();
    let router = router(&pool);

    let csrf = [("csrf_token", CSRF)];
    let manual = |id: &'static str| {
        let mut f = vec![
            ("csrf_token", CSRF),
            ("slug", "authz-series"),
            ("title", "pwned"),
            ("description", ""),
            ("difficulty", "beginner"),
            ("estimated_minutes", "1"),
            ("visibility", "public"),
        ];
        if !id.is_empty() {
            f.push(("id", id));
        }
        form(&f)
    };
    let chapter = |id: &'static str| {
        let mut f = vec![
            ("csrf_token", CSRF),
            ("slug", "pwned"),
            ("title", "pwned"),
            ("content_md", "pwned"),
            ("visibility", "public"),
        ];
        if !id.is_empty() {
            f.push(("id", id));
        }
        form(&f)
    };
    let new_post = form(&[
        ("csrf_token", CSRF),
        ("title", "pwned"),
        ("content_md", "pwned"),
        ("category", "general"),
    ]);
    for (path, body) in [
        ("/admin/posts/authz-post/delete", form(&csrf)),
        ("/admin/posts/create", new_post),
        ("/admin/events/authz-event/delete", form(&csrf)),
        ("/admin/reports/authz-report/resolve", form(&csrf)),
        ("/admin/reports/authz-report/dismiss", form(&csrf)),
        ("/admin/manuals/create", manual("")),
        ("/admin/manuals/update", manual("authz-series")),
        ("/admin/manuals/authz-series/chapters/create", chapter("")),
        ("/admin/manuals/chapters/update", chapter("authz-chapter")),
    ] {
        assert_eq!(
            post(&router, &member, path, body).await,
            "/admin/login",
            "{path}"
        );
    }

    assert_eq!(count(&pool, "SELECT COUNT(*) FROM posts").await, 1);
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM events").await, 1);
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM reports WHERE status = 'pending'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM manual_series WHERE title = 'keep'"
        )
        .await,
        1
    );
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM manual_series").await, 1);
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM manual_chapters WHERE title = 'keep'"
        )
        .await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM manual_chapters").await,
        1
    );

    // Control: the same request still works for an administrator.
    let admin = user_with_session(&pool, "authz-admin", "admin").await;
    assert_eq!(
        post(
            &router,
            &admin,
            "/admin/reports/authz-report/resolve",
            form(&csrf)
        )
        .await,
        "/admin/reports"
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM reports WHERE status = 'resolved'"
        )
        .await,
        1
    );
}

#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; creates an isolated test database"]
async fn public_login_enforces_totp(pool: Db) {
    use argon2::password_hash::{PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = argon2::Argon2::default()
        .hash_password(b"correct horse battery", &salt)
        .unwrap()
        .to_string();
    sqlx::query("INSERT INTO users (id, email, username, password_hash, role_id, totp_secret, totp_enabled) VALUES ('authz-totp', 'totp@example.com', 'totp', $1, 'admin', 'JBSWY3DPEHPK3PXP', 1)")
        .bind(hash).execute(&pool).await.unwrap();
    let router = router(&pool);

    let body = form(&[
        ("csrf_token", "seed"),
        ("email", "totp@example.com"),
        ("password", "correct horse battery"),
    ]);
    assert_eq!(
        post(&router, "csrf_seed=seed", "/auth/login", body).await,
        crate::auth::session::TWOFA_ROUTE
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sessions WHERE user_id = 'authz-totp' AND pending_2fa = 1"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sessions WHERE user_id = 'authz-totp' AND pending_2fa = 0"
        )
        .await,
        0
    );
}
