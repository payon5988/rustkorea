use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::db::Db;

#[component]
pub async fn AdminSidebar(cx: &Cx, active: String) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let logo_url = crate::services::site_settings::get_setting(db, "logo_url").await;

    let nav_item = |href: &str, active: &str| -> String {
        let is_active = active == href || (href != "/admin" && active.starts_with(href));
        if is_active {
            "display:flex; align-items:center; gap:10px; padding:9px 12px; font-size:13px; color:var(--text); background:rgba(222,91,48,0.12); border-radius:8px; font-weight:600; transition:all 0.15s;".to_string()
        } else {
            "display:flex; align-items:center; gap:10px; padding:9px 12px; font-size:13px; color:var(--muted); border-radius:8px; transition:all 0.15s;".to_string()
        }
    };

    let d = nav_item("/admin", &active);
    let d_manuals = nav_item("/admin/manuals", &active);
    let d_manuals_new = nav_item("/admin/manuals/new", &active);
    let d_chapters_new = nav_item("/admin/chapters/new", &active);
    let d_posts = nav_item("/admin/posts", &active);
    let d_write = nav_item("/admin/posts/new", &active);
    let d_comments = nav_item("/admin/comments", &active);
    let d_videos = nav_item("/admin/videos", &active);
    let d_users = nav_item("/admin/users", &active);
    let d_roles = nav_item("/admin/roles", &active);
    let d_audit = nav_item("/admin/audit", &active);
    let d_menus = nav_item("/admin/menus", &active);
    let d_badges = nav_item("/admin/badges", &active);
    let d_products = nav_item("/admin/products", &active);
    let d_blog = nav_item("/admin/blog", &active);
    let d_inquiries = nav_item("/admin/inquiries", &active);
    let d_settings = nav_item("/admin/settings", &active);
    let d_profile = nav_item("/admin/profile", &active);
    let d_categories = nav_item("/admin/categories", &active);
    let d_reports = nav_item("/admin/reports", &active);
    let d_events = nav_item("/admin/events", &active);
    let access = crate::auth::access::load_access(db, cx).await;
    let site_nav_items =
        crate::services::navigation::visible_items(db, "header", access.as_ref()).await?;
    let can_manuals = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::MANUALS_MANAGE))
        .unwrap_or(false);
    let can_posts = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::POSTS_MODERATE))
        .unwrap_or(false);
    let can_community = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::COMMUNITY_MANAGE))
        .unwrap_or(false);
    let can_videos = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::YOUTUBE_MANAGE))
        .unwrap_or(false);
    let can_events = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::EVENTS_MANAGE))
        .unwrap_or(false);
    let can_users = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::USERS_MANAGE))
        .unwrap_or(false);
    let can_roles = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::ROLES_MANAGE))
        .unwrap_or(false);
    let can_settings = access
        .as_ref()
        .map(|a| a.can(crate::auth::access::perms::SETTINGS_MANAGE))
        .unwrap_or(false);

    Ok(view! {
        <div style="display:contents;">
            <style>
                {r#"#admin-sidebar-overlay { display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); z-index: 30; }
                    #admin-mobile-toggle { display: none; position: fixed; top: 12px; left: 12px; z-index: 32; background: var(--bg2); border: 1px solid var(--border); color: var(--text2); width: 40px; height: 40px; border-radius: 8px; cursor: pointer; align-items: center; justify-content: center; }
                    #sidebar-close-btn { display: none; }
                    .admin-mobile-warning { display: none; }
                    @media (max-width: 768px) {
                        #admin-sidebar { position: fixed !important; top: 0 !important; left: 0 !important; z-index: 31 !important; transform: translateX(-100%) !important; height: 100vh !important; width: 280px !important; }
                        #sidebar-close-btn { display: flex !important; }
                        #admin-mobile-toggle { display: flex !important; }
                        .admin-mobile-warning { display: flex !important; }
                    }"#}
            </style>
            <script>
                {r#"function toggleAdminSidebar(){var s=document.getElementById('admin-sidebar');var o=document.getElementById('admin-sidebar-overlay');if(!s)return;var open=s.dataset.open==='1';if(open){s.style.setProperty('transform','translateX(-100%)','important');s.dataset.open='0';if(o)o.style.display='none'}else{s.style.setProperty('transform','translateX(0)','important');s.dataset.open='1';if(o)o.style.display='block'}}
                    function closeAdminSidebar(){var s=document.getElementById('admin-sidebar');var o=document.getElementById('admin-sidebar-overlay');if(s){s.style.setProperty('transform','translateX(-100%)','important');s.dataset.open='0'}if(o)o.style.display='none'}"#}
            </script>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>

            <button id="admin-mobile-toggle" onclick="toggleAdminSidebar()" aria-label="관리자 메뉴 열기">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <line x1="3" y1="6" x2="21" y2="6"/>
                    <line x1="3" y1="12" x2="21" y2="12"/>
                    <line x1="3" y1="18" x2="21" y2="18"/>
                </svg>
            </button>
            <div id="admin-sidebar-overlay" onclick="closeAdminSidebar()"></div>

            <aside id="admin-sidebar" style="width:250px; background:var(--bg2); border-right:1px solid var(--border-soft); flex-shrink:0; display:flex; flex-direction:column; height:calc(100vh - 56px); position:sticky; top:56px; overflow-y:auto; transition:transform 0.2s ease;">

                <div class="admin-mobile-warning" style="background:rgba(245,158,11,0.1); border-bottom:1px solid rgba(245,158,11,0.2); padding:10px 16px; align-items:center; gap:8px; font-size:12px; color:#f59e0b; flex-shrink:0;">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" style="flex-shrink:0;">
                        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                        <line x1="12" y1="9" x2="12" y2="13"/>
                        <line x1="12" y1="17" x2="12.01" y2="17"/>
                    </svg>
                    "관리자 패널은 PC 환경에서 이용해주세요"
                </div>

                <div style="padding:20px 16px 16px; border-bottom:1px solid var(--border-soft); display:flex; align-items:center; justify-content:space-between;">
                    <a href="/admin" style="display:flex; align-items:center; gap:10px; font-weight:800; font-size:14px; color:var(--text); letter-spacing:-0.01em;">
                        <div style="width:32px; height:32px; border-radius:8px; background:rgba(222,91,48,0.12); display:flex; align-items:center; justify-content:center;">
                            if !logo_url.is_empty() {
                                <img src=(logo_url.clone()) alt="logo" style="width:24px; height:24px; border-radius:4px; object-fit:contain;">
                            } else {
                                <img src="/logo.svg" alt="Rust logo" width="16" height="16" style="object-fit:contain;">
                            }
                        </div>
                        "Admin Panel"
                    </a>
                    <button onclick="closeAdminSidebar()" style="display:none; background:none; border:none; color:var(--muted); cursor:pointer; padding:4px;" id="sidebar-close-btn">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                    </button>
                </div>

                <nav style="flex:1; padding:12px 10px;">

                    <div style="font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em; padding:8px 12px 6px;">"메인"</div>
                    <a href="/admin" style=(d)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
                        "대시보드"
                    </a>

                    <div style="font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em; padding:16px 12px 6px;">"매뉴얼"</div>
                    if can_manuals {
                    <a href="/admin/manuals" style=(d_manuals)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>
                        "매뉴얼 목록"
                    </a>
                    }
                    if can_manuals {
                    <a href="/admin/manuals/new" style=(d_manuals_new)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                        "새 매뉴얼"
                    </a>
                    }
                    if can_manuals {
                    <a href="/admin/chapters/new" style=(d_chapters_new)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                        "새 챕터"
                    </a>
                    }

                    <div style="font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em; padding:16px 12px 6px;">"커뮤니티"</div>
                    if can_posts {
                    <a href="/admin/posts" style=(d_posts)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                        "게시글 관리"
                    </a>
                    }
                    if can_posts {
                    <a href="/admin/posts/new" style=(d_write)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                        "글쓰기"
                    </a>
                    }
                    if can_posts {
                    <a href="/admin/comments" style=(d_comments)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>
                        "댓글 관리"
                    </a>
                    }
                    if can_community {
                    <a href="/admin/categories" style=(d_categories)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>
                        "카테고리 관리"
                    </a>
                    }
                    if can_community {
                    <a href="/admin/reports" style=(d_reports)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                        "신고 관리"
                    </a>
                    }

                    <div style="font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em; padding:16px 12px 6px;">"영상"</div>
                    if can_videos {
                    <a href="/admin/videos" style=(d_videos)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
                        "YouTube 영상"
                    </a>
                    }
                    if can_events {
                    <a href="/admin/events" style=(d_events)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                        "이벤트"
                    </a>
                    }

                    if !site_nav_items.is_empty() {
                        <div style="font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em; padding:16px 12px 6px;">"사이트 메뉴"</div>
                        for item in &site_nav_items {
                            <a href=(item.href.clone()) style="display:flex; align-items:center; gap:10px; padding:9px 12px; font-size:13px; color:var(--muted); border-radius:8px; transition:all 0.15s;">
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="18" x2="21" y2="18"/></svg>
                                (item.label.clone())
                            </a>
                        }
                    }

                    <div style="font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em; padding:16px 12px 6px;">"설정"</div>
                    if can_settings {
                    <a href="/admin/settings" style=(d_settings)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 1-1.51 1Z"/></svg>
                        "사이트 설정"
                    </a>
                    }
                    if can_settings {
                    <a href="/admin/menus" style=(d_menus)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="18" x2="21" y2="18"/></svg>
                        "메뉴 관리"
                    </a>
                    }
                    if can_settings {
                    <a href="/admin/badges" style=(d_badges)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="8" r="6"/><path d="M15.5 13l1.5 8-5-3-5 3 1.5-8"/></svg>
                        "등급/배지"
                    </a>
                    }
                    if can_settings {
                    <a href="/admin/products" style=(d_products)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"/><line x1="3" y1="6" x2="21" y2="6"/><path d="M16 10a4 4 0 0 1-8 0"/></svg>
                        "상품 관리"
                    </a>
                    }
                    if can_settings {
                    <a href="/admin/blog" style=(d_blog)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4z"/></svg>
                        "블로그 관리"
                    </a>
                    }
                    if can_settings {
                    <a href="/admin/inquiries" style=(d_inquiries)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                        "문의 관리"
                    </a>
                    }
                    if can_users {
                    <a href="/admin/users" style=(d_users)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
                        "사용자 관리"
                    </a>
                    }
                    if can_roles {
                    <a href="/admin/roles" style=(d_roles)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M19.1 4.9L17 7M7 17l-2.1 2.1"/></svg>
                        "역할/권한"
                    </a>
                    }
                    if can_users {
                    <a href="/admin/audit" style=(d_audit)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
                        "감사 로그"
                    </a>
                    }
                    <a href="/admin/profile" style=(d_profile)>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                        "관리자 프로필"
                    </a>

                </nav>

                <div style="padding:12px 10px; border-top:1px solid var(--border-soft);">
                    <div style="padding:2px 12px 8px; font-size:10px; font-weight:700; color:var(--dim); text-transform:uppercase; letter-spacing:0.08em;">"테마"</div>
                    <div style="display:flex; gap:4px; padding:0 12px 10px;">
                        <button type="button" class="atheme-btn" data-admin-theme-btn="system" onclick="__rkSetAdminTheme('system')">"시스템"</button>
                        <button type="button" class="atheme-btn" data-admin-theme-btn="light" onclick="__rkSetAdminTheme('light')">"라이트"</button>
                        <button type="button" class="atheme-btn" data-admin-theme-btn="dark" onclick="__rkSetAdminTheme('dark')">"다크"</button>
                    </div>
                    <a href="/" style="display:flex; align-items:center; gap:10px; padding:9px 12px; font-size:13px; color:var(--muted2); border-radius:8px; transition:all 0.15s;">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>
                        "사이트 돌아가기"
                    </a>
                    <a href="/admin/logout" style="display:flex; align-items:center; gap:10px; padding:9px 12px; font-size:13px; color:var(--muted2); border-radius:8px; transition:all 0.15s;">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
                        "로그아웃"
                    </a>
                </div>

            </aside>
        </div>
    })
}
