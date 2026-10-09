use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

#[component]
pub async fn Footer(cx: &Cx) -> Result<impl View + '_> {
    let db: &crate::db::Db = topcoat::context::app_context(cx);
    let site_title = crate::services::site_settings::get_setting(db, "site_title").await;
    let site_title = if site_title.is_empty() {
        "Rust Korea Hub".to_string()
    } else {
        site_title
    };
    let site_description =
        crate::services::site_settings::get_setting(db, "site_description").await;
    let site_description = if site_description.is_empty() {
        "한국 Rust 생태계의 단일 진실 공급원".to_string()
    } else {
        site_description
    };
    let footer_text = crate::services::site_settings::get_setting(db, "footer_text").await;
    let footer_text = if footer_text.is_empty() {
        "2026 Rust Korea. Built with Rust and Topcoat.".to_string()
    } else {
        footer_text
    };
    let logo_url = crate::services::site_settings::get_setting(db, "logo_url").await;

    Ok(view! {
        <footer class="desktop-footer" style="border-top:1px solid #e5e7eb; margin-top:80px; background:#f8f9fa;">
            <div class="container" style="padding:48px 24px;">
                <div class="mobile-footer-links" style="display:grid; grid-template-columns:1.5fr 1fr 1fr 1fr; gap:40px; margin-bottom:40px;">
                    <div>
                        <div style="display:flex; align-items:center; gap:8px; font-weight:800; font-size:15px; margin-bottom:10px; color:#16181d;">
                            if !logo_url.is_empty() {
                                <img src=(logo_url.clone()) alt="logo" style="width:24px; height:24px; border-radius:4px; object-fit:contain;">
                            } else {
                                <svg width="18" height="18" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
                                    <path d="M75.2 0l-.2 81.5c0 24.3-18.6 44.4-42.8 46.2L11.6 129v10.5h40.2l2.1-9.4c.4 0 .6.1.8.1 24.8 0 44.9 20.1 44.9 44.9v51.3L96.1 256h10.4l.2-20.2V175.4c0-24.8 20.1-44.9 44.9-44.9 1 0 2.1 0 3.1.1l2.3 10.4h43.2v-10.5l-22.1-1.3c-23.7-1.8-42.1-21.6-42.1-45.5L134 0H75.2z" fill="#DE5B30"/>
                                    <path d="M75.2 157.3l-.1 46.8c0 13.5 10.5 24.8 24 25.6l.3.1V256h40v-26.5l.3-.1c13.5-.8 24-12.1 24-25.6v-47.4c0-13.5-10.5-24.8-24-25.6l-.3-.1v-8.6c0-13.5-10.5-24.8-24-25.6l-.3-.1v-8.6c0-13.5-10.5-24.8-24-25.6l-.3-.1v-8.7H93l-.3 8.7c-13.5.8-24 12.1-24 25.6v8.6l-.3.1c-13.5.8-24 12.1-24 25.6v8.6l-.3.1c-13.5.8-24 12.1-24 25.6l.3 8.3h39.7v26.6l.3.1c13.5.8 24 12.1 24 25.6z" fill="#fff"/>
                                </svg>
                            }
                            (site_title.clone())
                        </div>
                        <p style="font-size:13px; color:#6b7280; line-height:1.6;">(site_description.clone())</p>
                    </div>
                    <div>
                        <h4 style="font-size:12px; font-weight:600; color:#9ca3af; text-transform:uppercase; letter-spacing:0.05em; margin-bottom:14px;">"서비스"</h4>
                        <ul style="list-style:none; display:flex; flex-direction:column; gap:10px;">
                            <li><a href="/manuals" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"매뉴얼"</a></li>
                            <li><a href="/youtube" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"영상"</a></li>
                            <li><a href="/community" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"커뮤니티"</a></li>
                            <li><a href="/search" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"검색"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h4 style="font-size:12px; font-weight:600; color:#9ca3af; text-transform:uppercase; letter-spacing:0.05em; margin-bottom:14px;">"링크"</h4>
                        <ul style="list-style:none; display:flex; flex-direction:column; gap:10px;">
                            <li><a href="https://www.rust-lang.org" style="font-size:14px; color:#6b7280; transition:color 0.15s;" target="_blank" rel="noopener">"Rust 공식"</a></li>
                            <li><a href="https://github.com/rust-lang" style="font-size:14px; color:#6b7280; transition:color 0.15s;" target="_blank" rel="noopener">"GitHub"</a></li>
                            <li><a href="https://docs.rs/topcoat" style="font-size:14px; color:#6b7280; transition:color 0.15s;" target="_blank" rel="noopener">"Topcoat Docs"</a></li>
                        </ul>
                    </div>
                    <div>
                        <h4 style="font-size:12px; font-weight:600; color:#9ca3af; text-transform:uppercase; letter-spacing:0.05em; margin-bottom:14px;">"참여"</h4>
                        <ul style="list-style:none; display:flex; flex-direction:column; gap:10px;">
                            <li><a href="/community/new" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"글쓰기"</a></li>
                            <li><a href="/admin" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"관리자"</a></li>
                            <li><a href="/privacy" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"개인정보처리방침"</a></li>
                            <li><a href="/terms" style="font-size:14px; color:#6b7280; transition:color 0.15s;">"이용약관"</a></li>
                        </ul>
                    </div>
                </div>
                <div class="divider" style="margin-bottom:24px;"></div>
                <p style="font-size:13px; color:#9ca3af; text-align:center;">(footer_text.clone())</p>
            </div>
        </footer>

        <footer class="mobile-footer" style="display:none; border-top:1px solid #e5e7eb; margin-top:24px; padding:24px 16px 16px; background:#f8f9fa;">
            <div style="display:flex; flex-wrap:wrap; gap:16px 24px; margin-bottom:16px;">
                <a href="/manuals" style="font-size:13px; color:#6b7280; text-decoration:none;">"매뉴얼"</a>
                <a href="/youtube" style="font-size:13px; color:#6b7280; text-decoration:none;">"영상"</a>
                <a href="/community" style="font-size:13px; color:#6b7280; text-decoration:none;">"커뮤니티"</a>
                <a href="/search" style="font-size:13px; color:#6b7280; text-decoration:none;">"검색"</a>
                <a href="/admin" style="font-size:13px; color:#6b7280; text-decoration:none;">"관리자"</a>
                <a href="/privacy" style="font-size:13px; color:#6b7280; text-decoration:none;">"개인정보처리방침"</a>
                <a href="/terms" style="font-size:13px; color:#6b7280; text-decoration:none;">"이용약관"</a>
            </div>
            <p style="font-size:12px; color:#9ca3af; text-align:center;">(footer_text)</p>
        </footer>
    })
}
