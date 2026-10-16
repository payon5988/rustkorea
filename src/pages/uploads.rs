use topcoat::{
    Result,
    context::Cx,
    router::{content::multipart::Multipart, route},
};

use crate::db::Db;

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
    if user.is_none() || !csrf_ok {
        return Ok(crate::services::TypedBody {
            content_type: "application/json",
            body: "{\"error\":\"forbidden\"}".to_string(),
        });
    }
    match crate::services::uploads::store_image(&filename, &ctype, &bytes) {
        Ok(url) => Ok(crate::services::TypedBody {
            content_type: "application/json",
            body: format!("{{\"url\":\"{url}\"}}"),
        }),
        Err(code) => Ok(crate::services::TypedBody {
            content_type: "application/json",
            body: format!("{{\"error\":\"{code}\"}}"),
        }),
    }
}
