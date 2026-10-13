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
pub struct SettingsForm {
    pub csrf_token: String,
    pub site_title: String,
    pub site_description: String,
    pub footer_text: String,
    pub youtube_channel_id: String,
    pub youtube_visibility: String,
    pub posts_per_page: String,
    pub points_post: String,
    pub points_comment: String,
    pub points_like: String,
    pub site_keywords: String,
    pub logo_url: String,
    pub github_oauth_enabled: Option<String>,
    pub google_oauth_enabled: Option<String>,
    pub adsense_enabled: Option<String>,
    pub adsense_client: String,
    pub require_approval: Option<String>,
}

async fn get_setting(db: &Db, key: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM site_settings WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

async fn save_setting(db: &Db, key: &str, value: &str) {
    sqlx::query(
        "INSERT INTO site_settings (key, value, updated_at)
         VALUES ($1, $2, NOW()::text)
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()::text",
    )
    .bind(key)
    .bind(value)
    .execute(db)
    .await
    .ok();
}

const FIELD_LABEL_STYLE: &str =
    "display:block; font-size:12px; font-weight:600; color:var(--muted); margin-bottom:6px;";

#[page("/admin/settings")]
pub async fn admin_settings_page(cx: &Cx) -> Result<impl View + '_> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "settings.manage")
            .await
            .is_none()
    };
    if __denied {
        return Err(topcoat::router::error::redirect("/admin/login").into());
    }

    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;

    let site_title = get_setting(db, "site_title").await;
    let site_description = get_setting(db, "site_description").await;
    let footer_text = get_setting(db, "footer_text").await;
    let youtube_channel_id = get_setting(db, "youtube_channel_id").await;
    let posts_per_page = get_setting(db, "posts_per_page").await;
    let points_post = get_setting(db, "points_post").await;
    let points_post = if points_post.is_empty() {
        "10".to_string()
    } else {
        points_post
    };
    let points_comment = get_setting(db, "points_comment").await;
    let points_comment = if points_comment.is_empty() {
        "2".to_string()
    } else {
        points_comment
    };
    let points_like = get_setting(db, "points_like").await;
    let points_like = if points_like.is_empty() {
        "1".to_string()
    } else {
        points_like
    };
    let site_keywords = get_setting(db, "site_keywords").await;
    let logo_url = get_setting(db, "logo_url").await;
    let github_oauth = get_setting(db, "github_oauth_enabled").await;
    let google_oauth = get_setting(db, "google_oauth_enabled").await;
    let github_checked = github_oauth == "true";
    let google_checked = google_oauth == "true";
    let adsense_enabled = get_setting(db, "adsense_enabled").await;
    let adsense_client = get_setting(db, "adsense_client").await;
    let adsense_checked = adsense_enabled == "true";
    let require_approval = get_setting(db, "require_approval").await == "true";
    let youtube_visibility = get_setting(db, "youtube_visibility").await;
    let youtube_visibility = if youtube_visibility.is_empty() {
        "public".to_string()
    } else {
        youtube_visibility
    };
    let yt_public = youtube_visibility == "public";
    let yt_members = youtube_visibility == "members";
    let yt_staff = youtube_visibility == "staff";
    let yt_premium = youtube_visibility == "premium";

    let query_str = uri(cx).query().unwrap_or("");
    let input_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none;";
    let textarea_style = "width:100%; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:10px 14px; font-size:14px; color:var(--text2); outline:none; resize:vertical; font-family:inherit; line-height:1.6;";
    let preview_box_style = if logo_url.is_empty() {
        "margin-top:12px; display:none; align-items:center; gap:16px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:16px;"
    } else {
        "margin-top:12px; display:flex; align-items:center; gap:16px; background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:16px;"
    };

    Ok(view! {
        <html lang="ko">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>"사이트 설정 - Rust Korea Admin"</title>
            <style>{(ADMIN_STYLE)}</style>
            <script>{(crate::admin_ui::ADMIN_THEME_JS)}</script>
        </head>
        <body style="display:flex; min-height:100vh;">
            {AdminSidebar(active: "/admin/settings".into())}
            <div style="flex:1; padding:24px 32px; overflow-y:auto;">
                <div style="margin-bottom:24px;">
                    <h1 style="font-size:1.25rem; font-weight:700; color:var(--text); letter-spacing:-0.01em;">"사이트 설정"</h1>
                    <p style="font-size:13px; color:var(--muted2); margin-top:2px;">"사이트 전역에서 사용하는 기본 정보를 설정합니다."</p>
                </div>

                if query_str.contains("saved=1") {
                    <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:var(--green);">"설정이 저장되었습니다."</div>
                }
                if query_str.contains("error=invalid") {
                    <div style="background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px 16px; margin-bottom:16px; font-size:13px; color:var(--red-soft);">"페이지당 게시글 수는 1 이상의 숫자여야 합니다."</div>
                }

                <form action="/admin/settings" method="POST" style="max-width:720px;">
                    <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"기본 정보"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"사이트 이름과 소개글"</p>

                        <div style="margin-bottom:20px;">
                            <label for="site_title" style=(FIELD_LABEL_STYLE)>"사이트 제목"</label>
                            <input type="text" id="site_title" name="site_title" value=(site_title) placeholder="Rust Korea Hub" style=(input_style)>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label for="site_description" style=(FIELD_LABEL_STYLE)>"사이트 설명"</label>
                            <textarea id="site_description" name="site_description" rows="3" placeholder="한국 Rust 생태계의 단일 진실 공급원" style=(textarea_style)>(site_description)</textarea>
                        </div>
                        <div style="margin-bottom:20px;">
                            <label for="site_keywords" style=(FIELD_LABEL_STYLE)>"사이트 메타 키워드"</label>
                            <input type="text" id="site_keywords" name="site_keywords" value=(site_keywords) placeholder="rust, 러스트, 커뮤니티" style=(input_style)>
                        </div>
                        <div>
                            <label for="logo_url" style=(FIELD_LABEL_STYLE)>"로고 이미지 URL"</label>
                            <input type="text" id="logo_url" name="logo_url" value=(logo_url.clone()) placeholder="https://example.com/logo.png (비워두면 기본 Rust 로고)" style=(input_style) oninput="updateLogoPreview(this.value)">
                            <p style="font-size:11px; color:var(--dim); margin-top:4px;">"PNG, JPG, SVG, WebP 등. 비워두면 Rust 기어 로고가 표시됩니다."</p>
                            <div id="logo-preview-box" style=(preview_box_style)>
                                <div style="flex-shrink:0;">
                                    <img id="logo-preview-img" src=(logo_url.clone()) alt="logo" style="width:48px; height:48px; border-radius:8px; object-fit:contain; background:#fff; padding:4px;" onerror="this.parentElement.innerHTML='<div style=\\'width:48px;height:48px;border-radius:8px;background:var(--card);border:1px solid #ef4444;display:flex;align-items:center;justify-content:center;color:var(--red);font-size:11px;font-weight:600;\\'>에러</div>'">
                                </div>
                                <div>
                                    <div style="font-size:13px; font-weight:600; color:var(--text2);">"로고 미리보기"</div>
                                    <div style="font-size:11px; color:var(--muted2); margin-top:2px;">"헤더에서 48x48로 표시됩니다"</div>
                                </div>
                            </div>
                            <script type="text/javascript">
                                {r#"function updateLogoPreview(url) {
                                    var box = document.getElementById('logo-preview-box');
                                    var img = document.getElementById('logo-preview-img');
                                    if (url.trim()) {
                                        box.style.display = 'flex';
                                        img.src = url;
                                    } else {
                                        box.style.display = 'none';
                                    }
                                }"#}
                            </script>
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"표시 설정"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"푸터 문구와 목록 표시 개수"</p>

                        <div style="margin-bottom:20px;">
                            <label for="footer_text" style=(FIELD_LABEL_STYLE)>"푸터 문구"</label>
                            <textarea id="footer_text" name="footer_text" rows="2" placeholder="(c) Rust Korea. All rights reserved." style=(textarea_style)>(footer_text)</textarea>
                        </div>
                        <div>
                            <label for="posts_per_page" style=(FIELD_LABEL_STYLE)>"페이지당 게시글 수"</label>
                            <input type="number" id="posts_per_page" name="posts_per_page" min="1" value=(posts_per_page) placeholder="20" style=(input_style)>
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"포인트"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"행동별 지급 포인트 (0 이상)"</p>

                        <div style="display:grid; grid-template-columns:repeat(3, 1fr); gap:12px;">
                            <div>
                                <label for="points_post" style=(FIELD_LABEL_STYLE)>"게시글 작성"</label>
                                <input type="number" id="points_post" name="points_post" min="0" value=(points_post) style=(input_style)>
                            </div>
                            <div>
                                <label for="points_comment" style=(FIELD_LABEL_STYLE)>"댓글 작성"</label>
                                <input type="number" id="points_comment" name="points_comment" min="0" value=(points_comment) style=(input_style)>
                            </div>
                            <div>
                                <label for="points_like" style=(FIELD_LABEL_STYLE)>"좋아요"</label>
                                <input type="number" id="points_like" name="points_like" min="0" value=(points_like) style=(input_style)>
                            </div>
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"외부 연동"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"YouTube 영상을 가져올 채널"</p>

                        <div style="margin-bottom:20px;">
                            <label for="youtube_channel_id" style=(FIELD_LABEL_STYLE)>"YouTube 채널 ID"</label>
                            <input type="text" id="youtube_channel_id" name="youtube_channel_id" value=(youtube_channel_id) placeholder="UCxxxxxxxxxxxxxxxxxxxxxx" style=(input_style)>
                        </div>
                        <div>
                            <label for="youtube_visibility" style=(FIELD_LABEL_STYLE)>"영상 공개 범위"</label>
                            <select id="youtube_visibility" name="youtube_visibility" style=(input_style)>
                                <option value="public" selected=(yt_public)>"전체공개"</option>
                                <option value="members" selected=(yt_members)>"회원전용"</option>
                                <option value="staff" selected=(yt_staff)>"스태프전용"</option>
                                <option value="premium" selected=(yt_premium)>"유료전용(준비중)"</option>
                            </select>
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"접근 제어"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"회원가입 승인제와 공개 범위를 설정합니다."</p>

                        <label style="display:flex; align-items:center; gap:12px; cursor:pointer;">
                            if require_approval {
                                <input type="checkbox" name="require_approval" value="true" checked="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                            } else {
                                <input type="checkbox" name="require_approval" value="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                            }
                            <div>
                                <div style="font-size:14px; font-weight:600; color:var(--text2);">"가입 승인제"</div>
                                <div style="font-size:12px; color:var(--muted2);">"켜면 신규 가입이 승인대기 상태가 되어 관리자가 승인해야 로그인할 수 있습니다."</div>
                            </div>
                        </label>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"Google AdSense"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"사이트에 광고를 게재하기 위한 설정입니다. Publisher ID는 Google AdSense 대시보드에서 확인할 수 있습니다."</p>

                        <div style="display:flex; flex-direction:column; gap:16px;">
                            <label style="display:flex; align-items:center; gap:12px; cursor:pointer;">
                                if adsense_checked {
                                    <input type="checkbox" name="adsense_enabled" value="true" checked="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                } else {
                                    <input type="checkbox" name="adsense_enabled" value="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                }
                                <div>
                                    <div style="font-size:14px; font-weight:600; color:var(--text2);">"AdSense 활성화"</div>
                                    <div style="font-size:12px; color:var(--muted2);">"활성화하면 페이지에 광고 슬롯이 표시됩니다."</div>
                                </div>
                            </label>
                        </div>

                        <div style="margin-top:16px;">
                            <label for="adsense_client" style=(FIELD_LABEL_STYLE)>"Publisher ID (ca-pub-xxxxxxxxxx)"</label>
                            <input type="text" id="adsense_client" name="adsense_client" value=(adsense_client) placeholder="ca-pub-XXXXXXXXXXXXXXXX" style=(input_style)>
                            <p style="font-size:11px; color:var(--dim); margin-top:4px;">"Google AdSense 대시보드 → 사이트 → 광고 설정 에서 확인할 수 있습니다."</p>
                        </div>
                    </div>

                    <div style="background:var(--card); border:1px solid var(--border); border-radius:10px; padding:24px; margin-bottom:20px;">
                        <h2 style="font-size:15px; font-weight:700; color:var(--text); margin-bottom:4px;">"OAuth 로그인"</h2>
                        <p style="font-size:12px; color:var(--muted2); margin-bottom:20px;">"소셜 로그인 제공자를 활성화 또는 비활성화합니다."</p>

                        <div style="display:flex; flex-direction:column; gap:16px;">
                            <label style="display:flex; align-items:center; gap:12px; cursor:pointer;">
                                if github_checked {
                                    <input type="checkbox" name="github_oauth_enabled" value="true" checked="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                } else {
                                    <input type="checkbox" name="github_oauth_enabled" value="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                }
                                <div>
                                    <div style="font-size:14px; font-weight:600; color:var(--text2);">"GitHub 로그인"</div>
                                    <div style="font-size:12px; color:var(--muted2);">"GitHub OAuth를 통한 소셜 로그인"</div>
                                </div>
                            </label>
                            <label style="display:flex; align-items:center; gap:12px; cursor:pointer;">
                                if google_checked {
                                    <input type="checkbox" name="google_oauth_enabled" value="true" checked="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                } else {
                                    <input type="checkbox" name="google_oauth_enabled" value="true" style="width:18px; height:18px; accent-color:#DE5B30; cursor:pointer;">
                                }
                                <div>
                                    <div style="font-size:14px; font-weight:600; color:var(--text2);">"Google 로그인"</div>
                                    <div style="font-size:12px; color:var(--muted2);">"Google OAuth를 통한 소셜 로그인"</div>
                                </div>
                            </label>
                        </div>
                    </div>

                    <button type="submit" style="background:#DE5B30; color:#fff; padding:12px 28px; border-radius:8px; font-size:14px; font-weight:600; border:none; cursor:pointer;">"설정 저장"</button>
                </form>
            </div>
        </body>
        </html>
    })
}

#[route(POST "/admin/settings")]
pub async fn admin_save_settings(
    cx: &Cx,
    Form(input): Form<SettingsForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let __denied = {
        let __adb: &crate::db::Db = topcoat::context::app_context(cx);
        crate::auth::access::require_perm(__adb, cx, "settings.manage")
            .await
            .is_none()
    };
    if __denied {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    let posts_per_page = input.posts_per_page.trim().to_string();
    match posts_per_page.parse::<i64>() {
        Ok(n) if n >= 1 => {}
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/admin/settings?error=invalid",
            ));
        }
    }

    let points_post: i64 = match input.points_post.trim().parse() {
        Ok(n) if n >= 0 => n,
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/admin/settings?error=invalid",
            ));
        }
    };
    let points_comment: i64 = match input.points_comment.trim().parse() {
        Ok(n) if n >= 0 => n,
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/admin/settings?error=invalid",
            ));
        }
    };
    let points_like: i64 = match input.points_like.trim().parse() {
        Ok(n) if n >= 0 => n,
        _ => {
            return Ok(topcoat::router::error::see_other(
                "/admin/settings?error=invalid",
            ));
        }
    };

    let db: &Db = topcoat::context::app_context(cx);

    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "admin.settings").await
    {
        return Ok(topcoat::router::error::see_other("/admin/login"));
    }

    save_setting(db, "site_title", input.site_title.trim()).await;
    save_setting(db, "site_description", input.site_description.trim()).await;
    save_setting(db, "footer_text", input.footer_text.trim()).await;
    save_setting(db, "youtube_channel_id", input.youtube_channel_id.trim()).await;
    let yt_vis = input.youtube_visibility.trim();
    save_setting(
        db,
        "youtube_visibility",
        if ["public", "members", "staff", "premium"].contains(&yt_vis) {
            yt_vis
        } else {
            "public"
        },
    )
    .await;
    save_setting(
        db,
        "require_approval",
        if input.require_approval.is_some() {
            "true"
        } else {
            "false"
        },
    )
    .await;
    save_setting(db, "posts_per_page", &posts_per_page).await;
    save_setting(db, "points_post", &points_post.to_string()).await;
    save_setting(db, "points_comment", &points_comment.to_string()).await;
    save_setting(db, "points_like", &points_like.to_string()).await;
    save_setting(db, "site_keywords", input.site_keywords.trim()).await;
    save_setting(db, "logo_url", input.logo_url.trim()).await;
    save_setting(
        db,
        "github_oauth_enabled",
        if input.github_oauth_enabled.is_some() {
            "true"
        } else {
            "false"
        },
    )
    .await;
    save_setting(
        db,
        "google_oauth_enabled",
        if input.google_oauth_enabled.is_some() {
            "true"
        } else {
            "false"
        },
    )
    .await;
    save_setting(
        db,
        "adsense_enabled",
        if input.adsense_enabled.is_some() {
            "true"
        } else {
            "false"
        },
    )
    .await;
    save_setting(db, "adsense_client", input.adsense_client.trim()).await;

    let actor = crate::auth::access::load_access(db, cx)
        .await
        .map(|a| a.user.id)
        .unwrap_or_default();
    crate::auth::access::audit(db, cx, &actor, "admin.settings.save", "settings", "", "").await;

    Ok(topcoat::router::error::see_other("/admin/settings?saved=1"))
}
