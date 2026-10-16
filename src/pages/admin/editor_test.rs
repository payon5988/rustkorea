use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::admin_ui::ADMIN_STYLE;
use crate::components::admin_sidebar::AdminSidebar;

/// Editor diagnostics: loads the same Toast assets as the writing forms and
/// reports each init step as plain text (no devtools needed).
#[page("/admin/editor-test")]
pub async fn editor_test(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, crate::auth::access::perms::SETTINGS_MANAGE)
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"에디터 진단 - Rust Korea Admin"</title>
            <link rel="stylesheet" href=(crate::admin_ui::TOAST_CSS_URL)>
            <script src=(crate::admin_ui::TOAST_JS_URL)></script>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/blog".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700;">"에디터 진단"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"아래 4줄 결과를 그대로 알려주세요."</p>
                </div>
                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; max-width:720px; margin-bottom:16px;">
                    <ol id="diag-list" style="font-size:14px; color:var(--text2); line-height:2; padding-left:20px;">
                        <li>"진단 중..."</li>
                    </ol>
                </div>
                <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; max-width:720px;">
                    <p style="font-size:12px; color:var(--muted2); margin-bottom:12px;">"테스트 에디터 (아래에 툴바가 보여야 정상)"</p>
                    <div id="diag-box"></div>
                </div>
                <script>
                    {r#"function diagLine(t){var ul=document.getElementById('diag-list');if(!ul){return;}while(ul.firstChild){ul.removeChild(ul.firstChild);}var li=document.createElement('li');li.textContent=t;ul.appendChild(li);}function diagAdd(t){var ul=document.getElementById('diag-list');if(!ul){return;}var li=document.createElement('li');li.textContent=t;ul.appendChild(li);}try{var tags=document.querySelectorAll('script[src*="toast"]');var s1='NO';if(tags.length){s1='YES';}diagLine('1. script tag: '+s1);var hasG='NO';if(window.toastui&&window.toastui.Editor){hasG='YES';}diagAdd('2. toastui.Editor: '+hasG);if(hasG==='YES'){var box=document.getElementById('diag-box');var ed=new window.toastui.Editor({el:box,height:'200px',initialEditType:'wysiwyg',initialValue:'test'});var n=box.querySelectorAll('button').length;diagAdd('3. constructor: OK');diagAdd('4. toolbar buttons: '+n);}else{diagAdd('3. constructor: SKIP');diagAdd('4. toolbar buttons: SKIP');}}catch(e){var msg='?';if(e&&e.message){msg=e.message;}diagAdd('ERROR: '+String(msg).slice(0,120));}"#}
                </script>
            </div>
        </body>
        </html>
    })
}
