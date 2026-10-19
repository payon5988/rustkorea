use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::{page, path_param},
    view::{View, view},
};

use crate::db::Db;
use crate::models::BlogPost;

topcoat::router::path_param!(slug);

fn post_gate(
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

#[page("/blog")]
pub async fn blog_index(cx: &Cx) -> Result<impl View + '_> {
    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;
    let loc = crate::services::i18n::locale(cx);

    let posts = sqlx::query_as::<_, BlogPost>(&format!(
        "SELECT * FROM blog_posts WHERE {} ORDER BY created_at DESC",
        crate::util::published_clause("")
    ))
    .fetch_all(db)
    .await
    .unwrap_or_default();
    let posts: Vec<BlogPost> = posts
        .into_iter()
        .filter(|p| post_gate(access.as_ref(), &p.visibility) == crate::auth::access::Gate::Allow)
        .collect();
    let (featured, rest) = match posts.as_slice() {
        [first, others @ ..] => (Some(first.clone()), others.to_vec()),
        [] => (None, vec![]),
    };

    Ok(view! {
        <div class="container" style="padding-top:60px; padding-bottom:80px;">
            <style>
                {r#".blog-featured { display:grid; grid-template-columns:minmax(0,1.2fr) minmax(0,1fr); align-items:center; }
                .blog-featured-copy { min-width:0; overflow-wrap:anywhere; }
                @media (max-width:768px) { .blog-featured { grid-template-columns:minmax(0,1fr); } }"#}
            </style>
            <div style="margin-bottom:40px;">
                <div style="font-size:11px; font-weight:700; color:#DE5B30; text-transform:uppercase; letter-spacing:0.08em; margin-bottom:8px;">"BLOG"</div>
                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">(loc.pick("블로그", "Blog"))</h1>
                <p class="section-sub">(loc.pick("Rust 소식과 오픈소스를 전합니다.", "News from the Rust and open-source world."))</p>
            </div>

            if posts.is_empty() {
                <div class="card" style="padding:80px; text-align:center;">
                    <p style="color:#6b7280;">(loc.pick("첫 글이 곧 공개됩니다.", "The first post is coming soon."))</p>
                </div>
            } else {
                if let Some(f) = featured {
                    <a href=(format!("/blog/{}", f.slug)) class="card blog-featured" style="gap:0; overflow:hidden; margin-bottom:24px;">
                        if !f.thumbnail_url.is_empty() {
                            <img class="blog-thumbnail" src=(f.thumbnail_url.clone()) alt=(loc.text_owned(&f.title, &f.title_en)) loading="lazy" style="width:100%; max-width:100%; height:auto; object-fit:contain; display:block;">
                        } else {
                            <div style="width:100%; height:280px; background:linear-gradient(135deg, #1f2833 0%, #25283d 100%); display:flex; align-items:center; justify-content:center; font-size:64px;">"🦀"</div>
                        }
                        <div class="blog-featured-copy" style="padding:32px;">
                            <span class="badge" style="background:#eff6ff; color:#2563EB; margin-bottom:12px;">(loc.pick("최신 글", "Latest"))</span>
                            <h2 style="font-size:1.4rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:10px;">(loc.text_owned(&f.title, &f.title_en))</h2>
                            <p style="font-size:14px; color:#6b7280; line-height:1.6; margin-bottom:14px;">(loc.text_owned(&f.summary, &f.summary_en))</p>
                            <div style="display:flex; gap:6px; flex-wrap:wrap;">
                                for tag in f.tag_list() {
                                    <span style="font-size:11px; color:#6b7280;">(format!("#{tag}"))</span>
                                }
                            </div>
                        </div>
                    </a>
                }
                <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(min(100%,300px), 1fr)); gap:20px;">
                    for item in rest {
                        <a href=(format!("/blog/{}", item.slug)) class="card" style="display:block; overflow:hidden;">
                            if !item.thumbnail_url.is_empty() {
                                <img class="blog-thumbnail" src=(item.thumbnail_url.clone()) alt=(loc.text_owned(&item.title, &item.title_en)) loading="lazy" style="width:100%; max-width:100%; height:auto; object-fit:contain; display:block;">
                            } else {
                                <div style="width:100%; height:170px; background:linear-gradient(135deg, #1f2833 0%, #25283d 100%); display:flex; align-items:center; justify-content:center; font-size:44px;">"🦀"</div>
                            }
                            <div style="padding:20px 22px;">
                                <h3 style="font-size:16px; font-weight:700; margin-bottom:6px;">(loc.text_owned(&item.title, &item.title_en))</h3>
                                <p style="font-size:13px; color:#6b7280; line-height:1.5; margin-bottom:10px;">(loc.text_owned(&item.summary, &item.summary_en))</p>
                                <div style="font-size:12px; color:#9ca3af;">(item.created_at.split(' ').next().unwrap_or(""))</div>
                            </div>
                        </a>
                    }
                </div>
            }
        </div>
    })
}

#[page("/blog/{slug}")]
pub async fn blog_detail(cx: &Cx) -> Result<impl View + '_> {
    let slug = path_param::<Slug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let access = crate::auth::access::load_access(db, cx).await;

    let post: Option<BlogPost> = sqlx::query_as::<_, BlogPost>(&format!(
        "SELECT * FROM blog_posts WHERE slug = $1 AND {}",
        crate::util::published_clause("")
    ))
    .bind(slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let gate = post
        .as_ref()
        .map(|p| post_gate(access.as_ref(), &p.visibility))
        .unwrap_or(crate::auth::access::Gate::Allow);
    if gate == crate::auth::access::Gate::LoginRequired {
        return Err(
            topcoat::router::error::redirect(format!("/auth/login?redirect=/blog/{slug}")).into(),
        );
    }
    let forbidden = gate == crate::auth::access::Gate::Forbidden;
    let post = if forbidden { None } else { post };

    let loc = crate::services::i18n::locale(cx);
    let show_en = loc == crate::services::i18n::Locale::En
        && post
            .as_ref()
            .is_some_and(|p| !p.content_html_en.trim().is_empty());
    let untranslated = loc == crate::services::i18n::Locale::En && !show_en;
    // Korean ~500 chars/min, English ~200 words/min.
    let read_minutes: usize = post
        .as_ref()
        .map(|p| {
            if show_en {
                p.content_md_en.split_whitespace().count() / 200
            } else {
                p.content_md.chars().count() / 500
            }
            .max(1)
        })
        .unwrap_or(1);

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:860px;">
            <style>
                {r#".blog-layout { display: flex; gap: 28px; align-items: flex-start; }
                .blog-main { flex: 1; min-width: 0; }
                @media (max-width: 900px) { #toc-wrap { display: none; } .blog-layout { display: block; } }"#}
            </style>
            if let Some(p) = post {
                <a href="/blog" style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">
                    (loc.pick("<- 블로그", "<- Blog"))
                </a>

                if !p.thumbnail_url.is_empty() {
                    <img class="blog-thumbnail" src=(p.thumbnail_url.clone()) alt=(loc.text_owned(&p.title, &p.title_en)) style="width:100%; max-width:100%; height:auto; object-fit:contain; border-radius:14px; margin-bottom:28px; display:block;">
                }

                <div style="display:flex; align-items:center; gap:8px; margin-bottom:10px; flex-wrap:wrap;">
                    for tag in p.tag_list() {
                        <span style="font-size:11px; font-weight:600; padding:3px 9px; border-radius:9999px; color:#1e40af; background:#dbeafe;">(format!("#{tag}"))</span>
                    }
                </div>

                <h1 style="font-size:2rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">(loc.text_owned(&p.title, &p.title_en))</h1>
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:32px;">
                    <p style="font-size:13px; color:#9ca3af;">(p.created_at.split(' ').next().unwrap_or(""))</p>
                    <p style="font-size:13px; color:#9ca3af;">(match loc {
                        crate::services::i18n::Locale::Ko => format!("약 {read_minutes}분 읽기"),
                        crate::services::i18n::Locale::En => format!("{read_minutes} min read"),
                    })</p>
                    if !p.source_url.is_empty() {
                        <a href=(p.source_url.clone()) target="_blank" rel="noopener" style="font-size:13px; color:#2563EB; font-weight:600;">(loc.pick("원문 보기", "View source"))</a>
                    }
                </div>
                if untranslated {
                    <p style="font-size:13px; color:#92400e; background:#fffbeb; border:1px solid #fde68a; border-radius:8px; padding:10px 14px; margin:-16px 0 24px;">
                        "This post has not been translated into English yet. The Korean original is shown below."
                    </p>
                }

                <div class="blog-layout">
                    <div class="blog-main">
                        <div class="card" style="padding:40px 48px;">
                            <div class="markdown-body" lang=(if show_en { "en" } else { "ko" })>
                                (topcoat::view::Unescaped::new_unchecked(if show_en { p.content_html_en.clone() } else { p.content_html.clone() }))
                            </div>
                        </div>
                    </div>
                    <aside id="toc-wrap" style="width:200px; flex-shrink:0; position:sticky; top:80px; max-height:calc(100vh - 120px); overflow-y:auto; background:#fff; border:1px solid #E4E4E7; border-radius:12px; padding:16px 18px;">
                        <h2 style="font-size:12px; font-weight:700; color:#9ca3af; text-transform:uppercase; letter-spacing:0.06em; margin-bottom:10px;">(loc.pick("목차", "Contents"))</h2>
                        <ul id="toc-list" style="list-style:none; margin:0; padding:0; display:flex; flex-direction:column; gap:6px;"></ul>
                    </aside>
                </div>
                <script>{(crate::services::markdown::TOC_JS)}</script>
                <script>"buildToc();"</script>

                <link rel="stylesheet" href=(crate::services::markdown::HL_CSS_URL) integrity=(crate::services::markdown::HL_CSS_SRI) crossorigin="anonymous">
                <script src=(crate::services::markdown::HL_JS_URL) integrity=(crate::services::markdown::HL_JS_SRI) crossorigin="anonymous"></script>
                <script src=(crate::services::markdown::HL_RUST_URL) integrity=(crate::services::markdown::HL_RUST_SRI) crossorigin="anonymous"></script>
                <script>{(crate::services::markdown::HL_INIT_JS)}</script>
                <script>"initHighlight();"</script>
            } else {
                <div style="text-align:center; padding:100px 0;">
                    if forbidden {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">(loc.pick("접근 권한이 없습니다", "Access denied"))</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">(loc.pick("회원 전용 글입니다.", "This post is for members only."))</p>
                    } else {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">(loc.pick("글을 찾을 수 없습니다", "Post not found"))</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">(loc.pick("요청하신 글이 존재하지 않습니다.", "The post you requested does not exist."))</p>
                    }
                    <a href="/blog" class="btn-primary" style="display:inline-flex;">(loc.pick("블로그로", "Back to blog"))</a>
                </div>
            }
        </div>
    })
}
