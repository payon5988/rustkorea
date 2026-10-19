use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::db::Db;
use crate::models::User;

async fn get_user(cx: &Cx) -> Option<User> {
    let db: &Db = topcoat::context::app_context(cx);
    crate::auth::session::current_user(db, cx).await
}

async fn unread_count(cx: &Cx, user_id: &str) -> i64 {
    let db: &Db = topcoat::context::app_context(cx);
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = 0",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .unwrap_or(0)
}

#[component]
pub async fn Header(cx: &Cx) -> Result<impl View + '_> {
    let user = get_user(cx).await;
    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;
    let is_admin = access.as_ref().map(|a| a.is_admin()).unwrap_or(false);
    let nav_items =
        crate::services::navigation::visible_items(db, "header", access.as_ref()).await?;
    let mobile_items =
        crate::services::navigation::visible_items(db, "mobile", access.as_ref()).await?;
    let (site_title_val, logo_url_val) = {
        let db: &crate::db::Db = topcoat::context::app_context(cx);
        let t = crate::services::site_settings::get_setting(db, "site_title").await;
        let l = crate::services::site_settings::get_setting(db, "logo_url").await;
        (
            if t.is_empty() {
                "Rust Korea".to_string()
            } else {
                t
            },
            l,
        )
    };
    let notif_count = if let Some(ref u) = user {
        unread_count(cx, &u.id).await
    } else {
        0
    };
    let loc = crate::services::i18n::locale(cx);
    let locale_label = loc.pick("EN", "KO");
    let locale_title = loc.pick("Switch to English", "한국어로 보기");
    let switch_href = crate::services::i18n::switch_href(cx, loc.other());
    let label = move |item: &crate::models::NavItem| loc.text_owned(&item.label, &item.label_en);
    let notif_badge = if notif_count > 0 {
        "position:absolute; top:-4px; right:-4px; min-width:16px; height:16px; border-radius:8px; background:#DC2626; color:#fff; font-size:10px; font-weight:700; display:flex; align-items:center; justify-content:center; padding:0 4px;".to_string()
    } else {
        "display:none;".to_string()
    };

    let is_logged_in = user.is_some();
    let logout_csrf = if is_logged_in {
        crate::auth::session::csrf_token(db, cx).await
    } else {
        String::new()
    };
    let current_path = topcoat::router::request::uri(cx).path().to_string();
    let tab_profile_active = if is_logged_in {
        if let Some(ref u) = user {
            current_path == format!("/users/{}", u.username)
        } else {
            false
        }
    } else {
        current_path.starts_with("/auth")
    };

    let tab_active_style = "color:#2563EB;";
    let tab_inactive_style = "color:#9ca3af;";
    let tab_notif_style = "position:absolute; top:2px; right:4px; min-width:14px; height:14px; border-radius:7px; background:#DC2626; color:#fff; font-size:9px; font-weight:700; display:flex; align-items:center; justify-content:center; padding:0 3px; line-height:1;";

    Ok(view! {
        <header style="position:sticky; top:0; z-index:50; background:#fff; border-bottom:1px solid #e5e7eb; box-shadow:0 1px 3px rgba(0,0,0,0.05); transition:background 0.2s, border-color 0.2s;">
            <div class="container" style="display:flex; align-items:center; justify-content:space-between; height:64px;">
                <a href="/" style="display:flex; align-items:center; gap:8px; font-weight:800; font-size:17px; color:#16181d; letter-spacing:-0.02em; flex-shrink:0;">
                    if !logo_url_val.is_empty() {
                        <img src=(logo_url_val.clone()) alt="logo" style="width:28px; height:28px; border-radius:6px; object-fit:contain;">
                    } else {
                        <img src="/logo.svg" alt="Rust logo" width="28" height="28" style="object-fit:contain;">
                    }
                    <span>(site_title_val)</span>
                </a>

                <nav aria-label=(loc.pick("상단 메뉴", "Main menu")) class="desktop-nav" style="display:flex; align-items:center; gap:4px;">
                    for item in &nav_items {
                        <a href=(item.href.clone()) aria-current=(if crate::services::navigation::is_active(&item.href, &current_path) { "page" } else { "false" }) style="padding:8px 16px; font-size:15px; color:#374151; font-weight:400; border-bottom:2px solid transparent;">(label(item))</a>
                    }
                </nav>

                <div class="desktop-actions" style="display:flex; align-items:center; gap:10px;">
                    <button onclick="toggleTheme()" style="background:none; border:1px solid #E4E4E7; border-radius:8px; padding:6px 8px; cursor:pointer; display:flex; align-items:center; justify-content:center; transition:border-color 0.15s; color:#6b7280;" title=(loc.pick("테마 전환", "Toggle theme"))>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
                    </button>
                    <a href=(switch_href.clone()) hreflang=(loc.other().as_str()) title=(locale_title) style="font-size:12px; font-weight:600; color:#6b7280; border:1px solid #E4E4E7; border-radius:8px; padding:5px 10px; cursor:pointer; transition:color 0.15s;">(locale_label)</a>
                    if is_admin {
                        <a href="/admin" style="font-size:13px; color:#6b7280; font-weight:500; transition:color 0.15s;">(loc.pick("관리자", "Admin"))</a>
                    }
                    if let Some(u) = &user {
                        <a href="/notifications" style="position:relative; display:flex; align-items:center; padding:6px; color:#6b7280; border-radius:8px; transition:color 0.15s;">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
                            <span style=(notif_badge.clone())>(format!("{notif_count}"))</span>
                        </a>
                        <a href=(format!("/users/{}", u.username)) style="display:flex; align-items:center; gap:8px;">
                            if let Some(avatar) = &u.avatar_url {
                                <img src=(avatar.clone()) alt="" style="width:28px; height:28px; border-radius:50%;">
                            }
                            <span style="font-size:13px; color:#374151; font-weight:500;">(u.username.clone())</span>
                        </a>
                        <form action="/auth/logout" method="POST" style="margin:0; display:flex;">
                            <input type="hidden" name="csrf_token" value=(logout_csrf.clone())>
                            <button type="submit" style="background:none; border:none; padding:0; font:inherit; font-size:13px; color:#6b7280; cursor:pointer; transition:color 0.15s;">(loc.pick("로그아웃", "Log out"))</button>
                        </form>
                    } else {
                        <a href="/auth/login" style="font-size:14px; color:#6b7280; font-weight:500; transition:color 0.15s;">(loc.pick("로그인", "Log in"))</a>
                        <a href="/auth/register" style="background:#2563EB; color:#fff; padding:8px 16px; border-radius:8px; font-size:14px; font-weight:600; transition:background 0.15s;">(loc.pick("회원가입", "Sign up"))</a>
                    }
                </div>

                <div class="mobile-actions">
                    <button type="button" aria-label=(loc.pick("전체 메뉴 열기", "Open menu")) aria-haspopup="dialog" aria-controls="mobile-menu" onclick="document.getElementById('mobile-menu').showModal()" style="background:none; border:1px solid #E4E4E7; border-radius:8px; padding:6px; color:#6b7280; cursor:pointer;">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M3 12h18M3 18h18"/></svg>
                    </button>
                    <a href="/search" style="display:flex; align-items:center; padding:6px; color:#6b7280;">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
                    </a>
                    <button onclick="toggleTheme()" style="background:none; border:1px solid #E4E4E7; border-radius:8px; padding:5px 7px; cursor:pointer; display:flex; align-items:center; justify-content:center; color:#6b7280;" title=(loc.pick("테마 전환", "Toggle theme"))>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
                    </button>
                    if is_logged_in {
                        <a href="/notifications" style="position:relative; display:flex; align-items:center; padding:6px; color:#6b7280;">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
                            <span style=(notif_badge.clone())>(format!("{notif_count}"))</span>
                        </a>
                    }
                </div>
            </div>
        </header>

        <dialog id="mobile-menu" class="mobile-menu-dialog" aria-label=(loc.pick("전체 메뉴", "Menu")) onclick="if(event.target===this){const r=this.getBoundingClientRect();if(event.clientX<r.left||event.clientX>r.right||event.clientY<r.top||event.clientY>r.bottom)this.close()}">
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:16px;">
                <strong>(loc.pick("전체 메뉴", "Menu"))</strong>
                <button type="button" autofocus="true" aria-label=(loc.pick("전체 메뉴 닫기", "Close menu")) onclick="document.getElementById('mobile-menu').close()">(loc.pick("닫기", "Close"))</button>
            </div>
            <nav aria-label=(loc.pick("모바일 전체 메뉴", "Mobile menu")) style="display:flex; flex-direction:column;">
                for item in &nav_items {
                    <a href=(item.href.clone()) aria-current=(if crate::services::navigation::is_active(&item.href, &current_path) { "page" } else { "false" }) onclick="document.getElementById('mobile-menu').close()">(label(item))</a>
                }
                <a href=(switch_href.clone()) hreflang=(loc.other().as_str()) style="border-top:1px solid #e5e7eb; margin-top:8px;">(loc.pick("English", "한국어"))</a>
            </nav>
        </dialog>

        <nav class="bottom-tab-bar" aria-label=(loc.pick("모바일 하단 탭", "Mobile tabs"))>
            <div class="bottom-tab-scroll">
                for item in &mobile_items {
                    let active = crate::services::navigation::is_active(&item.href, &current_path);
                    let icon = match item.href.as_str() {
                        "/" => "M3 10L12 3L21 10V21H15V14H9V21H3Z",
                        "/manuals" => "M4 19V4Q4 2 7 2H20V22H7Q4 22 4 19ZM4 17H20",
                        "/community" => "M21 15Q21 17 19 17H7L3 21V5Q3 3 5 3H19Q21 3 21 5Z",
                        "/search" => "M21 21L16 16M18 10A8 8 0 1 1 2 10A8 8 0 1 1 18 10",
                        _ => "M5 3H15L19 7V21H5ZM9 11H15M9 15H15",
                    };
                    <a href=(item.href.clone()) class="bottom-tab" aria-current=(if active { "page" } else { "false" }) style=(if active { tab_active_style } else { tab_inactive_style })>
                        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d=(icon)/></svg>
                        <span>(label(item))</span>
                    </a>
                }
            </div>
            if let Some(u) = &user {
                <a href=(format!("/users/{}", u.username)) class="bottom-tab account-tab" style=(format!("position:relative; {}", if tab_profile_active { tab_active_style } else { tab_inactive_style }))>
                    if let Some(avatar) = &u.avatar_url {
                        <img src=(avatar.clone()) alt="" style="width:22px; height:22px; border-radius:50%;">
                    } else {
                        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                    }
                    <span>(loc.pick("프로필", "Profile"))</span>
                    if notif_count > 0 {
                        <span style=(tab_notif_style)>(format!("{notif_count}"))</span>
                    }
                </a>
            } else {
                <a href="/auth/login" class="bottom-tab account-tab" style=(if tab_profile_active { tab_active_style } else { tab_inactive_style })>
                    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
                    <span>(loc.pick("로그인", "Log in"))</span>
                </a>
            }
        </nav>
    })
}
