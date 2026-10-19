use topcoat::{
    Result,
    context::Cx,
    router::{content::multipart::Multipart, route},
};

use crate::db::Db;

/// Editor uploads per member per window (admin-console users are exempt).
pub const EDITOR_UPLOADS_PER_WINDOW: i64 = 30;
pub const EDITOR_UPLOAD_WINDOW_SECS: f64 = 3600.0;

/// Shared editor image upload (any authenticated user). Multipart with CSRF.
/// Returns `{"url": "..."}` for the Toast `addImageBlobHook`, or
/// `{"error": "<code>"}` on failure.
#[route(POST "/admin/uploads")]
pub async fn upload_editor_image(cx: &Cx, mut mp: Multipart) -> Result<crate::services::TypedBody> {
    let db: &Db = topcoat::context::app_context(cx);
    let mut csrf = String::new();
    let mut filename = String::new();
    let mut ctype = String::new();
    let mut bytes: Vec<u8> = vec![];
    loop {
        let field = match mp.next_field().await {
            Ok(f) => f,
            Err(_) => break,
        };
        let Some(field) = field else { break };
        match field.name().unwrap_or("") {
            "csrf_token" => csrf = field.text().await.unwrap_or_default(),
            "file" => {
                filename = field.file_name().unwrap_or("").to_string();
                ctype = field.content_type().unwrap_or("").to_string();
                bytes = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }
    // Accept either a logged-in session token or an anonymous double-submit seed.
    let user = crate::auth::session::current_user(db, cx).await;
    let csrf_ok = crate::auth::session::verify_csrf(db, cx, &csrf).await
        || crate::auth::session::verify_csrf_seed(cx, &csrf);
    let Some(user) = user.filter(|_| csrf_ok) else {
        return Ok(crate::services::TypedBody {
            content_type: "application/json",
            body: "{\"error\":\"forbidden\"}".to_string(),
        });
    };
    // Members are capped so one account cannot fill the upload volume.
    let staff = crate::auth::access::load_access_for_user(db, &user)
        .await
        .is_some_and(|a| crate::auth::access::can_access_admin(&a));
    if !staff {
        let recent: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM uploads WHERE user_id = $1
             AND created_at::timestamptz > NOW() - make_interval(secs => $2)",
        )
        .bind(&user.id)
        .bind(EDITOR_UPLOAD_WINDOW_SECS)
        .fetch_one(db)
        .await
        .unwrap_or(0);
        if recent >= EDITOR_UPLOADS_PER_WINDOW {
            return Ok(crate::services::TypedBody {
                content_type: "application/json",
                body: "{\"error\":\"rate\"}".to_string(),
            });
        }
    }
    match crate::services::uploads::store_image(&filename, &ctype, &bytes) {
        Ok(url) => {
            sqlx::query(
                "INSERT INTO uploads (user_id, filename, original_name, mime_type, size)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&user.id)
            .bind(url.trim_start_matches("/uploads/"))
            .bind(filename.chars().take(200).collect::<String>())
            .bind(&ctype)
            .bind(bytes.len() as i32)
            .execute(db)
            .await
            .map_err(|e| eprintln!("[upload] record failed: {e}"))
            .ok();
            Ok(crate::services::TypedBody {
                content_type: "application/json",
                body: format!("{{\"url\":\"{url}\"}}"),
            })
        }
        Err(code) => Ok(crate::services::TypedBody {
            content_type: "application/json",
            body: format!("{{\"error\":\"{code}\"}}"),
        }),
    }
}
