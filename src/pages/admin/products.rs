use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::{Form, multipart::Multipart},
    router::{page, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::{Product, ProductCategory, ProductImage};
use crate::services::markdown::render_markdown;
use crate::util::query_param;

fn slugify(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn clean_url(raw: &str) -> String {
    let t = raw.trim().chars().take(500).collect::<String>();
    if t.starts_with("https://") || t.starts_with("http://") {
        t
    } else {
        String::new()
    }
}

fn clean_image_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(name) = trimmed.strip_prefix("/uploads/") {
        if crate::services::uploads::valid_upload_name(name) {
            return trimmed.to_string();
        }
        return String::new();
    }
    clean_url(trimmed)
}

#[cfg(test)]
#[path = "products_tests.rs"]
mod tests;

fn product_error_message(code: &str) -> &'static str {
    match code {
        "size" => "이미지 파일은 비어 있지 않은 5MiB 이하 파일이어야 합니다.",
        "filetype" => "PNG, JPG, WebP 이미지 파일만 업로드할 수 있습니다.",
        "io" => "이미지 파일을 저장하지 못했습니다. 잠시 후 다시 시도해 주세요.",
        "save" => "상품 정보를 저장하지 못했습니다. 입력 내용을 확인하고 다시 시도해 주세요.",
        "upload_save" => "이미지 정보를 저장하지 못했습니다. 다시 업로드해 주세요.",
        "missing" => "상품을 찾을 수 없습니다. 상품 목록을 확인해 주세요.",
        "invalid" => "입력 내용을 확인해 주세요.",
        "upload" => {
            "이미지를 업로드하지 못했습니다. 파일 또는 이미지 URL을 확인하고 다시 시도해 주세요."
        }
        _ => "",
    }
}

fn product_error_redirect(product_id: &str, code: &str) -> topcoat::router::error::SeeOther {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("id", product_id)
        .append_pair("error", code)
        .finish();
    let path = if product_id.is_empty() {
        "/admin/products"
    } else {
        "/admin/products/edit"
    };
    topcoat::router::error::see_other(format!("{path}?{query}"))
}

fn multipart_failure(product_id: &str, error: topcoat::Error) -> topcoat::router::error::SeeOther {
    eprintln!("[product upload] multipart failed: {error}");
    let code = if error.is::<topcoat::router::error::ContentTooLargeError>() {
        "size"
    } else {
        "upload"
    };
    product_error_redirect(product_id, code)
}

fn clean_video_id(raw: &str) -> String {
    let t = raw.trim().to_string();
    if t.len() == 11
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        t
    } else {
        String::new()
    }
}

async fn require_products(cx: &Cx) -> bool {
    let adb: &crate::db::Db = topcoat::context::app_context(cx);
    crate::auth::access::require_perm(adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
        .await
        .is_none()
}

#[derive(serde::Deserialize)]
pub struct ProductForm {
    pub csrf_token: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description_md: String,
    #[serde(default)]
    pub description_html_import: String,
    pub category_id: String,
    pub price: String,
    pub price_visible: Option<String>,
    pub preview_url: String,
    pub video_id: String,
    pub techs: String,
    pub thumbnail_url: String,
    pub main_image_url: String,
    pub visibility: String,
    #[serde(default)]
    pub published_at: String,
}

/// Validated product fields: (slug, price, preview_url, video_id, published_at).
type ParsedProduct = (String, Option<i64>, String, String, Option<String>);

fn parse_product_form(input: &ProductForm) -> Result<ParsedProduct, &'static str> {
    let slug = slugify(&input.slug);
    if slug.is_empty() || input.title.trim().is_empty() {
        return Err("empty");
    }
    if !crate::auth::access::valid_visibility(&input.visibility) {
        return Err("visibility");
    }
    let price = if input.price.trim().is_empty() {
        None
    } else {
        match input.price.trim().parse::<i64>() {
            Ok(n) if n >= 0 => Some(n),
            _ => return Err("price"),
        }
    };
    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => return Err("schedule"),
    };
    Ok((
        slug,
        price,
        clean_url(&input.preview_url),
        clean_video_id(&input.video_id),
        published_at,
    ))
}

#[route(POST "/admin/products/create")]
pub async fn create_product(
    cx: &Cx,
    Form(input): Form<ProductForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.products.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let (slug, price, preview_url, video_id, published_at) = match parse_product_form(&input) {
        Ok(v) => v,
        Err(_) => {
            return Ok(topcoat::router::error::see_other(
                "/admin/products/new?error=invalid",
            ));
        }
    };
    let category_id: Option<String> = if input.category_id.trim().is_empty() {
        None
    } else {
        Some(input.category_id.trim().to_string())
    };
    let description_md = if input.description_html_import.trim().is_empty() {
        input.description_md.clone()
    } else {
        crate::services::markdown::html_to_markdown(&input.description_html_import)
    };
    let description_html = render_markdown(&description_md);
    let saved = sqlx::query(
        "INSERT INTO products (category_id, slug, title, summary, description_md, description_html,
         price, price_visible, preview_url, video_id, techs, thumbnail_url, main_image_url,
         visibility, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
    )
    .bind(&category_id)
    .bind(&slug)
    .bind(input.title.trim())
    .bind(input.summary.trim())
    .bind(&description_md)
    .bind(&description_html)
    .bind(price)
    .bind(if input.price_visible.is_some() { 1 } else { 0 })
    .bind(&preview_url)
    .bind(&video_id)
    .bind(input.techs.trim())
    .bind(clean_image_url(&input.thumbnail_url))
    .bind(clean_image_url(&input.main_image_url))
    .bind(&input.visibility)
    .bind(&published_at)
    .execute(db)
    .await;
    if let Err(error) = saved {
        eprintln!("[product create] save failed: {error}");
        return Ok(topcoat::router::error::see_other(
            "/admin/products/new?error=save",
        ));
    }
    Ok(topcoat::router::error::see_other("/admin/products"))
}

#[derive(serde::Deserialize)]
pub struct ProductUpdateForm {
    pub csrf_token: String,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description_md: String,
    #[serde(default)]
    pub description_html_import: String,
    pub category_id: String,
    pub price: String,
    pub price_visible: Option<String>,
    pub preview_url: String,
    pub video_id: String,
    pub techs: String,
    pub thumbnail_url: String,
    pub main_image_url: String,
    pub visibility: String,
    #[serde(default)]
    pub published_at: String,
}

#[route(POST "/admin/products/update")]
pub async fn update_product(
    cx: &Cx,
    Form(input): Form<ProductUpdateForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.products.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let slug = slugify(&input.slug);
    if slug.is_empty() || input.title.trim().is_empty() {
        return Ok(topcoat::router::error::see_other("/admin/products"));
    }
    if !crate::auth::access::valid_visibility(&input.visibility) {
        return Ok(topcoat::router::error::see_other("/admin/products"));
    }
    let price = if input.price.trim().is_empty() {
        None
    } else {
        match input.price.trim().parse::<i64>() {
            Ok(n) if n >= 0 => Some(n),
            _ => return Ok(topcoat::router::error::see_other("/admin/products")),
        }
    };
    let published_at = match crate::util::parse_schedule_kst(&input.published_at) {
        Ok(v) => v,
        Err(()) => return Ok(topcoat::router::error::see_other("/admin/products")),
    };
    let category_id: Option<String> = if input.category_id.trim().is_empty() {
        None
    } else {
        Some(input.category_id.trim().to_string())
    };
    let description_md = if input.description_html_import.trim().is_empty() {
        input.description_md.clone()
    } else {
        crate::services::markdown::html_to_markdown(&input.description_html_import)
    };
    let description_html = render_markdown(&description_md);
    let saved = sqlx::query(
        "UPDATE products SET category_id = $1, slug = $2, title = $3, summary = $4,
         description_md = $5, description_html = $6, price = $7, price_visible = $8,
         preview_url = $9, video_id = $10, techs = $11, thumbnail_url = $12,
         main_image_url = $13, visibility = $14, published_at = $15, updated_at = NOW()::text
         WHERE id = $16",
    )
    .bind(&category_id)
    .bind(&slug)
    .bind(input.title.trim())
    .bind(input.summary.trim())
    .bind(&description_md)
    .bind(&description_html)
    .bind(price)
    .bind(if input.price_visible.is_some() { 1 } else { 0 })
    .bind(clean_url(&input.preview_url))
    .bind(clean_video_id(&input.video_id))
    .bind(input.techs.trim())
    .bind(clean_image_url(&input.thumbnail_url))
    .bind(clean_image_url(&input.main_image_url))
    .bind(&input.visibility)
    .bind(&published_at)
    .bind(&input.id)
    .execute(db)
    .await;
    match saved {
        Ok(result) if result.rows_affected() > 0 => {}
        Ok(_) => return Ok(product_error_redirect("", "missing")),
        Err(error) => {
            eprintln!("[product update] save failed: {error}");
            return Ok(product_error_redirect(&input.id, "save"));
        }
    }
    Ok(topcoat::router::error::see_other(format!(
        "/admin/products/edit?id={}",
        input.id
    )))
}

#[derive(serde::Deserialize)]
pub struct DeleteProductForm {
    pub csrf_token: String,
    pub id: String,
}

#[route(POST "/admin/products/delete")]
pub async fn delete_product(
    cx: &Cx,
    Form(input): Form<DeleteProductForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.products.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM products WHERE id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/products"))
}

/// Image upload (thumbnail / main / gallery). Multipart with CSRF + target.
#[route(POST "/admin/products/upload")]
pub async fn upload_product_image(
    cx: &Cx,
    mut mp: Multipart,
) -> Result<topcoat::router::error::SeeOther> {
    let db: &Db = topcoat::context::app_context(cx);
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let mut csrf = String::new();
    let mut product_id = String::new();
    let mut target = String::new();
    let mut url = String::new();
    let mut filename = String::new();
    let mut ctype = String::new();
    let mut bytes: Vec<u8> = vec![];
    loop {
        let field = match mp.next_field().await {
            Ok(f) => f,
            Err(error) => return Ok(multipart_failure(&product_id, error)),
        };
        let Some(field) = field else { break };
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "csrf_token" | "product_id" | "target" | "url" => {
                let value = match field.text().await {
                    Ok(value) => value,
                    Err(error) => return Ok(multipart_failure(&product_id, error)),
                };
                match field_name.as_str() {
                    "csrf_token" => csrf = value,
                    "product_id" => product_id = value,
                    "target" => target = value,
                    _ => url = value,
                }
            }
            "file" => {
                filename = field.file_name().unwrap_or("").to_string();
                ctype = field.content_type().unwrap_or("").to_string();
                match field.bytes().await {
                    Ok(b) => bytes = b.to_vec(),
                    Err(error) => return Ok(multipart_failure(&product_id, error)),
                }
            }
            _ => {}
        }
    }
    if !crate::auth::session::verify_csrf_logged(db, cx, &csrf, "admin.products.upload").await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    if !matches!(target.as_str(), "thumbnail" | "main" | "gallery") {
        return Ok(product_error_redirect(&product_id, "upload"));
    }
    let exists = sqlx::query_scalar::<_, bool>("SELECT COUNT(*) > 0 FROM products WHERE id = $1")
        .bind(&product_id)
        .fetch_one(db)
        .await
        .map_err(|error| eprintln!("[product upload] product lookup failed: {error}"));
    let exists = match exists {
        Ok(exists) => exists,
        Err(()) => return Ok(product_error_redirect(&product_id, "upload_save")),
    };
    if !exists {
        return Ok(product_error_redirect("", "missing"));
    }
    // Either an uploaded file or an external URL.
    let stored = if !filename.is_empty() || !bytes.is_empty() {
        match crate::services::uploads::store_image(&filename, &ctype, &bytes) {
            Ok(u) => u,
            Err(code) => {
                eprintln!("[product upload] image rejected: {code}");
                return Ok(product_error_redirect(&product_id, code));
            }
        }
    } else {
        let u = clean_image_url(&url);
        if u.is_empty() {
            return Ok(product_error_redirect(&product_id, "upload"));
        }
        u
    };
    let saved = match target.as_str() {
        "thumbnail" => {
            sqlx::query("UPDATE products SET thumbnail_url = $1 WHERE id = $2")
                .bind(&stored)
                .bind(&product_id)
                .execute(db)
                .await
        }
        "main" => {
            sqlx::query("UPDATE products SET main_image_url = $1 WHERE id = $2")
                .bind(&stored)
                .bind(&product_id)
                .execute(db)
                .await
        }
        _ => {
            let max_pos: Result<i64, sqlx::Error> = sqlx::query_scalar(
                "SELECT COALESCE(MAX(position), -1) FROM product_images WHERE product_id = $1",
            )
            .bind(&product_id)
            .fetch_one(db)
            .await;
            let max_pos = match max_pos {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("[product upload] gallery lookup failed: {error}");
                    return Ok(product_error_redirect(&product_id, "upload_save"));
                }
            };
            sqlx::query(
                "INSERT INTO product_images (product_id, url, position) VALUES ($1, $2, $3)",
            )
            .bind(&product_id)
            .bind(&stored)
            .bind(max_pos + 1)
            .execute(db)
            .await
        }
    };
    match saved {
        Ok(result) if result.rows_affected() > 0 => {}
        Ok(_) => return Ok(product_error_redirect("", "missing")),
        Err(error) => {
            eprintln!("[product upload] image save failed: {error}");
            return Ok(product_error_redirect(&product_id, "upload_save"));
        }
    }
    Ok(topcoat::router::error::see_other(format!(
        "/admin/products/edit?id={product_id}"
    )))
}

#[derive(serde::Deserialize)]
pub struct DeleteImageForm {
    pub csrf_token: String,
    pub id: String,
    pub product_id: String,
}

#[route(POST "/admin/products/images/delete")]
pub async fn delete_product_image(
    cx: &Cx,
    Form(input): Form<DeleteImageForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.products.images.delete",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM product_images WHERE id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other(format!(
        "/admin/products/edit?id={}",
        input.product_id
    )))
}

#[derive(serde::Deserialize)]
pub struct CategoryForm {
    pub csrf_token: String,
    pub slug: String,
    pub name: String,
}

#[route(POST "/admin/products/categories/create")]
pub async fn create_product_category(
    cx: &Cx,
    Form(input): Form<CategoryForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.products.categories.create",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let slug = slugify(&input.slug);
    if slug.is_empty() || input.name.trim().is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/products?error=invalid",
        ));
    }
    let max_pos: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(position), -1) FROM product_categories")
            .fetch_one(db)
            .await
            .unwrap_or(-1);
    sqlx::query("INSERT INTO product_categories (slug, name, position) VALUES ($1, $2, $3) ON CONFLICT (slug) DO NOTHING")
        .bind(&slug)
        .bind(input.name.trim())
        .bind(max_pos + 1)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/products"))
}

#[derive(serde::Deserialize)]
pub struct DeleteCategoryForm {
    pub csrf_token: String,
    pub id: String,
}

#[route(POST "/admin/products/categories/delete")]
pub async fn delete_product_category(
    cx: &Cx,
    Form(input): Form<DeleteCategoryForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_products(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.products.categories.delete",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM product_categories WHERE id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/products"))
}

#[page("/admin/products")]
pub async fn admin_products(cx: &Cx) -> Result<impl View + '_> {
    if require_products(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let error_message = product_error_message(query_param(cx, "error").as_deref().unwrap_or(""));
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY created_at DESC")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    let categories = sqlx::query_as::<_, ProductCategory>(
        "SELECT * FROM product_categories ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    let cat_map: std::collections::HashMap<String, String> = categories
        .iter()
        .map(|c| (c.id.clone(), c.name.clone()))
        .collect();
    let new_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inquiries WHERE status = 'new'")
        .fetch_one(db)
        .await
        .unwrap_or(0);

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"상품 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/products".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"상품 관리"</h1>
                        <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"바이브코딩 쇼케이스 상품"</p>
                    </div>
                    <div style="display:flex; gap:8px;">
                        <a href="/admin/inquiries" style="background:var(--card); color:var(--text2); padding:10px 16px; border-radius:8px; font-size:13px; font-weight:600; border:1px solid var(--border);">(format!("문의 관리 ({})", new_count))</a>
                        <a href="/admin/products/new" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600;">"+ 새 상품"</a>
                    </div>
                </div>

                if !error_message.is_empty() {
                    <p role="alert" style="color:#dc2626; margin-bottom:16px;">(error_message)</p>
                }
                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden; margin-bottom:20px;">
                    <table style="width:100%; font-size:13px; border-collapse:collapse;">
                        <thead>
                            <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"상품"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"가격"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"공개"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"관리"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for p in &products {
                                <tr style="border-bottom:1px solid var(--line);">
                                    <td style="padding:14px 20px;">
                                        <a href=(format!("/admin/products/edit?id={}", p.id)) style="font-weight:600; color:var(--text3);">(p.title.clone())</a>
                                        <div style="font-size:11px; color:var(--dim); font-family:monospace; margin-top:2px;">(p.slug.clone())</div>
                                        if !crate::util::is_released(p.published_at.as_deref()) {
                                            <span style="font-size:11px; font-weight:600; color:#b45309;">" 예약됨"</span>
                                        }
                                    </td>
                                    <td style="padding:14px 20px; color:var(--muted);">
                                        if let Some(price) = p.price_label() {
                                            (price)
                                        } else {
                                            <span style="color:var(--dim);">"미표시"</span>
                                        }
                                    </td>
                                    <td style="padding:14px 20px;">
                                        <span class="badge" style="background:rgba(96,165,250,0.1); color:var(--blue);">(crate::auth::access::visibility_label(&p.visibility))</span>
                                        <div style="font-size:11px; color:var(--dim); margin-top:2px;">(cat_map.get(&p.category_id.clone().unwrap_or_default()).cloned().unwrap_or_default())</div>
                                    </td>
                                    <td style="padding:14px 20px; text-align:right;">
                                        <div style="display:flex; gap:8px; justify-content:flex-end; align-items:center;">
                                            <a href=(format!("/vibecoding/{}", p.slug)) style="font-size:12px; color:var(--muted2);">"보기"</a>
                                            <a href=(format!("/admin/products/edit?id={}", p.id)) style="font-size:12px; color:var(--amber);">"수정"</a>
                                            <form action="/admin/products/delete" method="POST" onsubmit="return confirm('이 상품을 삭제하시겠습니까?')" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="id" value=(p.id.clone())>
                                                <button type="submit" style="background:none; border:none; padding:0; font-size:12px; color:var(--red); cursor:pointer; font-family:inherit; font-weight:600;">"삭제"</button>
                                            </form>
                                        </div>
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; max-width:720px;">
                    <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:16px;">"카테고리"</h2>
                    <div style="display:flex; gap:8px; flex-wrap:wrap; margin-bottom:16px;">
                        for c in &categories {
                            <span style="display:inline-flex; align-items:center; gap:8px; font-size:12px; color:var(--text3); background:var(--bg); border:1px solid var(--border); border-radius:9999px; padding:6px 8px 6px 14px;">
                                (c.name.clone())
                                <form action="/admin/products/categories/delete" method="POST" onsubmit="return confirm('이 카테고리를 삭제하시겠습니까?')" style="display:inline;">
                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                    <input type="hidden" name="id" value=(c.id.clone())>
                                    <button type="submit" style="background:none; border:none; color:var(--red); cursor:pointer; font-size:12px; padding:0 4px;">"×"</button>
                                </form>
                            </span>
                        }
                    </div>
                    <form action="/admin/products/categories/create" method="POST" style="display:flex; gap:8px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="text" name="slug" required="true" placeholder="slug (영문)" style="flex:1; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:13px; color:var(--text2); outline:none; font-family:monospace;">
                        <input type="text" name="name" required="true" placeholder="이름 (예: 웹앱)" style="flex:1; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:13px; color:var(--text2); outline:none;">
                        <button type="submit" style="background:var(--card); color:var(--text2); padding:10px 20px; border-radius:8px; font-size:13px; font-weight:600; border:1px solid var(--border); cursor:pointer; white-space:nowrap;">"추가"</button>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[page("/admin/products/new")]
pub async fn new_product(cx: &Cx) -> Result<impl View + '_> {
    if require_products(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let error_message = product_error_message(query_param(cx, "error").as_deref().unwrap_or(""));
    let categories = sqlx::query_as::<_, ProductCategory>(
        "SELECT * FROM product_categories ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"새 상품 - Rust Korea Admin"</title>
            <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL)>
            <script src=(crate::admin_ui::TOAST_JS_URL)></script>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
            <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/products".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <a href="/admin/products" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">"← 상품 목록"</a>
                    <h1 style="font-size:1.25rem; font-weight:700;">"새 상품"</h1>
                </div>
                if !error_message.is_empty() {
                    <p role="alert" style="color:#dc2626; margin-bottom:16px;">(error_message)</p>
                }
                <form action="/admin/products/create" method="POST" style="max-width:760px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:16px; display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"슬러그 (URL)"</label>
                            <input type="text" name="slug" required="true" placeholder="my-app" class="input" style="font-family:monospace;">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"상품명"</label>
                            <input type="text" name="title" required="true" class="input">
                        </div>
                        <div style="grid-column:1 / -1;">
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"요약"</label>
                            <input type="text" name="summary" class="input" placeholder="목록에 보이는 한 줄 소개">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"카테고리"</label>
                            <select name="category_id" class="input">
                                <option value="">"미지정"</option>
                                for c in &categories {
                                    <option value=(c.id.clone())>(c.name.clone())</option>
                                }
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"기술 스택 (쉼표 구분)"</label>
                            <input type="text" name="techs" class="input" placeholder="Rust, Topcoat, PostgreSQL">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"가격 (원, 비워두면 미정)"</label>
                            <input type="number" name="price" min="0" class="input" placeholder="500000">
                        </div>
                        <div style="display:flex; align-items:flex-end; padding-bottom:12px;">
                            <label style="display:flex; align-items:center; gap:8px; font-size:13px; color:var(--muted); cursor:pointer;">
                                <input type="checkbox" name="price_visible" value="1" checked="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                "가격 표시"
                            </label>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"미리보기 URL"</label>
                            <input type="text" name="preview_url" class="input" placeholder="https://...">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"소개 영상 ID (YouTube)"</label>
                            <input type="text" name="video_id" class="input" placeholder="11자리 영상 ID" style="font-family:monospace;">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"썸네일 URL"</label>
                            <input type="text" name="thumbnail_url" class="input" placeholder="https://... (또는 아래 업로드)">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"메인 이미지 URL"</label>
                            <input type="text" name="main_image_url" class="input" placeholder="https://... (또는 아래 업로드)">
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"공개 범위"</label>
                            <select name="visibility" class="input">
                                <option value="public">"전체공개 (비회원 포함)"</option>
                                <option value="members">"회원전용 (로그인 필요)"</option>
                                <option value="staff">"스태프전용"</option>
                                <option value="premium">"유료전용(준비중)"</option>
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"예약 발행 (비워두면 즉시)"</label>
                            <input type="datetime-local" name="published_at" class="input">
                        </div>
                        <div style="grid-column:1 / -1;">
                            <div style="display:flex; gap:8px; margin-bottom:12px;">
                                <button type="button" id="tabbtn-md" onclick="showEditorTab('md')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--brand); background:rgba(222,91,48,0.12); color:var(--text); cursor:pointer;">"마크다운"</button>
                                <button type="button" id="tabbtn-html" onclick="showEditorTab('html')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--border); background:transparent; color:var(--muted); cursor:pointer;">"HTML 가져오기"</button>
                            </div>
                            <div id="tab-md">
                            <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"상세 설명 (Markdown)"</label>
                            <textarea id="editor" name="description_md" rows="16" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;"></textarea>
                            </div>
                            <div id="tab-html" style="display:none;">
                                <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"HTML 소스 붙여넣기"</label>
                                <textarea name="description_html_import" rows="16" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;" placeholder="여기에 HTML 소스를 붙여넣으세요"></textarea>
                            </div>
                        </div>
                    </div>
                    <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; width:100%; border:none; cursor:pointer;">"상품 저장"</button>
                </form>
                <script>
                    {(crate::services::markdown::EDITOR_TAB_JS)}
                    {(crate::services::markdown::EASYMDE_INIT_JS)}
                    "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}"
                </script>
            </div>
        </body>
        </html>
    })
}

#[page("/admin/products/edit")]
pub async fn edit_product(cx: &Cx) -> Result<impl View + '_> {
    if require_products(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let id = query_param(cx, "id").unwrap_or_default();
    if id.is_empty() {
        return Err(topcoat::router::error::redirect("/admin/products").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let error_message = product_error_message(query_param(cx, "error").as_deref().unwrap_or(""));
    let product: Option<Product> =
        sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(&id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
    let categories = sqlx::query_as::<_, ProductCategory>(
        "SELECT * FROM product_categories ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    match product {
        Some(p) => {
            let images = sqlx::query_as::<_, ProductImage>(
                "SELECT * FROM product_images WHERE product_id = $1 ORDER BY position ASC",
            )
            .bind(&p.id)
            .fetch_all(db)
            .await
            .unwrap_or_default();
            let price_str = p.price.map(|v| v.to_string()).unwrap_or_default();
            Ok(view! {
                <html lang="ko">
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>"상품 수정 - Rust Korea Admin"</title>
                    <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL)>
                    <script src=(crate::admin_ui::TOAST_JS_URL)></script>
                    <style>{(ADMIN_STYLE)}</style>
                    <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
                    <style>{(crate::services::markdown::MARKDOWN_CSS)}</style>
                </head>
                <body style="display:flex; min-height:100vh;">
                    {AdminSidebar(active: "/admin/products".into())}
                    <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                        <div style="margin-bottom:24px;">
                            <a href="/admin/products" style="font-size:12px; color:var(--muted2); display:inline-flex; align-items:center; gap:6px; margin-bottom:12px;">"← 상품 목록"</a>
                            <h1 style="font-size:1.25rem; font-weight:700;">"상품 수정"</h1>
                        </div>
                        if !error_message.is_empty() {
                            <p role="alert" style="color:#dc2626; margin-bottom:16px;">(error_message)</p>
                        }
                        <form action="/admin/products/update" method="POST" style="max-width:760px;">
                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                            <input type="hidden" name="id" value=(p.id.clone())>
                            <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:16px; display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"슬러그 (URL)"</label>
                                    <input type="text" name="slug" required="true" class="input" style="font-family:monospace;" value=(p.slug.clone())>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"상품명"</label>
                                    <input type="text" name="title" required="true" class="input" value=(p.title.clone())>
                                </div>
                                <div style="grid-column:1 / -1;">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"요약"</label>
                                    <input type="text" name="summary" class="input" value=(p.summary.clone())>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"카테고리"</label>
                                    <select name="category_id" class="input">
                                        <option value="">"미지정"</option>
                                        for c in &categories {
                                            if Some(c.id.clone()) == p.category_id {
                                                <option value=(c.id.clone()) selected="true">(c.name.clone())</option>
                                            } else {
                                                <option value=(c.id.clone())>(c.name.clone())</option>
                                            }
                                        }
                                    </select>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"기술 스택 (쉼표 구분)"</label>
                                    <input type="text" name="techs" class="input" value=(p.techs.clone())>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"가격 (원, 비워두면 미정)"</label>
                                    <input type="number" name="price" min="0" class="input" value=(price_str)>
                                </div>
                                <div style="display:flex; align-items:flex-end; padding-bottom:12px;">
                                    <label style="display:flex; align-items:center; gap:8px; font-size:13px; color:var(--muted); cursor:pointer;">
                                        if p.price_visible == 1 {
                                            <input type="checkbox" name="price_visible" value="1" checked="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                        } else {
                                            <input type="checkbox" name="price_visible" value="1" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                        }
                                        "가격 표시"
                                    </label>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"미리보기 URL"</label>
                                    <input type="text" name="preview_url" class="input" value=(p.preview_url.clone())>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"소개 영상 ID (YouTube)"</label>
                                    <input type="text" name="video_id" class="input" style="font-family:monospace;" value=(p.video_id.clone())>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"썸네일 URL"</label>
                                    <input type="text" name="thumbnail_url" class="input" value=(p.thumbnail_url.clone())>
                                    if !p.thumbnail_url.is_empty() {
                                        <img src=(p.thumbnail_url.clone()) alt="현재 썸네일" onclick="openShot(this.src)" style="display:block; max-width:100%; height:120px; object-fit:contain; margin-top:8px; cursor:zoom-in;">
                                    }
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"메인 이미지 URL"</label>
                                    <input type="text" name="main_image_url" class="input" value=(p.main_image_url.clone())>
                                    if !p.main_image_url.is_empty() {
                                        <img src=(p.main_image_url.clone()) alt="현재 메인 이미지" onclick="openShot(this.src)" style="display:block; max-width:100%; height:120px; object-fit:contain; margin-top:8px; cursor:zoom-in;">
                                    }
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"공개 범위"</label>
                                    <select name="visibility" class="input">
                                        <option value="public" selected=(p.visibility == "public")>"전체공개 (비회원 포함)"</option>
                                        <option value="members" selected=(p.visibility == "members")>"회원전용 (로그인 필요)"</option>
                                        <option value="staff" selected=(p.visibility == "staff")>"스태프전용"</option>
                                        <option value="premium" selected=(p.visibility == "premium")>"유료전용(준비중)"</option>
                                    </select>
                                </div>
                                <div>
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"예약 발행 (비우면 즉시)"</label>
                                    <input type="datetime-local" name="published_at" class="input" value=(crate::util::format_schedule_kst(p.published_at.as_deref()))>
                                </div>
                                <div style="grid-column:1 / -1;">
                                    <div style="display:flex; gap:8px; margin-bottom:12px;">
                                        <button type="button" id="tabbtn-md" onclick="showEditorTab('md')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--brand); background:rgba(222,91,48,0.12); color:var(--text); cursor:pointer;">"마크다운"</button>
                                        <button type="button" id="tabbtn-html" onclick="showEditorTab('html')" style="font-size:12px; font-weight:600; padding:6px 14px; border-radius:6px; border:1px solid var(--border); background:transparent; color:var(--muted); cursor:pointer;">"HTML 가져오기"</button>
                                    </div>
                                    <div id="tab-md">
                                    <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"상세 설명 (Markdown)"</label>
                                    <textarea id="editor" name="description_md" rows="16" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;">(p.description_md.clone())</textarea>
                                    </div>
                                    <div id="tab-html" style="display:none;">
                                        <label style="display:block; font-size:13px; font-weight:600; color:var(--muted); margin-bottom:8px;">"HTML 소스 붙여넣기 (비워두면 기존 유지)"</label>
                                        <textarea name="description_html_import" rows="16" class="input" style="font-family:monospace; font-size:13px; line-height:1.7;" placeholder="여기에 HTML 소스를 붙여넣으세요"></textarea>
                                    </div>
                                </div>
                            </div>
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 20px; border-radius:8px; font-weight:600; font-size:14px; width:100%; border:none; cursor:pointer;">"변경 사항 저장"</button>
                        </form>

                        <div style="max-width:760px; background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-top:16px;">
                            <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"이미지 업로드 (5MiB 이하)"</h2>
                            <p style="font-size:12px; color:var(--muted2); margin-bottom:16px;">"PNG/JPG/WebP. 썸네일·메인은 기존 URL을 덮어씁니다."</p>
                            <div style="display:grid; grid-template-columns:1fr 1fr 1fr; gap:12px;">
                                for target in ["thumbnail", "main", "gallery"] {
                                    <form action="/admin/products/upload" method="POST" enctype="multipart/form-data" style="background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:16px;">
                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                        <input type="hidden" name="product_id" value=(p.id.clone())>
                                        <input type="hidden" name="target" value=(target.to_string())>
                                        <div style="font-size:13px; font-weight:600; color:var(--text2); margin-bottom:8px;">(match target { "thumbnail" => "썸네일", "main" => "메인", _ => "갤러리 추가" })</div>
                                        <input type="file" name="file" accept=".png,.jpg,.jpeg,.webp" style="font-size:12px; color:var(--muted); margin-bottom:8px; width:100%;">
                                        <input type="text" name="url" placeholder="또는 이미지 URL" style="width:100%; background:var(--card); border:1px solid var(--border); border-radius:6px; padding:8px 10px; font-size:12px; color:var(--text2); outline:none; margin-bottom:8px;">
                                        <button type="submit" style="background:var(--card); color:var(--text2); padding:8px 16px; border-radius:6px; font-size:12px; font-weight:600; border:1px solid var(--border); cursor:pointer; width:100%;">"업로드"</button>
                                    </form>
                                }
                            </div>
                            if !images.is_empty() {
                                <div style="display:flex; gap:8px; flex-wrap:wrap; margin-top:16px;">
                                    for img in &images {
                                        <div style="position:relative;">
                                            <img src=(img.url.clone()) alt="" onclick="openShot(this.src)" style="width:120px; height:80px; object-fit:cover; border-radius:6px; border:1px solid var(--border); display:block; cursor:zoom-in;" title="클릭하면 크게 보기">
                                            <form action="/admin/products/images/delete" method="POST" onsubmit="return confirm('이 이미지를 삭제하시겠습니까?')" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="id" value=(img.id.clone())>
                                                <input type="hidden" name="product_id" value=(p.id.clone())>
                                                <button type="submit" style="background:rgba(0,0,0,0.6); color:#fff; border:none; border-radius:50%; width:22px; height:22px; font-size:12px; cursor:pointer; position:absolute; top:4px; right:4px;">"×"</button>
                                            </form>
                                        </div>
                                    }
                                </div>
                            }
                        </div>
                        <script>
                            {(crate::services::markdown::EDITOR_TAB_JS)}
                            {(crate::services::markdown::EASYMDE_INIT_JS)}
                            "var easymde=null;try{easymde=initEasyMDE('editor');}catch(e){}"
                        </script>
                        <div id="shot-lightbox" onclick="closeShot()" style="display:none; position:fixed; top:0; left:0; right:0; bottom:0; background:rgba(0,0,0,0.85); z-index:100; align-items:center; justify-content:center; padding:24px;">
                            <img id="shot-lightbox-img" src="" alt="확대 보기" style="max-width:100%; max-height:100%; border-radius:12px;">
                        </div>
                        <script>
                            {r#"function openShot(u){var l=document.getElementById('shot-lightbox');var i=document.getElementById('shot-lightbox-img');if(l){if(i){i.setAttribute('src',u);l.style.display='flex';}}}function closeShot(){var l=document.getElementById('shot-lightbox');if(l){l.style.display='none';}}"#}
                        </script>
                    </div>
                </body>
                </html>
            })
        }
        _ => Err(topcoat::router::error::redirect("/admin/products").into()),
    }
}
