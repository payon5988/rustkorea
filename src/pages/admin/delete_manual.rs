use topcoat::{Result, context::Cx, router::content::Form, router::route};

use crate::db::Db;

#[derive(serde::Deserialize)]
pub struct DeleteManual {
    pub csrf_token: String,
    pub id: String,
}

#[route(POST "/admin/manuals/delete")]
pub async fn delete_manual(
    cx: &Cx,
    Form(input): Form<DeleteManual>,
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

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.manuals.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    sqlx::query("DELETE FROM manual_chapters WHERE series_id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();

    sqlx::query("DELETE FROM manual_series WHERE id = $1")
        .bind(&input.id)
        .execute(db)
        .await
        .ok();

    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.manual.delete",
        "manual_series",
        "",
        "",
    )
    .await;

    Ok(topcoat::router::error::see_other("/admin/manuals"))
}
