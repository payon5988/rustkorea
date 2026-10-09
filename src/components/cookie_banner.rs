use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookies, cookies},
    view::{View, component, view},
};

#[component]
pub async fn CookieBanner(cx: &Cx) -> Result<impl View + '_> {
    let has_consent = cookies(cx)
        .get("cookie_consent")
        .map(|c| c.value() == "1")
        .unwrap_or(false);

    Ok(view! {
        if !has_consent {
            <div id="cookie-banner" style="position:fixed; bottom:0; left:0; right:0; z-index:100; background:#1a1d27; border-top:1px solid #2a2d3a; padding:16px 24px; display:flex; align-items:center; justify-content:space-between; gap:16px; flex-wrap:wrap; box-shadow:0 -4px 20px rgba(0,0,0,0.15);">
                <p style="font-size:13px; color:#d1d5db; line-height:1.6; flex:1; min-width:280px; margin:0;">
                    "이 사이트는 서비스 개선과 광고 게재를 위해 쿠키를 사용합니다. 계속 사용하시면 쿠키 사용에 동의하시는 것으로 간주됩니다. 자세한 내용은 "
                    <a href="/privacy" style="color:#60a5fa; text-decoration:underline;">"개인정보처리방침"</a>
                    "을 참조하세요."
                </p>
                <div style="display:flex; gap:8px; flex-shrink:0;">
                    <button onclick="acceptCookies()" style="background:#2563EB; color:#fff; padding:8px 20px; border-radius:8px; font-size:13px; font-weight:600; border:none; cursor:pointer; transition:background 0.15s;">
                        "동의"
                    </button>
                    <button onclick="document.getElementById('cookie-banner').style.display='none'" style="background:transparent; color:#9ca3af; padding:8px 16px; border-radius:8px; font-size:13px; font-weight:500; border:1px solid #3a3d4a; cursor:pointer; transition:color 0.15s;">
                        "닫기"
                    </button>
                </div>
            </div>
            <script>
                {r#"function acceptCookies(){document.cookie='cookie_consent=1;path=/;max-age=31536000;SameSite=Lax'+(location.protocol==='https:'?';Secure':'');var b=document.getElementById('cookie-banner');if(b)b.style.display='none';}"#}
            </script>
        }
    })
}
