use topcoat::{Result, context::Cx, router::content::Form, router::route};

use crate::db::Db;

#[derive(serde::Deserialize)]
pub struct DeleteChapter {
    pub csrf_token: String,
    pub id: String,
    pub series_slug: String,
}

#[route(POST "/admin/manuals/chapters/delete")]
pub async fn delete_chapter(
    cx: &Cx,
    Form(input): Form<DeleteChapter>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::MANUALS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(
        db,
        cx,
        &input.csrf_token,
        "admin.manuals.chapters.delete",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    sqlx::query("DELETE FROM manual_chapters WHERE id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.chapter.delete",
        "manual_chapters",
        &input.id,
        "",
    )
    .await;

    Ok(topcoat::router::error::see_other(format!(
        "/admin/manuals/{}",
        input.series_slug
    )))
}
