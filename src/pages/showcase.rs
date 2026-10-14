use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::db::Db;
use crate::models::{Product, ProductCategory};

topcoat::router::path_param!(slug);

/// Inquiry throttle: at most N per IP per window.
pub const INQUIRY_MAX_PER_WINDOW: i64 = 3;
pub const INQUIRY_WINDOW_SECS: i64 = 600;

fn product_gate(
    access: Option<&crate::auth::access::Access>,
    visibility: &str,
) -> crate::auth::access::Gate {
    crate::auth::access::visibility_gate(
        access,
        visibility,
        crate::auth::access::perms::MANUALS_VIEW_STAFF,
        crate::auth::access::perms::PREMIUM_ACCESS,
    )
}

#[page("/vibecoding")]
pub async fn showcase_index(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;

    let categories = sqlx::query_as::<_, ProductCategory>(
        "SELECT * FROM product_categories ORDER BY position ASC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let products = sqlx::query_as::<_, Product>(&format!(
        "SELECT * FROM products WHERE {} ORDER BY created_at DESC",
        crate::util::published_clause("")
    ))
    .fetch_all(db)
    .await
    .unwrap_or_default();
    // Tier gate in Rust (per-row visibility differs).
    let products: Vec<Product> = products
        .into_iter()
        .filter(|p| {
            product_gate(access.as_ref(), &p.visibility) == crate::auth::access::Gate::Allow
        })
        .collect();

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <div style="margin-bottom:40px;">
                <div style="font-size:11px; font-weight:700; color:#DE5B30; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"VIBECODING"</div>
                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">"바이브코딩 프로젝트"</h1>
                <p class="section-sub">"Rust로 만든 실제 프로덕트를 소개합니다."</p>
            </div>

            if !categories.is_empty() {
                <div style="display:flex; gap:8px; flex-wrap:wrap; margin-bottom:24px;">
                    for cat in &categories {
                        <span class="badge" style="background:#f3f4f6; color:#374151;">(cat.name.clone())</span>
                    }
                </div>
            }

            if products.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="color:#6b7280;">"소개 중인 프로젝트가 곧 공개됩니다."</p>
                </div>
            } else {
                <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(300px, 1fr)); gap:20px;">
                    for item in products {
                        <a href=(format!("/vibecoding/{}", item.slug)) class="card" style="display:block; overflow:hidden;">
                            if !item.thumbnail_url.is_empty() {
                                <img src=(item.thumbnail_url.clone()) alt=(item.title.clone()) loading="lazy" style="width:100%; height:180px; object-fit:cover; display:block;">
                            } else {
                                <div style="width:100%; height:180px; background:linear-gradient(135deg, #1f2833 0%, #25283d 100%); display:flex; align-items:center; justify-content:center; font-size:48px;">"🦀"</div>
                            }
                            <div style="padding:20px 22px;">
                                <h3 style="font-size:16px; font-weight:700; margin-bottom:6px;">(item.title.clone())</h3>
                                <p style="font-size:13px; color:#6b7280; line-height:1.5; margin-bottom:12px;">(item.summary.clone())</p>
                                <div style="display:flex; align-items:center; gap:8px; flex-wrap:wrap;">
                                    for tech in item.tech_list() {
                                        <span style="font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:#1e40af; background:#dbeafe;">(tech)</span>
                                    }
                                </div>
                                if let Some(price) = item.price_label() {
                                    <div style="font-size:16px; font-weight:800; color:#DE5B30; margin-top:12px;">(price)</div>
                                }
                            </div>
                        </a>
                    }
                </div>
            }
        </div>
    })
}

#[page("/vibecoding/{slug}")]
pub async fn showcase_detail(cx: &Cx) -> Result<impl View + '_> {
    let slug = path_param::<Slug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let session_token = crate::auth::session::csrf_token(db, cx).await;
    let csrf_token = if session_token.is_empty() {
        crate::auth::session::issue_csrf_seed(cx)
    } else {
        session_token
    };
    let access = crate::auth::access::load_access(db, cx).await;

    let product: Option<Product> = sqlx::query_as::<_, Product>(&format!(
        "SELECT * FROM products WHERE slug = $1 AND {}",
        crate::util::published_clause("")
    ))
    .bind(slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let gate = product
        .as_ref()
        .map(|p| product_gate(access.as_ref(), &p.visibility))
        .unwrap_or(crate::auth::access::Gate::Allow);
    if gate == crate::auth::access::Gate::LoginRequired {
        return Err(topcoat::router::error::redirect(format!(
            "/auth/login?redirect=/vibecoding/{slug}"
        ))
        .into());
    }
    let forbidden = gate == crate::auth::access::Gate::Forbidden;
    let product = if forbidden { None } else { product };

    let images: Vec<crate::models::ProductImage> = if let Some(ref p) = product {
        sqlx::query_as::<_, crate::models::ProductImage>(
            "SELECT * FROM product_images WHERE product_id = $1 ORDER BY position ASC",
        )
        .bind(&p.id)
        .fetch_all(db)
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };

    let category_name: String = if let Some(ref p) = product {
        sqlx::query_scalar::<_, String>("SELECT name FROM product_categories WHERE id = $1")
            .bind(&p.category_id)
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .unwrap_or_default()
    } else {
        String::new()
    };

    let sent = topcoat::router::request::uri(cx)
        .query()
        .map(|q| q.contains("sent=1"))
        .unwrap_or(false);

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:960px;">
            <style>
                {r#".show-gallery { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
                .show-gallery img { width: 100%; height: 160px; object-fit: cover; border-radius: 10px; border: 1px solid #E4E4E7; }
                @media (max-width: 640px) { .show-hero { height: 200px !important; } }"#}
            </style>
            if let Some(p) = product {
                <a href="/vibecoding" style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">
                    "<- 프로젝트 목록"
                </a>

                if !p.main_image_url.is_empty() {
                    <img src=(p.main_image_url.clone()) alt=(p.title.clone()) class="show-hero" style="width:100%; height:320px; object-fit:cover; border-radius:14px; margin-bottom:28px; display:block;">
                }

                <div style="display:flex; align-items:center; gap:10px; margin-bottom:10px; flex-wrap:wrap;">
                    if !category_name.is_empty() {
                        <span class="badge" style="background:#eff6ff; color:#2563EB;">(category_name.clone())</span>
                    }
                    for tech in p.tech_list() {
                        <span style="font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:#1e40af; background:#dbeafe;">(tech)</span>
                    }
                </div>

                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">(p.title.clone())</h1>
                <p class="section-sub" style="margin-bottom:16px;">(p.summary.clone())</p>

                <div style="display:flex; gap:12px; align-items:center; margin-bottom:32px; flex-wrap:wrap;">
                    if let Some(price) = p.price_label() {
                        <div style="font-size:1.5rem; font-weight:800; color:#DE5B30;">(price)</div>
                    }
                    if !p.preview_url.is_empty() {
                        <a href=(p.preview_url.clone()) target="_blank" rel="noopener" class="btn-primary">"미리보기"</a>
                    }
                </div>

                <div class="card" style="padding:40px 48px; margin-bottom:24px;">
                    <div class="markdown-body">
                        (topcoat::view::Unescaped::new_unchecked(p.description_html.clone()))
                    </div>
                </div>

                if !p.video_id.is_empty() {
                    <div style="margin-bottom:24px;">
                        <div class="youtube-embed"><iframe src=(format!("https://www.youtube-nocookie.com/embed/{}", p.video_id)) title="소개 영상" loading="lazy" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen="true"></iframe></div>
                    </div>
                }

                if !images.is_empty() {
                    <h2 style="font-size:1.1rem; font-weight:700; margin-bottom:12px;">"화면 미리보기"</h2>
                    <div class="show-gallery" style="margin-bottom:24px;">
                        for img in &images {
                            <img src=(img.url.clone()) alt="프로젝트 화면" loading="lazy">
                        }
                    </div>
                }

                <div class="card" style="padding:32px; margin-top:8px;">
                    <h2 style="font-size:1.1rem; font-weight:700; margin-bottom:4px;">"도입 문의"</h2>
                    <p style="font-size:13px; color:#6b7280; margin-bottom:20px;">"연락처를 남겨주시면 확인 후 연락드립니다."</p>
                    if sent {
                        <div style="background:rgba(52,211,153,0.1); border:1px solid rgba(52,211,153,0.3); border-radius:8px; padding:12px 16px; font-size:13px; color:#059669; margin-bottom:20px;">"문의가 접수되었습니다. 확인 후 연락드리겠습니다."</div>
                    }
                    <form action=(format!("/vibecoding/{}/inquire", p.slug)) method="POST" style="display:flex; flex-direction:column; gap:12px; max-width:520px;">
                        <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                        <input type="text" name="name" required="true" maxlength="40" placeholder="성함" class="input">
                        <input type="text" name="contact" required="true" maxlength="80" placeholder="연락처 (이메일 또는 전화번호)" class="input">
                        <textarea name="message" rows="4" maxlength="2000" placeholder="문의 내용" class="input"></textarea>
                        <input type="text" name="company_site" style="display:none;" tabindex="-1" autocomplete="off">
                        <button type="submit" class="btn-primary" style="justify-content:center;">"문의 보내기"</button>
                    </form>
                </div>
            } else {
                <div style="text-align:center; padding:100px 0;">
                    if forbidden {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"접근 권한이 없습니다"</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">"회원 전용 프로젝트입니다."</p>
                    } else {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">"프로젝트를 찾을 수 없습니다"</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">"요청하신 프로젝트가 존재하지 않습니다."</p>
                    }
                    <a href="/vibecoding" class="btn-primary" style="display:inline-flex;">"목록으로"</a>
                </div>
            }
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct InquiryForm {
    pub csrf_token: String,
    pub name: String,
    pub contact: String,
    pub message: String,
    pub company_site: String,
}

#[route(POST "/vibecoding/{slug}/inquire")]
pub async fn submit_inquiry(
    cx: &Cx,
    Form(input): Form<InquiryForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let slug = path_param::<Slug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let back = format!("/vibecoding/{slug}");

    // Honeypot: bots fill the hidden field.
    if !input.company_site.trim().is_empty() {
        return Ok(topcoat::router::error::see_other(&back));
    }
    let csrf_ok = crate::auth::session::verify_csrf(db, cx, &input.csrf_token).await
        || crate::auth::session::verify_csrf_seed(cx, &input.csrf_token);
    if !csrf_ok {
        crate::auth::access::audit(db, cx, "", "csrf.blocked", "", "", "showcase.inquire").await;
        return Ok(topcoat::router::error::see_other("/"));
    }
    let name = input.name.trim().chars().take(40).collect::<String>();
    let contact = input.contact.trim().chars().take(80).collect::<String>();
    let message = input.message.trim().chars().take(2000).collect::<String>();
    if name.is_empty() || contact.is_empty() {
        return Ok(topcoat::router::error::see_other(format!(
            "{back}?error=empty"
        )));
    }
    let ip = crate::auth::session::client_ip(cx);
    let recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inquiries WHERE ip = $1 AND created_at::timestamptz > NOW() - make_interval(secs => $2)",
    )
    .bind(&ip)
    .bind(INQUIRY_WINDOW_SECS)
    .fetch_one(db)
    .await
    .unwrap_or(0);
    if recent >= INQUIRY_MAX_PER_WINDOW {
        return Ok(topcoat::router::error::see_other(format!(
            "{back}?error=rate"
        )));
    }
    let product_id: Option<String> = sqlx::query_scalar("SELECT id FROM products WHERE slug = $1")
        .bind(slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    sqlx::query(
        "INSERT INTO inquiries (product_id, name, contact, message, ip) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&product_id)
    .bind(&name)
    .bind(&contact)
    .bind(&message)
    .bind(&ip)
    .execute(db)
    .await
    .ok();
    Ok(topcoat::router::error::see_other(format!("{back}?sent=1")))
}
