use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{path_param, route},
};

use crate::db::Db;
use crate::models::Post;

use super::helpers::{can_edit_post, get_current_user};

topcoat::router::path_param!(id);

#[route(POST "/community/{id}/delete")]
pub async fn delete_post(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    if get_current_user(cx).await.is_none() {
        return Ok(topcoat::router::error::see_other(format!(
            "/auth/login?redirect=/community/{id}"
        )));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "community.post.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let post: Option<Post> = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let Some(post) = post else {
        return Ok(topcoat::router::error::see_other("/community"));
    };

    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;
    if !can_edit_post(&access, &post.author_id) {
        return Ok(topcoat::router::error::see_other(format!(
            "/community/{}",
            post.id
        )));
    }

    sqlx::query("DELETE FROM comments WHERE post_id = $1")
        .bind(&post.id)
        .execute(db)
        .await
        .ok();

    sqlx::query("DELETE FROM post_likes WHERE post_id = $1")
        .bind(&post.id)
        .execute(db)
        .await
        .ok();

    sqlx::query("DELETE FROM bookmarks WHERE post_id = $1")
        .bind(&post.id)
        .execute(db)
        .await
        .ok();

    sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(&post.id)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other("/community"))
}
