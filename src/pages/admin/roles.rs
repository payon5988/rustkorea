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
use crate::models::{Permission, Role};

#[derive(serde::Deserialize)]
pub struct CreateRoleForm {
    pub csrf_token: String,
    pub name: String,
    pub description: String,
}

#[derive(serde::Deserialize)]
pub struct DeleteRoleForm {
    pub csrf_token: String,
    pub role_id: String,
}

fn valid_role_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty()
        && n.len() <= 32
        && n.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[page("/admin/roles")]
pub async fn admin_roles_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "roles.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let roles = sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY position ASC")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    let permissions =
        sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY category ASC, key ASC")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    let grants: Vec<(String, String)> =
        sqlx::query_as("SELECT role_id, permission_id FROM role_permissions")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    let member_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT role_id, COUNT(*) FROM users GROUP BY role_id")
            .fetch_all(db)
            .await
            .unwrap_or_default();

    let query_str = topcoat::router::request::uri(cx).query().unwrap_or("");

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"역할/권한 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/roles".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:#f9fafb; letter-spacing:-0.01em;">"역할/권한 관리"</h1>
                    <p style="font-size:13px; color:#6b7280; margin-top:2px;">"역할을 만들고 기능별 권한을 부여합니다. 시스템 역할(관리자/에디터/일반)은 삭제할 수 없습니다."</p>
                </div>

                if query_str.contains("saved=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#34d399;">"권한 매트릭스가 저장되었습니다."</div>
                }
                if query_str.contains("created=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#34d399;">"역할이 생성되었습니다."</div>
                }
                if query_str.contains("deleted=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#34d399;">"역할이 삭제되고 소속 사용자는 일반 회원으로 이동했습니다."</div>
                }
                if query_str.contains("error=") {
                    <div style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:#fca5a5;">"요청을 처리할 수 없습니다. 이름을 확인하세요(소문자·숫자·밑줄, 32자 이내)."</div>
                }

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden; margin-bottom:20px;">
                    <div style="padding:16px 20px; border-bottom:1px solid #2a2d3a;">
                        <span style="font-size:14px; font-weight:700; color:#e5e7eb;">"역할 목록"</span>
                    </div>
                    <table style="width:100%; border-collapse:collapse; font-size:13px;">
                        <thead>
                            <tr>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"역할"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"설명"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"회원수"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"권한수"</th>
                                <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; text-transform:uppercase; letter-spacing:0.05em; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"관리"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for r in &roles {
                                <tr>
                                    <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                        <span style="font-weight:600; color:#d1d5db;">(crate::auth::access::role_label(&r.name))</span>
                                        <span style="font-size:11px; color:#4b5563; font-family:monospace; margin-left:8px;">(r.id.clone())</span>
                                        if r.is_superuser == 1 {
                                            <span style="background:rgba(239,68,68,0.1); color:#f87171; padding:2px 8px; border-radius:4px; font-size:11px; font-weight:600; margin-left:8px;">"슈퍼유저"</span>
                                        }
                                        if r.is_system == 1 {
                                            <span style="background:rgba(107,114,128,0.1); color:#9ca3af; padding:2px 8px; border-radius:4px; font-size:11px; font-weight:600; margin-left:8px;">"시스템"</span>
                                        }
                                    </td>
                                    <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#9ca3af;">(r.description.clone())</td>
                                    <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#9ca3af;">(member_counts.iter().find(|(id, _)| id == &r.id).map(|(_, c)| c.to_string()).unwrap_or_else(|| "0".to_string()))</td>
                                    <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03); color:#9ca3af;">(grants.iter().filter(|(rid, _)| rid == &r.id).count())</td>
                                    <td style="padding:14px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                        if r.is_system == 0 {
                                            <form action="/admin/roles/delete" method="POST" style="display:inline;" onsubmit="return confirm('이 역할을 삭제할까요? 소속 사용자는 일반 회원으로 이동합니다.')">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="role_id" value=(r.id.clone())>
                                                <button type="submit" style="background:none; border:1px solid rgba(239,68,68,0.3); color:#f87171; padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer; font-weight:600;">"삭제"</button>
                                            </form>
                                        }
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>

                <form action="/admin/roles/matrix" method="POST">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; overflow:hidden; margin-bottom:20px;">
                        <div style="padding:16px 20px; border-bottom:1px solid #2a2d3a; display:flex; align-items:center; justify-content:space-between;">
                            <span style="font-size:14px; font-weight:700; color:#e5e7eb;">"권한 매트릭스"</span>
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:8px 16px; border-radius:8px; font-size:13px; font-weight:600; border:none; cursor:pointer;">"매트릭스 저장"</button>
                        </div>
                        <table style="width:100%; border-collapse:collapse; font-size:13px;">
                            <thead>
                                <tr>
                                    <th style="padding:12px 20px; text-align:left; font-size:11px; font-weight:600; color:#6b7280; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">"권한"</th>
                                    for r in &roles {
                                        <th style="padding:12px 20px; text-align:center; font-size:11px; font-weight:600; color:#6b7280; border-bottom:1px solid #2a2d3a; background:rgba(255,255,255,0.02);">(crate::auth::access::role_label(&r.name))</th>
                                    }
                                </tr>
                            </thead>
                            <tbody>
                                for p in &permissions {
                                    <tr>
                                        <td style="padding:10px 20px; border-bottom:1px solid rgba(255,255,255,0.03);">
                                            <div style="font-family:monospace; font-size:12px; color:#d1d5db;">(p.key.clone())</div>
                                            <div style="font-size:11px; color:#4b5563;">(p.description.clone())</div>
                                        </td>
                                        for r in &roles {
                                            <td style="padding:10px 20px; border-bottom:1px solid rgba(255,255,255,0.03); text-align:center;">
                                                if grants.iter().any(|(rid, pid)| rid == &r.id && pid == &p.id) {
                                                    <input type="checkbox" name="grant" value=(format!("{}:{}", r.id, p.id)) checked="true" style="width:16px; height:16px; accent-color:#DE5B30; cursor:pointer;">
                                                } else {
                                                    <input type="checkbox" name="grant" value=(format!("{}:{}", r.id, p.id)) style="width:16px; height:16px; accent-color:#DE5B30; cursor:pointer;">
                                                }
                                            </td>
                                        }
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                </form>

                <div style="background:#1a1d27; border:1px solid #2a2d3a; border-radius:10px; padding:24px; max-width:560px;">
                    <h2 style="font-size:15px; font-weight:700; color:#f9fafb; margin-bottom:4px;">"역할 추가"</h2>
                    <p style="font-size:12px; color:#6b7280; margin-bottom:20px;">"이름은 소문자·숫자·밑줄만, 32자 이내 (예: vip, partner)"</p>
                    <form action="/admin/roles/create" method="POST">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div style="margin-bottom:16px;">
                            <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"이름(키)"</label>
                            <input type="text" name="name" required="true" placeholder="vip" style="width:100%; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;">
                        </div>
                        <div style="margin-bottom:16px;">
                            <label style="display:block; font-size:12px; font-weight:600; color:#9ca3af; margin-bottom:6px;">"설명"</label>
                            <input type="text" name="description" placeholder="유료 회원" style="width:100%; background:#0f1117; border:1px solid #2a2d3a; border-radius:8px; padding:10px 14px; font-size:14px; color:#e5e7eb; outline:none;">
                        </div>
                        <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"역할 생성"</button>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/roles/matrix")]
pub async fn admin_save_matrix(
    cx: &Cx,
    Form(items): Form<Vec<(String, String)>>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "roles.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &crate::db::Db = topcoat::context::app_context(cx);

    // Rebuild grants for non-system roles from the submitted checkboxes.
    // System-role rows are preserved so the console can never lock itself out.
    let csrf = items
        .iter()
        .find(|(k, _)| k == "csrf_token")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    if !crate::auth::session::verify_csrf_logged(db, cx, &csrf, "admin.roles.matrix").await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let mut wanted: Vec<(String, String)> = vec![];
    for (key, value) in items {
        if key == "csrf_token" {
            continue;
        }
        if key == "grant" {
            if let Some((rid, pid)) = value.split_once(':') {
                wanted.push((rid.to_string(), pid.to_string()));
            }
        }
    }
    let editable: Vec<String> = sqlx::query_scalar("SELECT id FROM roles WHERE is_system = 0")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    let valid_perms: Vec<String> = sqlx::query_scalar("SELECT id FROM permissions")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(_) => return Ok(topcoat::router::error::see_other("/admin/roles?error=1")),
    };
    for rid in &editable {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(rid)
            .execute(&mut *tx)
            .await
            .ok();
    }
    for (rid, pid) in &wanted {
        if editable.contains(rid) && valid_perms.contains(pid) {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(rid)
                .bind(pid)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }
    if tx.commit().await.is_err() {
        return Ok(topcoat::router::error::see_other("/admin/roles?error=1"));
    }

    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(
        db,
        cx,
        &actor,
        "admin.roles.matrix",
        "role",
        "",
        &format!("{} grants", wanted.len()),
    )
    .await;

    Ok(topcoat::router::error::see_other("/admin/roles?saved=1"))
}

#[route(POST "/admin/roles/create")]
pub async fn admin_create_role(
    cx: &Cx,
    Form(input): Form<CreateRoleForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "roles.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let name = input.name.trim().to_lowercase();
    if !valid_role_name(&name) {
        return Ok(topcoat::router::error::see_other("/admin/roles?error=1"));
    }

    let db: &crate::db::Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.roles.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let max_pos: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(position), 0) FROM roles")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let done = sqlx::query(
        "INSERT INTO roles (id, name, description, is_system, is_superuser, position)
         VALUES ($1, $1, $2, 0, 0, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(&name)
    .bind(input.description.trim())
    .bind(max_pos + 1)
    .execute(db)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);

    if done != 1 {
        return Ok(topcoat::router::error::see_other("/admin/roles?error=1"));
    }
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(db, cx, &actor, "admin.roles.create", "role", &name, "").await;

    Ok(topcoat::router::error::see_other("/admin/roles?created=1"))
}

#[route(POST "/admin/roles/delete")]
pub async fn admin_delete_role(
    cx: &Cx,
    Form(input): Form<DeleteRoleForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "roles.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let db: &crate::db::Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.roles.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let target: Option<Role> = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
        .bind(&input.role_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    let Some(target) = target else {
        return Ok(topcoat::router::error::see_other("/admin/roles?error=1"));
    };
    if target.is_system == 1 {
        return Ok(topcoat::router::error::see_other("/admin/roles?error=1"));
    }

    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(_) => return Ok(topcoat::router::error::see_other("/admin/roles?error=1")),
    };
    sqlx::query("UPDATE users SET role_id = 'member' WHERE role_id = $1")
        .bind(&target.id)
        .execute(&mut *tx)
        .await
        .ok();
    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
        .bind(&target.id)
        .execute(&mut *tx)
        .await
        .ok();
    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(&target.id)
        .execute(&mut *tx)
        .await
        .ok();
    if tx.commit().await.is_err() {
        return Ok(topcoat::router::error::see_other("/admin/roles?error=1"));
    }
    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(db, cx, &actor, "admin.roles.delete", "role", &target.id, "").await;

    Ok(topcoat::router::error::see_other("/admin/roles?deleted=1"))
}
