use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::Category;

#[derive(serde::Deserialize)]
pub struct CategoryForm {
    pub csrf_token: String,
    pub slug: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub write_permission: Option<String>,
}

#[route(POST "/admin/categories/create")]
pub async fn create_category(
    cx: &Cx,
    Form(input): Form<CategoryForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
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
        "admin.categories.create",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let slug = input.slug.trim().to_lowercase().replace(' ', "-");
    let max_pos: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(position), 0) FROM categories")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let write_perm = input
        .write_permission
        .unwrap_or_else(|| "everyone".to_string());

    sqlx::query(
        "INSERT INTO categories (id, slug, name, color, description, position, visible, write_permission, created_at) VALUES ($1, $2, $3, $4, $5, $6, 1, $7, NOW()::text)"
    )
    .bind(&slug).bind(&slug).bind(&input.name).bind(&input.color).bind(&input.description).bind(max_pos + 1).bind(&write_perm)
    .execute(db).await.ok();

    Ok(topcoat::router::error::see_other("/admin/categories"))
}

#[derive(serde::Deserialize)]
pub struct CategoryUpdateForm {
    pub csrf_token: String,
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub write_permission: Option<String>,
}

#[route(POST "/admin/categories/update")]
pub async fn update_category(
    cx: &Cx,
    Form(input): Form<CategoryUpdateForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
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
        "admin.categories.update",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let write_perm = input
        .write_permission
        .unwrap_or_else(|| "everyone".to_string());
    sqlx::query("UPDATE categories SET name = $1, color = $2, description = $3, write_permission = $4 WHERE id = $5")
        .bind(&input.name).bind(&input.color).bind(&input.description).bind(&write_perm).bind(&input.id)
        .execute(db).await.ok();
    Ok(topcoat::router::error::see_other("/admin/categories"))
}

#[route(POST "/admin/categories/{cat_id}/delete")]
pub async fn delete_category(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
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
        "admin.categories.x.delete",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let cat_id: String = {
        let uri = topcoat::router::request::uri(cx);
        let path = uri.path();
        path.strip_prefix("/admin/categories/")
            .unwrap_or("")
            .trim_end_matches("/delete")
            .to_string()
    };
    sqlx::query("DELETE FROM categories WHERE id = $1 AND slug != 'general'")
        .bind(&cat_id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/categories"))
}

#[route(POST "/admin/categories/{cat_id}/toggle")]
pub async fn toggle_category(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
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
        "admin.categories.x.toggle",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let cat_id: String = {
        let uri = topcoat::router::request::uri(cx);
        let path = uri.path();
        path.strip_prefix("/admin/categories/")
            .unwrap_or("")
            .trim_end_matches("/toggle")
            .to_string()
    };
    sqlx::query(
        "UPDATE categories SET visible = CASE WHEN visible = 1 THEN 0 ELSE 1 END WHERE id = $1",
    )
    .bind(&cat_id)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other("/admin/categories"))
}

#[route(POST "/admin/categories/{cat_id}/move_up")]
pub async fn move_category_up(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
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
        "admin.categories.x.move_up",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let cat_id: String = {
        let uri = topcoat::router::request::uri(cx);
        let path = uri.path();
        path.strip_prefix("/admin/categories/")
            .unwrap_or("")
            .trim_end_matches("/move_up")
            .to_string()
    };
    let cur_pos: Option<i64> = sqlx::query_scalar("SELECT position FROM categories WHERE id = $1")
        .bind(&cat_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    if let Some(pos) = cur_pos {
        let above_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM categories WHERE position = $1")
                .bind(pos - 1)
                .fetch_optional(db)
                .await
                .ok()
                .flatten();
        if let Some(above) = above_id {
            sqlx::query("UPDATE categories SET position = $1 WHERE id = $2")
                .bind(pos - 1)
                .bind(&cat_id)
                .execute(db)
                .await
                .ok();
            sqlx::query("UPDATE categories SET position = $1 WHERE id = $2")
                .bind(pos)
                .bind(&above)
                .execute(db)
                .await
                .ok();
        }
    }
    Ok(topcoat::router::error::see_other("/admin/categories"))
}

#[route(POST "/admin/categories/{cat_id}/move_down")]
pub async fn move_category_down(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
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
        "admin.categories.x.move_down",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let cat_id: String = {
        let uri = topcoat::router::request::uri(cx);
        let path = uri.path();
        path.strip_prefix("/admin/categories/")
            .unwrap_or("")
            .trim_end_matches("/move_down")
            .to_string()
    };
    let cur_pos: Option<i64> = sqlx::query_scalar("SELECT position FROM categories WHERE id = $1")
        .bind(&cat_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    if let Some(pos) = cur_pos {
        let below_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM categories WHERE position = $1")
                .bind(pos + 1)
                .fetch_optional(db)
                .await
                .ok()
                .flatten();
        if let Some(below) = below_id {
            sqlx::query("UPDATE categories SET position = $1 WHERE id = $2")
                .bind(pos + 1)
                .bind(&cat_id)
                .execute(db)
                .await
                .ok();
            sqlx::query("UPDATE categories SET position = $1 WHERE id = $2")
                .bind(pos)
                .bind(&below)
                .execute(db)
                .await
                .ok();
        }
    }
    Ok(topcoat::router::error::see_other("/admin/categories"))
}

#[page("/admin/categories")]
pub async fn admin_categories(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "community.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let categories =
        sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY position ASC")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    let post_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT category, COUNT(*) as cnt FROM posts GROUP BY category")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    let count_map: std::collections::HashMap<String, i64> = post_counts.into_iter().collect();

    let input_style = "width:100%; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;";

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"카테고리 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/categories".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">

                <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:24px;">
                    <div>
                        <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"카테고리 관리"</h1>
                        <p style="font-size:13px; color:#6b7280; margin-top:2px;">(format!("{}개 카테고리", categories.len()))</p>
                    </div>
                </div>

                <div style="display:grid; grid-template-columns:1fr 320px; gap:24px;">
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden;">
                        <table style="width:100%; font-size:13px; border-collapse:collapse;">
                            <thead>
                                <tr style="border-bottom:1px solid #2a2d3a; text-align:left;">
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"색상"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"이름"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"슬러그"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"쓰기 권한"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em;">"게시글"</th>
                                    <th style="padding:12px 20px; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"관리"</th>
                                </tr>
                            </thead>
                            <tbody>
                                for cat in &categories {
                                    let cnt = count_map.get(&cat.slug).unwrap_or(&0);
                                    let opacity = if cat.visible == 0 { "0.5" } else { "1" };
                                    let toggle_color = if cat.visible == 1 { "#fbbf24" } else { "#6b7280" };
                                    let toggle_label = if cat.visible == 1 { "숨기기" } else { "표시" };
                                    <tr style=(format!("border-bottom:1px solid rgba(255,255,255,0.03); opacity:{};", opacity))>
                                        <td style="padding:14px 20px;">
                                            <div style="width:12px; height:12px; border-radius:50%; background:(cat.color.clone());"></div>
                                        </td>
                                        <td style="padding:14px 20px; font-weight:600; color:#d1d5db;">(cat.name.clone())</td>
                                        <td style="padding:14px 20px; color:#9ca3af; font-family:monospace; font-size:12px;">(cat.slug.clone())</td>
                                        <td style="padding:14px 20px;">
                                            <span style=(format!("font-size:11px; font-weight:600; padding:3px 8px; border-radius:9999px; color:{}; background:{}1a;",
                                                match cat.write_permission.as_str() { "admin" => "#DC2626", "editor" => "#F59E0B", _ => "#10B981" },
                                                match cat.write_permission.as_str() { "admin" => "#DC2626", "editor" => "#F59E0B", _ => "#10B981" }
                                            ))>
                                                (match cat.write_permission.as_str() { "admin" => "관리자만", "editor" => "운영진", _ => "전체" })
                                            </span>
                                        </td>
                                        <td style="padding:14px 20px; color:#9ca3af;">(cnt)</td>
                                        <td style="padding:14px 20px; text-align:right;">
                                            <div style="display:flex; gap:8px; justify-content:flex-end; align-items:center;">
                                                <form action=(format!("/admin/categories/{}/move_up", cat.id)) method="POST" style="display:inline;">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <button type="submit" style="font-size:11px; color:#6b7280; background:none; border:none; cursor:pointer; padding:2px 4px;" title="위로">"▲"</button>
                                                </form>
                                                <form action=(format!("/admin/categories/{}/move_down", cat.id)) method="POST" style="display:inline;">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <button type="submit" style="font-size:11px; color:#6b7280; background:none; border:none; cursor:pointer; padding:2px 4px;" title="아래로">"▼"</button>
                                                </form>
                                                <button type="button" onclick="fillCatEdit(this)" data-id=(cat.id.clone()) data-name=(cat.name.clone()) data-color=(cat.color.clone()) data-desc=(cat.description.clone()) data-perm=(cat.write_permission.clone()) style="font-size:12px; color:#60a5fa; background:none; border:none; cursor:pointer; font-weight:600;">"수정"</button>
                                                <form action=(format!("/admin/categories/{}/toggle", cat.id)) method="POST" style="display:inline;">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <button type="submit" style=(format!("font-size:12px; color:{}; background:none; border:none; cursor:pointer; font-weight:600;", toggle_color))>
                                                        (toggle_label)
                                                    </button>
                                                </form>
                                                if cat.slug != "general" {
                                                    <form action=(format!("/admin/categories/{}/delete", cat.id)) method="POST" style="display:inline;" onsubmit="return confirm('정말 삭제하시겠습니까?')">
                                                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                        <button type="submit" style="font-size:12px; color:#f87171; background:none; border:none; cursor:pointer; font-weight:600;">"삭제"</button>
                                                    </form>
                                                }
                                            </div>
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>

                    <div>
                        <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px;">
                            <h2 style="font-size:14px; font-weight:700; color:#e5e7eb; margin-bottom:16px;">"새 카테고리 추가"</h2>
                            <form action="/admin/categories/create" method="POST" style="display:flex; flex-direction:column; gap:14px;">
                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                <div>
                                    <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"슬러그 (영문, 고유값)"</label>
                                    <input type="text" name="slug" required="true" placeholder="예: rust-tips" style=(input_style)>
                                </div>
                                <div>
                                    <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"이름"</label>
                                    <input type="text" name="name" required="true" placeholder="예: Rust 팁" style=(input_style)>
                                </div>
                                <div>
                                    <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"색상 (HEX)"</label>
                                    <div style="display:flex; gap:8px; align-items:center;">
                                        <input type="color" id="color-picker" value="#6b7280" style="width:40px; height:36px; border:none; cursor:pointer; background:transparent;" oninput="document.getElementById('color-hex').value=this.value">
                                        <input type="text" name="color" id="color-hex" value="#6b7280" style="width:100px; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:8px 10px; font-size:13px; color:#e5e7eb; outline:none; font-family:monospace;" oninput="document.getElementById('color-picker').value=this.value">
                                    </div>
                                </div>
                                <div>
                                    <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"설명"</label>
                                    <input type="text" name="description" placeholder="카테고리 설명" style=(input_style)>
                                </div>
                                <div>
                                    <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"쓰기 권한"</label>
                                    <select name="write_permission" style=(input_style)>
                                        <option value="everyone">"전체 (로그인 사용자)"</option>
                                        <option value="editor">"운영진 (관리자 + 에디터)"</option>
                                        <option value="admin">"관리자만"</option>
                                    </select>
                                </div>
                                <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer; transition:background 0.15s;">"추가"</button>
                            </form>
                        </div>
                    </div>
                </div>
            </div>

            <div id="edit-modal" style="display:none; position:fixed; inset:0; z-index:100; background:rgba(0,0,0,0.6); align-items:center; justify-content:center;" onclick="if(event.target===this)this.style.display='none'">
                <script>
                    {r#"function fillCatEdit(btn){var d=btn.dataset;document.getElementById('edit-id').value=d.id;document.getElementById('edit-name').value=d.name;document.getElementById('edit-color').value=d.color;document.getElementById('edit-color-pick').value=d.color;document.getElementById('edit-desc').value=d.desc;document.getElementById('edit-perm').value=d.perm;document.getElementById('edit-modal').style.display='flex'}"#}
                </script>
                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:12px; padding:28px; width:420px; max-width:90vw;">
                    <h3 style="font-size:16px; font-weight:700; color:#f9fafb; margin-bottom:20px;">"카테고리 수정"</h3>
                    <form action="/admin/categories/update" method="POST" style="display:flex; flex-direction:column; gap:14px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="hidden" id="edit-id" name="id">
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"이름"</label>
                            <input type="text" id="edit-name" name="name" required="true" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"색상"</label>
                            <div style="display:flex; gap:8px; align-items:center;">
                                <input type="color" id="edit-color-pick" value="#6b7280" style="width:40px; height:36px; border:none; cursor:pointer; background:transparent;" oninput="document.getElementById('edit-color').value=this.value">
                                <input type="text" id="edit-color" name="color" value="#6b7280" style="width:100px; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:8px 10px; font-size:13px; color:#e5e7eb; outline:none; font-family:monospace;" oninput="document.getElementById('edit-color-pick').value=this.value">
                            </div>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"설명"</label>
                            <input type="text" id="edit-desc" name="description" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"쓰기 권한"</label>
                            <select id="edit-perm" name="write_permission" style=(input_style)>
                                <option value="everyone">"전체 (로그인 사용자)"</option>
                                <option value="editor">"운영진 (관리자 + 에디터)"</option>
                                <option value="admin">"관리자만"</option>
                            </select>
                        </div>
                        <div style="display:flex; gap:8px; justify-content:flex-end; margin-top:8px;">
                            <button type="button" onclick="document.getElementById('edit-modal').style.display='none'" style="background:#2a2d3a; color:#9ca3af; padding:10px 20px; border-radius:8px; font-weight:500; font-size:14px; border:none; cursor:pointer;">"취소"</button>
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-weight:600; font-size:14px; border:none; cursor:pointer;">"저장"</button>
                        </div>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}
