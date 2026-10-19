use crate::services::navigation::{FOOTER_GROUPS, SECTIONS, valid_href};
use crate::{
    admin_ui::ADMIN_STYLE, components::admin_sidebar::AdminSidebar, db::Db, models::NavItem,
};
use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        page, route,
    },
    view::{View, view},
};

#[derive(serde::Deserialize)]
pub struct NavItemForm {
    pub csrf_token: String,
    pub label: String,
    pub href: String,
    pub position: Option<i32>,
    pub section: String,
    #[serde(default = "default_group")]
    pub footer_group: String,
    #[serde(default)]
    pub required_permission: String,
}
fn default_group() -> String {
    "service".into()
}

#[derive(serde::Deserialize)]
pub struct UpdateNavItemForm {
    pub csrf_token: String,
    pub item_id: String,
    pub label: String,
    pub href: String,
    #[serde(default = "default_group")]
    pub footer_group: String,
}
#[derive(serde::Deserialize)]
pub struct ItemForm {
    pub csrf_token: String,
    pub item_id: String,
}
#[derive(serde::Deserialize)]
pub struct PermForm {
    pub csrf_token: String,
    pub item_id: String,
    pub required_permission: String,
}
#[derive(serde::Deserialize)]
pub struct MoveNavItemForm {
    pub csrf_token: String,
    pub item_id: String,
    pub direction: String,
}

async fn authorized(cx: &Cx, csrf: &str, action: &str) -> bool {
    let db: &Db = topcoat::context::app_context(cx);
    crate::auth::access::require_perm(db, cx, crate::auth::access::perms::SETTINGS_MANAGE)
        .await
        .is_some()
        && crate::auth::session::verify_csrf_logged(db, cx, csrf, action).await
}
fn error(code: &str) -> SeeOther {
    see_other(format!("/admin/menus?error={code}"))
}
fn saved(
    result: std::result::Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
    success: &str,
) -> SeeOther {
    match result {
        Ok(r) if r.rows_affected() > 0 => see_other(format!("/admin/menus?{success}=1")),
        Ok(_) => error("missing"),
        Err(e) => {
            eprintln!("[admin] menu write failed: {e}");
            error("database")
        }
    }
}
fn valid_group(group: &str) -> bool {
    FOOTER_GROUPS.iter().any(|(key, _)| *key == group)
}
fn input_error(label: &str, href: &str, group: &str) -> Option<&'static str> {
    if label.trim().is_empty() || href.trim().is_empty() {
        Some("empty")
    } else if !valid_href(href.trim()) {
        Some("href")
    } else if !valid_group(group) {
        Some("group")
    } else {
        None
    }
}
async fn permission_exists(db: &Db, perm: &str) -> std::result::Result<bool, sqlx::Error> {
    if perm.is_empty() {
        return Ok(true);
    }
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM permissions WHERE key = $1)")
        .bind(perm)
        .fetch_one(db)
        .await
}
async fn duplicate(
    db: &Db,
    href: &str,
    section: &str,
    excluded: &str,
) -> std::result::Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM nav_items WHERE href = $1 AND section = $2 AND id <> $3)",
    )
    .bind(href)
    .bind(section)
    .bind(excluded)
    .fetch_one(db)
    .await
}

#[route(POST "/admin/menus/add")]
pub async fn admin_menu_add(cx: &Cx, Form(input): Form<NavItemForm>) -> Result<SeeOther> {
    if !authorized(cx, &input.csrf_token, "admin.menus.add").await {
        return Ok(see_other("/admin/login"));
    }
    if let Some(code) = input_error(&input.label, &input.href, &input.footer_group) {
        return Ok(error(code));
    }
    if !SECTIONS.iter().any(|(key, _)| *key == input.section)
        || input.position.is_some_and(|p| p < 0)
    {
        return Ok(error("section"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    let perm = input.required_permission.trim();
    if !permission_exists(db, perm).await? {
        return Ok(error("perm"));
    }
    if duplicate(db, input.href.trim(), &input.section, "").await? {
        return Ok(error("exists"));
    }
    let result = sqlx::query("INSERT INTO nav_items (label, href, position, section, required_permission, footer_group) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(input.label.trim()).bind(input.href.trim()).bind(input.position.unwrap_or(0))
        .bind(&input.section).bind(perm).bind(&input.footer_group).execute(db).await;
    Ok(saved(result, "added"))
}

#[route(POST "/admin/menus/update")]
pub async fn admin_menu_update(cx: &Cx, Form(input): Form<UpdateNavItemForm>) -> Result<SeeOther> {
    if !authorized(cx, &input.csrf_token, "admin.menus.update").await {
        return Ok(see_other("/admin/login"));
    }
    if let Some(code) = input_error(&input.label, &input.href, &input.footer_group) {
        return Ok(error(code));
    }
    let db: &Db = topcoat::context::app_context(cx);
    let section: Option<String> = sqlx::query_scalar("SELECT section FROM nav_items WHERE id = $1")
        .bind(&input.item_id)
        .fetch_optional(db)
        .await?;
    let Some(section) = section else {
        return Ok(error("missing"));
    };
    if duplicate(db, input.href.trim(), &section, &input.item_id).await? {
        return Ok(error("exists"));
    }
    let result = sqlx::query("UPDATE nav_items SET label=$1, href=$2, footer_group=$3, updated_at=NOW()::text WHERE id=$4")
        .bind(input.label.trim()).bind(input.href.trim()).bind(&input.footer_group).bind(&input.item_id).execute(db).await;
    Ok(saved(result, "updated"))
}

#[route(POST "/admin/menus/delete")]
pub async fn admin_menu_delete(cx: &Cx, Form(input): Form<ItemForm>) -> Result<SeeOther> {
    if !authorized(cx, &input.csrf_token, "admin.menus.delete").await {
        return Ok(see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    Ok(saved(
        sqlx::query("DELETE FROM nav_items WHERE id=$1")
            .bind(&input.item_id)
            .execute(db)
            .await,
        "deleted",
    ))
}

#[route(POST "/admin/menus/toggle")]
pub async fn admin_menu_toggle(cx: &Cx, Form(input): Form<ItemForm>) -> Result<SeeOther> {
    if !authorized(cx, &input.csrf_token, "admin.menus.toggle").await {
        return Ok(see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    Ok(saved(sqlx::query("UPDATE nav_items SET visible=CASE WHEN visible=1 THEN 0 ELSE 1 END, updated_at=NOW()::text WHERE id=$1")
        .bind(&input.item_id).execute(db).await, "toggled"))
}

#[route(POST "/admin/menus/move")]
pub async fn admin_menu_move(cx: &Cx, Form(input): Form<MoveNavItemForm>) -> Result<SeeOther> {
    if !authorized(cx, &input.csrf_token, "admin.menus.move").await {
        return Ok(see_other("/admin/login"));
    }
    if !matches!(input.direction.as_str(), "up" | "down") {
        return Ok(error("direction"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    Ok(
        match crate::services::navigation::move_item(db, &input.item_id, &input.direction).await {
            Ok(true) => see_other("/admin/menus?moved=1"),
            Ok(false) => error("missing"),
            Err(e) => {
                eprintln!("[admin] menu move failed: {e}");
                error("database")
            }
        },
    )
}

#[route(POST "/admin/menus/permission")]
pub async fn admin_menu_permission(cx: &Cx, Form(input): Form<PermForm>) -> Result<SeeOther> {
    if !authorized(cx, &input.csrf_token, "admin.menus.permission").await {
        return Ok(see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    let perm = input.required_permission.trim();
    if !permission_exists(db, perm).await? {
        return Ok(error("perm"));
    }
    let result = sqlx::query(
        "UPDATE nav_items SET required_permission=$1, updated_at=NOW()::text WHERE id=$2",
    )
    .bind(perm)
    .bind(&input.item_id)
    .execute(db)
    .await;
    if result.as_ref().is_ok_and(|r| r.rows_affected() > 0) {
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
            perm,
        )
        .await;
    }
    Ok(saved(result, "saved"))
}

#[page("/admin/menus")]
pub async fn admin_menus_page(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    if crate::auth::access::require_perm(db, cx, crate::auth::access::perms::SETTINGS_MANAGE)
        .await
        .is_none()
    {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let all_items = sqlx::query_as::<_, NavItem>("SELECT * FROM nav_items ORDER BY position, id")
        .fetch_all(db)
        .await?;
    let all_perms: Vec<String> = sqlx::query_scalar("SELECT key FROM permissions ORDER BY key")
        .fetch_all(db)
        .await?;
    let error_code = crate::util::query_param(cx, "error");
    let error_message = match error_code.as_deref() {
        Some("exists") => "해당 섹션에 같은 경로의 메뉴가 이미 존재합니다.",
        Some("empty") => "라벨과 경로를 모두 입력하세요.",
        Some("href") => "경로는 /로 시작하는 사이트 경로 또는 http(s) 주소를 입력하세요.",
        Some("perm") => "존재하지 않는 권한입니다.",
        Some("missing") => "메뉴가 존재하지 않습니다. 새로고침 후 다시 확인하세요.",
        Some("group" | "section" | "direction") => "메뉴 위치 또는 그룹 설정이 올바르지 않습니다.",
        Some(_) => "저장하지 못했습니다. 잠시 후 다시 시도하세요.",
        None => "",
    };
    let success = ["added", "updated", "deleted", "toggled", "moved", "saved"]
        .iter()
        .any(|key| crate::util::query_param(cx, key).as_deref() == Some("1"));
    let toggle_style = |visible: i32| -> String {
        format!(
            "background:none; border:1px solid var(--border); color:{}; padding:4px 10px; border-radius:6px; font-size:11px; cursor:pointer;",
            if visible == 1 {
                "var(--green)"
            } else {
                "var(--muted2)"
            }
        )
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
                <div style="flex:1; min-width:0; padding:24px; overflow-y:auto;">
                    <h1>"메뉴 관리"</h1>
                    <p style="color:var(--muted2); margin:8px 0 24px;">"상단 메뉴는 PC와 모바일 전체 메뉴에, 푸터는 페이지 맨 아래에 표시됩니다. 모바일 하단 탭은 별도로 관리하며 로그인·프로필 버튼은 자동 표시됩니다."</p>
                    if !error_message.is_empty() {
                        <div role="alert" style="padding:16px; margin-bottom:16px; color:var(--red-soft);">(error_message)</div>
                    } else if success {
                        <div role="status" style="padding:16px; margin-bottom:16px; color:var(--green);">"변경사항이 저장되었습니다."</div>
                    }
                    for (section, section_label) in SECTIONS {
                        let items: Vec<&NavItem> = all_items.iter().filter(|i| i.section == section).collect();
                        let section_help = match section {
                            "header" => "PC 상단과 모바일 전체 메뉴에 표시됩니다.",
                            "footer" => "PC와 모바일 페이지 하단에 표시됩니다. 그룹별로 묶어 출력합니다.",
                            _ => "모바일 고정 하단에 표시됩니다. 메뉴가 많으면 가로로 스크롤할 수 있습니다.",
                        };
    <div>
                            <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden; margin-bottom:20px;">
                                <div style="padding:16px 20px; border-bottom:1px solid var(--border);">
                                    <h2 style="font-size:14px; font-weight:700; color:var(--text2);">(section_label)</h2>
                                    <p style="font-size:11px; color:var(--muted2); margin-top:2px;">(section_help)</p>
                                </div>
                                if items.is_empty() {
                                    <div style="padding:40px; text-align:center;">
                                        <p style="font-size:13px; color:var(--dim);">"등록된 메뉴가 없습니다"</p>
                                    </div>
                                } else {
                                    for item in &items {
                                        <div class="menu-row" style="display:flex; flex-wrap:wrap; align-items:center; gap:12px; padding:12px 20px; border-bottom:1px solid var(--line);">
                                            <div style="color:var(--dim); font-size:13px; font-weight:600; width:20px; text-align:center;">(item.position)</div>
                                            <form action="/admin/menus/update" method="POST" style="flex:1; min-width:0; display:flex; gap:6px; align-items:center;">
                                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                <input type="hidden" name="item_id" value=(item.id.clone())>
                                                <input type="text" name="label" required="true" value=(item.label.clone()) style="width:130px; background:var(--bg); border:1px solid var(--border); border-radius:6px; padding:6px 8px; font-size:12px; font-weight:600; color:var(--text3); outline:none;">
                                                <input type="text" name="href" required="true" value=(item.href.clone()) style="flex:1; min-width:0; background:var(--bg); border:1px solid var(--border); border-radius:6px; padding:6px 8px; font-size:11px; font-family:monospace; color:var(--dim); outline:none;">
                                                if section == "footer" {
                                                    <select name="footer_group" aria-label="푸터 그룹">
                                                        for (key, label) in FOOTER_GROUPS {
                                                            if key == item.footer_group {
                                                                <option value=(key) selected="true">(label)</option>
                                                            } else {
                                                                <option value=(key)>(label)</option>
                                                            }
                                                        }
                                                    </select>
                                                }
                                                <button type="submit" style="background:none; border:1px solid rgba(96,165,250,0.3); color:var(--blue); padding:4px 8px; border-radius:6px; font-size:11px; cursor:pointer; white-space:nowrap;">"저장"</button>
                                            </form>
                                            <div style="display:flex; gap:2px;">
                                                <form action="/admin/menus/move" method="POST" style="display:inline;">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <input type="hidden" name="item_id" value=(item.id.clone())>
                                                    <input type="hidden" name="direction" value="up">
                                                    <button type="submit" style="background:none; border:none; color:var(--muted2); cursor:pointer; font-size:12px; padding:4px;" title="위로">"▲"</button>
                                                </form>
                                                <form action="/admin/menus/move" method="POST" style="display:inline;">
                                                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                                    <input type="hidden" name="item_id" value=(item.id.clone())>
                                                    <input type="hidden" name="direction" value="down">
                                                    <button type="submit" style="background:none; border:none; color:var(--muted2); cursor:pointer; font-size:12px; padding:4px;" title="아래로">"▼"</button>
                                                </form>
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
                    }
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-top:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"새 메뉴 추가"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"내비게이션에 새로운 메뉴 항목을 추가합니다."</p>

                        <form action="/admin/menus/add" method="POST" style="display:grid; grid-template-columns:repeat(auto-fit,minmax(150px,1fr)); gap:12px; align-items:end;">
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
                                    <option value="mobile">"모바일 하단 탭"</option>
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
                            <div>
                                <label>"푸터 그룹 (푸터 메뉴에만 적용)"</label>
                                <select name="footer_group">
                                    for (key, label) in FOOTER_GROUPS {
                                        <option value=(key)>(label)</option>
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
