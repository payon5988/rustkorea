use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;
use crate::db::Db;
use crate::models::Badge;

topcoat::router::path_param!(id);

fn slugify_id(raw: &str) -> String {
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

async fn require_settings(cx: &Cx) -> bool {
    let adb: &crate::db::Db = topcoat::context::app_context(cx);
    crate::auth::access::require_perm(adb, cx, "settings.manage")
        .await
        .is_none()
}

#[derive(serde::Deserialize)]
pub struct BadgeForm {
    pub csrf_token: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub points_required: String,
}

#[route(POST "/admin/badges/create")]
pub async fn create_badge(
    cx: &Cx,
    Form(input): Form<BadgeForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_settings(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.badges.create")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let id = slugify_id(&input.id);
    let points: i64 = match input.points_required.trim().parse() {
        Ok(n) if n >= 0 => n,
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/admin/badges?error=invalid",
            ));
        }
    };
    if id.is_empty() || input.name.trim().is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/badges?error=invalid",
        ));
    }
    sqlx::query(
        "INSERT INTO badges (id, name, description, icon, color, points_required) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&id)
    .bind(input.name.trim())
    .bind(input.description.trim())
    .bind(input.icon.trim())
    .bind(input.color.trim())
    .bind(points)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other("/admin/badges"))
}

#[derive(serde::Deserialize)]
pub struct BadgeUpdateForm {
    pub csrf_token: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub points_required: String,
}

#[route(POST "/admin/badges/update")]
pub async fn update_badge(
    cx: &Cx,
    Form(input): Form<BadgeUpdateForm>,
) -> Result<topcoat::router::error::SeeOther> {
    if require_settings(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.badges.update")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let points: i64 = match input.points_required.trim().parse() {
        Ok(n) if n >= 0 => n,
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/admin/badges?error=invalid",
            ));
        }
    };
    if input.name.trim().is_empty() {
        return Ok(topcoat::router::error::see_other(
            "/admin/badges?error=invalid",
        ));
    }
    sqlx::query(
        "UPDATE badges SET name = $1, description = $2, icon = $3, color = $4, points_required = $5 WHERE id = $6",
    )
    .bind(input.name.trim())
    .bind(input.description.trim())
    .bind(input.icon.trim())
    .bind(input.color.trim())
    .bind(points)
    .bind(&input.id)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other("/admin/badges"))
}

#[route(POST "/admin/badges/{id}/delete")]
pub async fn delete_badge(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx);
    if require_settings(cx).await {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    let db: &Db = topcoat::context::app_context(cx);
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.badges.delete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }
    sqlx::query("DELETE FROM user_badges WHERE badge_id = $1")
        .bind(id)
        .execute(db)
        .await
        .ok();
    sqlx::query("DELETE FROM badges WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .ok();
    Ok(topcoat::router::error::see_other("/admin/badges"))
}

#[page("/admin/badges")]
pub async fn admin_badges(cx: &Cx) -> Result<impl View + '_> {
    if require_settings(cx).await {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let badges = sqlx::query_as::<_, Badge>("SELECT * FROM badges ORDER BY points_required ASC")
        .fetch_all(db)
        .await
        .unwrap_or_default();
    let counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT badge_id, COUNT(*) FROM user_badges GROUP BY badge_id")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    let count_map: std::collections::HashMap<String, i64> = counts.into_iter().collect();

    let input_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;";

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"등급/배지 관리 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/badges".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; letter-spacing:-0.01em;">"등급 / 배지 관리"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"포인트 달성 시 자동 수여. 배지가 곧 회원 등급입니다."</p>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; overflow:hidden; margin-bottom:20px;">
                    <table style="width:100%; font-size:13px; border-collapse:collapse;">
                        <thead>
                            <tr style="border-bottom:1px solid var(--border); text-align:left;">
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"등급"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"필요 포인트"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em;">"보유자"</th>
                                <th style="padding:12px 20px; font-size:11px; font-weight:600; color:var(--muted2); text-transform:uppercase; letter-spacing:0.04em; text-align:right;">"관리"</th>
                            </tr>
                        </thead>
                        <tbody>
                            for b in &badges {
                                <tr style="border-bottom:1px solid var(--line);">
                                    <td style="padding:14px 20px;">
                                        <form action="/admin/badges/update" method="POST" style="display:flex; gap:8px; align-items:center; flex-wrap:wrap;">
                                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                            <input type="hidden" name="id" value=(b.id.clone())>
                                            <input type="text" name="icon" value=(b.icon.clone()) placeholder="🏅" style="width:52px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:8px 10px; font-size:14px; color:var(--text2); outline:none; text-align:center;">
                                            <input type="text" name="name" required="true" value=(b.name.clone()) style="width:130px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:8px 10px; font-size:13px; color:var(--text2); outline:none;">
                                            <input type="text" name="description" value=(b.description.clone()) placeholder="설명" style="width:180px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:8px 10px; font-size:12px; color:var(--muted); outline:none;">
                                            <input type="text" name="color" value=(b.color.clone()) placeholder="#6b7280" style="width:90px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:8px 10px; font-size:12px; font-family:monospace; color:var(--muted); outline:none;">
                                            <button type="submit" style="background:none; border:none; font-size:12px; color:var(--blue); cursor:pointer; font-weight:600;">"저장"</button>
                                        </form>
                                    </td>
                                    <td style="padding:14px 20px;">
                                        <form action="/admin/badges/update" method="POST" style="display:flex; gap:8px; align-items:center;">
                                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                            <input type="hidden" name="id" value=(b.id.clone())>
                                            <input type="hidden" name="name" value=(b.name.clone())>
                                            <input type="hidden" name="description" value=(b.description.clone())>
                                            <input type="hidden" name="icon" value=(b.icon.clone())>
                                            <input type="hidden" name="color" value=(b.color.clone())>
                                            <input type="number" name="points_required" required="true" min="0" value=(b.points_required) style="width:100px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:8px 10px; font-size:13px; color:var(--text2); outline:none;">
                                            <button type="submit" style="background:none; border:none; font-size:12px; color:var(--blue); cursor:pointer; font-weight:600;">"저장"</button>
                                        </form>
                                    </td>
                                    <td style="padding:14px 20px; color:var(--muted);">(count_map.get(&b.id).copied().unwrap_or(0))</td>
                                    <td style="padding:14px 20px; text-align:right;">
                                        <form action=(format!("/admin/badges/{}/delete", b.id)) method="POST" onsubmit="return confirm('이 등급을 삭제하시겠습니까? 보유 기록도 함께 삭제됩니다.')" style="display:inline;">
                                            <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                            <button type="submit" style="font-size:12px; color:var(--red); background:none; border:none; cursor:pointer; font-weight:600;">"삭제"</button>
                                        </form>
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>

                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; max-width:720px;">
                    <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:16px;">"새 등급 추가"</h2>
                    <form action="/admin/badges/create" method="POST" style="display:grid; grid-template-columns:1fr 1fr; gap:12px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"ID (영문, 예: vip)"</label>
                            <input type="text" name="id" required="true" placeholder="vip" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"이름 (예: 우수회원)"</label>
                            <input type="text" name="name" required="true" placeholder="우수회원" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"설명"</label>
                            <input type="text" name="description" placeholder="300P 달성" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"아이콘 (이모지)"</label>
                            <input type="text" name="icon" placeholder="🏅" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"색상"</label>
                            <input type="text" name="color" placeholder="#6b7280" style=(input_style)>
                        </div>
                        <div>
                            <label style="display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;">"필요 포인트"</label>
                            <input type="number" name="points_required" required="true" min="0" value="0" style=(input_style)>
                        </div>
                        <div style="grid-column:1 / -1;">
                            <button type="submit" style="background:#DE5B30; color:#fff; padding:10px 20px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"등급 추가"</button>
                        </div>
                    </form>
                </div>
            </div>
        </body>
        </html>
    })
}
