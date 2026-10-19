use super::*;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Body, BodyLimit, Router, RouterBuilderDiscoverExt, request::Request, to_bytes},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

#[test]
fn image_urls_preserve_upload_paths_and_external_urls() {
    for url in [
        "/uploads/20261017000000-abcdef12.png",
        "/uploads/shot.JPG",
        "https://example.com/image.webp",
        "http://example.com/image.jpg",
    ] {
        assert_eq!(clean_image_url(&format!(" {url} ")), url);
    }
    // The separate preview-link validation must still reject local paths.
    assert_eq!(clean_url("/uploads/shot.png"), "");
}

#[test]
fn image_urls_reject_unsafe_or_unserved_paths() {
    for url in [
        "",
        "/uploads/../shot.png",
        "/uploads/%2e%2e/shot.png",
        "/uploads/nested/shot.png",
        "/uploads/shot.png?x=1",
        "/uploads/shot.svg",
        "/uploads/shot.png\\other",
        "//example.com/shot.png",
        "/other/shot.png",
        "javascript:alert(1)",
        "data:image/png;base64,AAAA",
    ] {
        assert_eq!(clean_image_url(url), "", "accepted {url}");
    }
}

fn product_body(slug: &str, id: &str, thumbnail: &str, main: &str) -> String {
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("csrf_token", "product-test-csrf"),
            ("id", id),
            ("slug", slug),
            ("title", "Image regression"),
            ("summary", ""),
            ("description_md", ""),
            ("category_id", ""),
            ("price", ""),
            ("preview_url", "https://example.com"),
            ("video_id", ""),
            ("techs", "Rust"),
            ("thumbnail_url", thumbnail),
            ("main_image_url", main),
            ("visibility", "public"),
        ])
        .finish()
}

async fn send(
    router: &Router,
    cookie: &str,
    method: &str,
    path: &str,
    content_type: &str,
    body: impl Into<Body>,
) -> topcoat::router::response::Response {
    router
        .handle(
            Request::builder()
                .method(method)
                .uri(path)
                .header("cookie", cookie)
                .header("content-type", content_type)
                .body(body.into())
                .unwrap(),
        )
        .await
}

async fn upload(
    router: &Router,
    cookie: &str,
    id: &str,
    target: &str,
    size: usize,
    valid_png: bool,
) -> topcoat::router::response::Response {
    let mut body = Vec::new();
    for (name, value) in [
        ("csrf_token", "product-test-csrf"),
        ("product_id", id),
        ("target", target),
    ] {
        body.extend_from_slice(
            format!(
                "--image-test\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
            )
            .as_bytes(),
        );
    }
    body.extend_from_slice(b"--image-test\r\nContent-Disposition: form-data; name=\"file\"; filename=\"image.png\"\r\nContent-Type: image/png\r\n\r\n");
    let start = body.len();
    body.resize(start + size, 0);
    if valid_png {
        body[start..start + 8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    }
    body.extend_from_slice(b"\r\n--image-test--\r\n");
    send(
        router,
        cookie,
        "POST",
        "/admin/products/upload",
        "multipart/form-data; boundary=image-test",
        body,
    )
    .await
}

/// Run with DATABASE_URL pointing at a PostgreSQL instance whose user can
/// create databases, after `cargo build` and `topcoat asset bundle`.
/// SQLx creates and removes a separate migrated test DB.
#[sqlx::test]
#[ignore = "requires PostgreSQL DATABASE_URL and bundled assets; creates an isolated test database"]
async fn uploaded_product_images_survive_save_and_are_served(pool: Db) {
    use crate::services::uploads::{MAX_UPLOAD_BYTES, MAX_UPLOAD_REQUEST_BYTES};

    sqlx::query(
        "INSERT INTO users (id, username, role_id) VALUES ('image-admin', 'image-admin', 'admin')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let token = topcoat::session::Token::random();
    let hash: String = token.hash().iter().map(|b| format!("{b:02x}")).collect();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, csrf_token, expires_at) VALUES ($1, 'image-admin', 'product-test-csrf', NOW() + INTERVAL '1 hour')")
        .bind(hash).execute(&pool).await.unwrap();
    let cookie = format!("__Host-session={}", token.encode());
    let assets_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets");
    let router = Router::builder()
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
        .layer(BodyLimit::max(MAX_UPLOAD_REQUEST_BYTES).at("/admin/products/upload"))
        .runtime()
        .discover()
        .build();

    let response = send(
        &router,
        &cookie,
        "POST",
        "/admin/products/create",
        "application/x-www-form-urlencoded",
        product_body(
            "image-regression",
            "",
            "/uploads/existing.png",
            "https://example.com/main.jpg",
        ),
    )
    .await;
    assert_eq!(response.headers()["location"], "/admin/products");
    let product: Product = sqlx::query_as("SELECT * FROM products WHERE slug = 'image-regression'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(product.thumbnail_url, "/uploads/existing.png");
    assert_eq!(product.main_image_url, "https://example.com/main.jpg");

    // Exercise the old 2MiB failure boundary and the advertised 5MiB maximum.
    for (target, size) in [("thumbnail", 3 * 1024 * 1024), ("main", MAX_UPLOAD_BYTES)] {
        let response = upload(&router, &cookie, &product.id, target, size, true).await;
        assert_eq!(
            response.headers()["location"],
            format!("/admin/products/edit?id={}", product.id)
        );
    }
    let uploaded: Product = sqlx::query_as("SELECT * FROM products WHERE id = $1")
        .bind(&product.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(uploaded.thumbnail_url.starts_with("/uploads/"));
    assert!(uploaded.main_image_url.starts_with("/uploads/"));

    for _ in 0..2 {
        let response = send(
            &router,
            &cookie,
            "POST",
            "/admin/products/update",
            "application/x-www-form-urlencoded",
            product_body(
                &uploaded.slug,
                &uploaded.id,
                &uploaded.thumbnail_url,
                &uploaded.main_image_url,
            ),
        )
        .await;
        assert_eq!(
            response.headers()["location"],
            format!("/admin/products/edit?id={}", product.id)
        );
        let saved: Product = sqlx::query_as("SELECT * FROM products WHERE id = $1")
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(saved.thumbnail_url, uploaded.thumbnail_url);
        assert_eq!(saved.main_image_url, uploaded.main_image_url);
    }
    for (path, expected_images) in [
        (
            format!("/admin/products/edit?id={}", product.id),
            vec![&uploaded.thumbnail_url, &uploaded.main_image_url],
        ),
        (
            "/vibecoding/image-regression".into(),
            vec![&uploaded.main_image_url],
        ),
        ("/vibecoding".into(), vec![&uploaded.thumbnail_url]),
    ] {
        let response = send(&router, &cookie, "GET", &path, "text/plain", "").await;
        assert_eq!(response.status(), 200);
        let html = String::from_utf8(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        for url in expected_images {
            assert!(html.contains(url), "{path} did not render {url}");
        }
    }
    for (url, size) in [
        (&uploaded.thumbnail_url, 3 * 1024 * 1024),
        (&uploaded.main_image_url, MAX_UPLOAD_BYTES),
    ] {
        let response = send(&router, "", "GET", url, "text/plain", "").await;
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "image/png");
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .len(),
            size
        );
        std::fs::remove_file(
            std::path::Path::new(&crate::config::upload_dir())
                .join(url.trim_start_matches("/uploads/")),
        )
        .unwrap();
    }
    for (size, valid_png, code) in [
        (0, false, "size"),
        (MAX_UPLOAD_BYTES + 1, true, "size"),
        (MAX_UPLOAD_REQUEST_BYTES + 1, true, "size"),
        (32, false, "filetype"),
    ] {
        let response = upload(&router, &cookie, &product.id, "main", size, valid_png).await;
        assert!(
            response.headers()["location"]
                .to_str()
                .unwrap()
                .ends_with(&format!("&error={code}"))
        );
    }
    let saved: Product = sqlx::query_as("SELECT * FROM products WHERE id = $1")
        .bind(&product.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(saved.main_image_url, uploaded.main_image_url);

    // A DB failure must return a visible error instead of reporting success.
    // Use a URL-only multipart form, including the browser's empty file field.
    sqlx::query("ALTER TABLE products ADD CONSTRAINT reject_test_image CHECK (main_image_url <> 'https://example.com/rejected.png')")
        .execute(&pool).await.unwrap();
    let body = format!(
        "--image-test\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\nproduct-test-csrf\r\n\
         --image-test\r\nContent-Disposition: form-data; name=\"product_id\"\r\n\r\n{}\r\n\
         --image-test\r\nContent-Disposition: form-data; name=\"target\"\r\n\r\nmain\r\n\
         --image-test\r\nContent-Disposition: form-data; name=\"file\"; filename=\"\"\r\n\r\n\r\n\
         --image-test\r\nContent-Disposition: form-data; name=\"url\"\r\n\r\nhttps://example.com/rejected.png\r\n\
         --image-test--\r\n",
        product.id
    );
    let response = send(
        &router,
        &cookie,
        "POST",
        "/admin/products/upload",
        "multipart/form-data; boundary=image-test",
        body,
    )
    .await;
    let location = response.headers()["location"].to_str().unwrap();
    assert!(location.ends_with("&error=upload_save"));
    let response = send(&router, &cookie, "GET", location, "text/plain", "").await;
    let html = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(html.contains(product_error_message("upload_save")));
    let saved: Product = sqlx::query_as("SELECT * FROM products WHERE id = $1")
        .bind(&product.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(saved.main_image_url, uploaded.main_image_url);
}
