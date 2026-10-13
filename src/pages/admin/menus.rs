use std::future::Future;
use topcoat::router::request::uri;
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

#[derive(serde::Deserialize)]
pub struct NavItemForm {
    pub csrf_token: String,
    pub label: String,
    pub href: String,
    pub position: Option<i32>,
    pub section: String,
    pub required_permission: String,
}

#[derive(serde::Deserialize)]
pub struct PermForm {
    pub csrf_token: String,
    pub item_id: String,
    pub required_permission: String,
}

#[derive(serde::Deserialize)]
pub struct DeleteNavItemForm {
    pub csrf_token: String,
    pub item_id: String,
}

#[derive(serde::Deserialize)]
pub struct ToggleNavItemForm {
    pub csrf_token: String,
    pub item_id: String,
}

#[page("/admin/menus")]
pub async fn admin_menus_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let header_items = sqlx::query_as::<_, crate::models::NavItem>(
        "SELECT * FROM nav_items WHERE section = 'header' ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let footer_items = sqlx::query_as::<_, crate::models::NavItem>(
        "SELECT * FROM nav_items WHERE section = 'footer' ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let all_perms: Vec<String> = sqlx::query_scalar("SELECT key FROM permissions ORDER BY key ASC")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let query_str = uri(cx).query().unwrap_or("");

    let toggle_style = |visible: i32| -> String {
        if visible == 1 {
            "background:none; border:1px solid rgba(52,211,153,0.3); color:var(--green); padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;".into()
        } else {
            "background:none; border:1px solid var(--line-strong); color:var(--muted2); padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;".into()
        }
    };
    let toggle_label = |visible: i32| -> &str { if visible == 1 { "표시" } else { "숨김" } };

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"메뉴 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/menus".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"메뉴 관리"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"헤더/푸터 내비게이션 메뉴를 관리합니다."</p>
                </div>

                if query_str.contains("error=exists") {
                    <div style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:var(--red-soft);">"해당 경로의 메뉴가 이미 존재합니다."</div>
                } else if query_str.contains("error=empty") {
                    <div style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:var(--red-soft);">"라벨과 경로를 모두 입력하세요."</div>
                }
                if query_str.contains("added=1") || query_str.contains("deleted=1") || query_str.contains("toggled=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:var(--green);">"변경사항이 저장되었습니다."</div>
                }

                <div style="display:grid; grid-template-columns:1fr 1fr; gap:20px;">
                    <div>
                        <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden; margin-bottom:20px;">
                            <div style="padding:16px 20px; border-bottom:1px solid var(--border);">
                                <h2 style="font-size:14px; font-weight:700; color:var(--text2);">"헤더 메뉴"</h2>
                                <p style="font-size:11px; color:var(--muted2); margin-top:2px;">"상단 네비게이션에 표시됩니다."</p>
                            </div>
                            if header_items.is_empty() {
                                <div style="padding:40px; text-align:center;">
                                    <p style="font-size:13px; color:var(--dim);">"등록된 메뉴가 없습니다"</p>
                                </div>
                            } else {
                                for item in &header_items {
                                    <div style="display:flex; align-items:center; gap:12px; padding:12px 20px; border-bottom:1px solid var(--line);">
                                        <div style="color:var(--dim); font-size:13px; font-weight:600; width:20px; text-align:center;">(item.position)</div>
                                        <div style="flex:1; min-width:0;">
                                            <div style="font-size:13px; font-weight:600; color:var(--text3);">(item.label.clone())</div>
                                            <div style="font-size:11px; color:var(--dim); font-family:monospace; margin-top:2px;">(item.href.clone())</div>
                                            if !item.required_permission.is_empty() {
                                                <div style="font-size:11px; color:var(--purple); font-family:monospace; margin-top:2px;">(format!("🔒 {}", item.required_permission))</div>
                                            }
                                        </div>
                                        <form action="/admin/menus/permission" method="POST" style="display:flex; gap:4px; align-items:center;">
                                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                            <input type="hidden" name="item_id" value=(item.id.clone())>
                                            <select name="required_permission" style="background:var(--bg); border:1px solid var(--border); border-radius:6px; padding:4px 6px; font-size:11px; color:var(--text2); outline:none; max-width:140px;">
                                                if item.required_permission.is_empty() {
                                                    <option value="" selected="true">"전체"</option>
                                                } else {
                                                    <option value="">"전체"</option>
                                                }
                                                for perm in &all_perms {
                                                    if *perm == item.required_permission {
                                                        <option value=(perm.clone()) selected="true">(perm.clone())</option>
                                                    } else {
                                                        <option value=(perm.clone())>(perm.clone())</option>
                                                    }
                                                }
                                            </select>
                                            <button type="submit" style="background:none; border:1px solid rgba(96,165,250,0.3); color:var(--blue); padding:4px 8px; border-radius:6px; font-size:11px; cursor:pointer;">"저장"</button>
                                        </form>
                                        <div style="display:flex; gap:6px;">
                                            <form action="/admin/menus/toggle" method="POST" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="item_id" value=(item.id.clone())>
                                                <button type="submit" style=(toggle_style(item.visible))>(toggle_label(item.visible))</button>
                                            </form>
                                            <form action="/admin/menus/delete" method="POST" style="display:inline;" onsubmit="return confirm('이 메뉴를 삭제하시겠습니까?')">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="item_id" value=(item.id.clone())>
                                                <button type="submit" style="background:none; border:1px solid rgba(239,68,68,0.3); color:var(--red); padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600; transition:all 0.15s;">"삭제"</button>
                                            </form>
                                        </div>
                                    </div>
                                }
                            }
                        </div>
                    </div>

                    <div>
                        <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden; margin-bottom:20px;">
                            <div style="padding:16px 20px; border-bottom:1px solid var(--border);">
                                <h2 style="font-size:14px; font-weight:700; color:var(--text2);">"푸터 메뉴"</h2>
                                <p style="font-size:11px; color:var(--muted2); margin-top:2px;">"하단 영역에 표시됩니다."</p>
                            </div>
                            if footer_items.is_empty() {
                                <div style="padding:40px; text-align:center;">
                                    <p style="font-size:13px; color:var(--dim);">"등록된 메뉴가 없습니다"</p>
                                </div>
                            } else {
                                for item in &footer_items {
                                    <div style="display:flex; align-items:center; gap:12px; padding:12px 20px; border-bottom:1px solid var(--line);">
                                        <div style="color:var(--dim); font-size:13px; font-weight:600; width:20px; text-align:center;">(item.position)</div>
                                        <div style="flex:1; min-width:0;">
                                            <div style="font-size:13px; font-weight:600; color:var(--text3);">(item.label.clone())</div>
                                            <div style="font-size:11px; color:var(--dim); font-family:monospace; margin-top:2px;">(item.href.clone())</div>
                                            if !item.required_permission.is_empty() {
                                                <div style="font-size:11px; color:var(--purple); font-family:monospace; margin-top:2px;">(format!("🔒 {}", item.required_permission))</div>
                                            }
                                        </div>
                                        <form action="/admin/menus/permission" method="POST" style="display:flex; gap:4px; align-items:center;">
                                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                            <input type="hidden" name="item_id" value=(item.id.clone())>
                                            <select name="required_permission" style="background:var(--bg); border:1px solid var(--border); border-radius:6px; padding:4px 6px; font-size:11px; color:var(--text2); outline:none; max-width:140px;">
                                                if item.required_permission.is_empty() {
                                                    <option value="" selected="true">"전체"</option>
                                                } else {
                                                    <option value="">"전체"</option>
                                                }
                                                for perm in &all_perms {
                                                    if *perm == item.required_permission {
                                                        <option value=(perm.clone()) selected="true">(perm.clone())</option>
                                                    } else {
                                                        <option value=(perm.clone())>(perm.clone())</option>
                                                    }
                                                }
                                            </select>
                                            <button type="submit" style="background:none; border:1px solid rgba(96,165,250,0.3); color:var(--blue); padding:4px 8px; border-radius:6px; font-size:11px; cursor:pointer;">"저장"</button>
                                        </form>
                                        <div style="display:flex; gap:6px;">
                                            <form action="/admin/menus/toggle" method="POST" style="display:inline;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="item_id" value=(item.id.clone())>
                                                <button type="submit" style=(toggle_style(item.visible))>(toggle_label(item.visible))</button>
                                            </form>
                                            <form action="/admin/menus/delete" method="POST" style="display:inline;" onsubmit="return confirm('이 메뉴를 삭제하시겠습니까?')">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="item_id" value=(item.id.clone())>
                                                <button type="submit" style="background:none; border:1px solid rgba(239,68,68,0.3); color:var(--red); padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600; transition:all 0.15s;">"삭제"</button>
                                            </form>
                                        </div>
                                    </div>
                                }
                            }
                        </div>
                    </div>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-top:20px;">
                    <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"새 메뉴 추가"</h2>
                    <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"내비게이션에 새로운 메뉴 항목을 추가합니다."</p>

                    <form action="/admin/menus/add" method="POST" style="display:grid; grid-template-columns:1fr 1fr auto auto; gap:12px; align-items:end;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"라벨"</label>
                            <input type="text" name="label" required="true" placeholder="메뉴 이름" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"경로"</label>
                            <input type="text" name="href" required="true" placeholder="/page" style="width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none; font-family:monospace;">
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"위치"</label>
                            <input type="number" name="position" value="0" min="0" style="width:80px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;">
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"섹션"</label>
                            <select name="section" style="background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none; appearance:auto;">
                                <option value="header">"헤더"</option>
                                <option value="footer">"푸터"</option>
                            </select>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"필요 권한 (비우면 전체)"</label>
                            <select name="required_permission" style="background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none; appearance:auto;">
                                <option value="">"전체"</option>
                                for perm in &all_perms {
                                    <option value=(perm.clone())>(perm.clone())</option>
                                }
                            </select>
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer; white-space:nowrap;">"추가"</button>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/menus/add")]
pub async fn admin_menu_add(
    cx: &Cx,
    Form(input): Form<NavItemForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    if input.label.is_empty() || input.href.is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/menus?error=empty",
        ));
    }

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.menus.add").await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM nav_items WHERE href = $1 AND section = $2")
            .bind(&input.href)
            .bind(&input.section)
            .fetch_one(db)
            .await
            .unwrap_or(0);

    if exists > 0 {
        return Ok(topcoat::router::error::see_other(
            "/admin/menus?error=exists",
        ));
    }

    let position = input.position.unwrap_or(0);

    let perm = input.required_permission.trim().to_string();
    if !perm.is_empty() {
        let ok: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM permissions WHERE key = $1")
            .bind(&perm)
            .fetch_one(db)
            .await
            .unwrap_or(false);
        if !ok {
            return Ok(topcoat::router::error::see_other("/admin/menus?error=perm"));
        }
    }

    sqlx::query("INSERT INTO nav_items (label, href, position, visible, section, required_permission) VALUES ($1, $2, $3, 1, $4, $5)")
        .bind(&input.label)
        .bind(&input.href)
        .bind(position)
        .bind(&input.section)
        .bind(&perm)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other("/admin/menus?added=1"))
}

#[route(POST "/admin/menus/delete")]
pub async fn admin_menu_delete(
    cx: &Cx,
    Form(input): Form<DeleteNavItemForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.menus.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM nav_items WHERE id = $1")
        .bind(&input.item_id)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other("/admin/menus?deleted=1"))
}

#[route(POST "/admin/menus/toggle")]
pub async fn admin_menu_toggle(
    cx: &Cx,
    Form(input): Form<ToggleNavItemForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.menus.toggle")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("UPDATE nav_items SET visible = CASE WHEN visible = 1 THEN 0 ELSE 1 END, updated_at = NOW()::text WHERE id = $1")
        .bind(&input.item_id)
        .execute(db)
        .await
        .ok();

    Ok(topcoat::router::error::see_other("/admin/menus?toggled=1"))
}

#[route(POST "/admin/menus/permission")]
pub async fn admin_menu_permission(
    cx: &Cx,
    Form(input): Form<PermForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
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
        "admin.menus.permission",
    )
    .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let perm = input.required_permission.trim().to_string();
    if !perm.is_empty() {
        let ok: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM permissions WHERE key = $1")
            .bind(&perm)
            .fetch_one(db)
            .await
            .unwrap_or(false);
        if !ok {
            return Ok(topcoat::router::error::see_other("/admin/menus?error=perm"));
        }
    }
    sqlx::query(
        "UPDATE nav_items SET required_permission = $1, updated_at = NOW()::text WHERE id = $2",
    )
    .bind(&perm)
    .bind(&input.item_id)
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
        "admin.menus.permission",
        "nav_items",
        &input.item_id,
        &perm,
    )
    .await;

    Ok(topcoat::router::error::see_other("/admin/menus?saved=1"))
}
