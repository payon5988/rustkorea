use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::content::Form,
    router::{page, path_param, route},
    view::{View, view},
};

use crate::components::ad_unit::AdUnit;
use crate::db::Db;
use crate::models::ManualChapter;

topcoat::router::path_param!(series_slug);

topcoat::router::path_param!(chapter_slug);

#[page("/manuals/{series_slug}/{chapter_slug}")]
pub async fn chapter_detail(cx: &Cx) -> Result<impl View + '_> {
    let series_slug = path_param::<SeriesSlug>(cx);
    let chapter_slug = path_param::<ChapterSlug>(cx);
    let db: &Db = topcoat::context::app_context(cx);
    let csrf_token = crate::auth::session::csrf_token(db, cx).await;
    let access = crate::auth::access::load_access(db, cx).await;
    let loc = crate::services::i18n::locale(cx);

    // Effective gate is the chapter's own tier: a series can mix public
    // previews with members/staff-only chapters. Both chapter and series
    // must be released (published_at) for public access.
    let chapter: Option<ManualChapter> = sqlx::query_as::<_, ManualChapter>(&format!(
        r#"SELECT mc.* FROM manual_chapters mc
            JOIN manual_series ms ON mc.series_id = ms.id
            WHERE ms.slug = $1 AND mc.slug = $2 AND {} AND {}"#,
        crate::util::published_clause("mc"),
        crate::util::published_clause("ms"),
    ))
    .bind(series_slug)
    .bind(chapter_slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    let gate = chapter
        .as_ref()
        .map(|c| {
            crate::auth::access::visibility_gate(
                access.as_ref(),
                &c.visibility,
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS,
            )
        })
        .unwrap_or(crate::auth::access::Gate::Allow);
    if gate == crate::auth::access::Gate::LoginRequired {
        return Err(topcoat::router::error::redirect(format!(
            "/auth/login?redirect=/manuals/{series_slug}/{chapter_slug}"
        ))
        .into());
    }
    let forbidden = gate == crate::auth::access::Gate::Forbidden;
    let forbidden_msg: String = chapter
        .as_ref()
        .map(|c| match c.visibility.as_str() {
            "members" => loc.pick(
                "이 챕터는 회원 전용입니다.",
                "This chapter is for members only.",
            ),
            "staff" => loc.pick(
                "이 챕터는 스태프 전용입니다.",
                "This chapter is for staff only.",
            ),
            _ => loc.pick(
                "이 챕터는 유료 전용입니다(준비중).",
                "This chapter is premium-only (coming soon).",
            ),
        })
        .unwrap_or_default()
        .to_string();
    let chapter = if forbidden { None } else { chapter };

    /// Neighbor chapter (prev/next) with its gate precomputed.
    struct Neighbor {
        slug: String,
        title: String,
        gate: crate::auth::access::Gate,
    }
    let prev: Option<Neighbor> = if let Some(ch) = &chapter {
        sqlx::query_as::<_, (String, String, String, String)>(
            &format!("SELECT slug, title, title_en, visibility FROM manual_chapters
             WHERE series_id = $1 AND chapter_index < $2 AND {} ORDER BY chapter_index DESC LIMIT 1",
                crate::util::published_clause("")),
        )
        .bind(&ch.series_id)
        .bind(ch.chapter_index)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .map(|(slug, title, title_en, visibility)| Neighbor {
            slug,
            title: loc.text_owned(&title, &title_en),
            gate: crate::auth::access::visibility_gate(
                access.as_ref(),
                &visibility,
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS,
            ),
        })
    } else {
        None
    };
    let next: Option<Neighbor> = if let Some(ch) = &chapter {
        sqlx::query_as::<_, (String, String, String, String)>(&format!(
            "SELECT slug, title, title_en, visibility FROM manual_chapters
             WHERE series_id = $1 AND chapter_index > $2 AND {} ORDER BY chapter_index ASC LIMIT 1",
            crate::util::published_clause("")
        ))
        .bind(&ch.series_id)
        .bind(ch.chapter_index)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .map(|(slug, title, title_en, visibility)| Neighbor {
            slug,
            title: loc.text_owned(&title, &title_en),
            gate: crate::auth::access::visibility_gate(
                access.as_ref(),
                &visibility,
                crate::auth::access::perms::MANUALS_VIEW_STAFF,
                crate::auth::access::perms::PREMIUM_ACCESS,
            ),
        })
    } else {
        None
    };
    // English view uses the translation when present; otherwise the Korean
    // original is shown with a notice.
    let show_en = loc == crate::services::i18n::Locale::En
        && chapter
            .as_ref()
            .is_some_and(|ch| !ch.content_html_en.trim().is_empty());
    let untranslated = loc == crate::services::i18n::Locale::En && !show_en;
    // Reading pace: Korean ~500 chars/min, English ~200 words/min, min 1.
    // TOC auto-shows only for heading-rich chapters (>= 4); short split
    // chapters keep full width with an on-demand toggle.
    let (read_minutes, heading_count): (usize, usize) = chapter
        .as_ref()
        .map(|ch| {
            let md = if show_en {
                &ch.content_md_en
            } else {
                &ch.content_md
            };
            let minutes = if show_en {
                md.split_whitespace().count() / 200
            } else {
                md.chars().count() / 500
            }
            .max(1);
            let headings = md
                .lines()
                .filter(|l| {
                    let t = l.trim_start();
                    t.starts_with("# ") || t.starts_with("## ") || t.starts_with("### ")
                })
                .count();
            (minutes, headings)
        })
        .unwrap_or((1, 0));
    let toc_open = heading_count >= 4;

    let current_user = crate::auth::session::current_user(db, cx).await;
    let completed: bool = if let (Some(u), Some(ch)) = (&current_user, &chapter) {
        sqlx::query_scalar::<_, i32>(
            "SELECT completed FROM reading_progress WHERE user_id = $1 AND chapter_id = $2",
        )
        .bind(&u.id)
        .bind(&ch.id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .map(|c| c == 1)
        .unwrap_or(false)
    } else {
        false
    };

    Ok(view! {
        <div class="container" style="padding-top:40px; padding-bottom:80px; max-width:960px;">
            <style>
                {r#".reader-layout { display: flex; gap: 32px; align-items: flex-start; }
                .reader-main { flex: 1; min-width: 0; }
                #toc-wrap { width: 220px; flex-shrink: 0; position: sticky; top: 80px; max-height: calc(100vh - 120px); overflow-y: auto; background: #fff; border: 1px solid #E4E4E7; border-radius: 12px; padding: 16px 18px; }
                #toc-wrap h2 { font-size: 12px; font-weight: 700; color: #9ca3af; text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 10px; }
                #toc-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
                #toc-list a { font-size: 13px; color: #4b5563; line-height: 1.5; display: block; }
                #toc-list a:hover { color: #2563EB; }
                @media (max-width: 900px) { #toc-wrap { display: none; } .reader-layout { display: block; } }"#}
            </style>
            if let Some(ch) = chapter {
                <a href=(format!("/manuals/{}", series_slug)) style="font-size:13px; color:#6b7280; display:inline-flex; align-items:center; gap:6px; margin-bottom:24px;">
                    (loc.pick("<- 시리즈 목록", "<- Series contents"))
                </a>

                <h1 style="font-size:1.75rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px;">(loc.text_owned(&ch.title, &ch.title_en))</h1>
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:32px;">
                    <p style="font-size:13px; color:#9ca3af;">(match loc {
                        crate::services::i18n::Locale::Ko => format!("약 {read_minutes}분 읽기"),
                        crate::services::i18n::Locale::En => format!("{read_minutes} min read"),
                    })</p>
                    if heading_count > 0 {
                        <button type="button" onclick="toggleToc()" style="font-size:12px; font-weight:600; color:#2563EB; background:#eff6ff; border:1px solid #bfdbfe; border-radius:6px; padding:4px 12px; cursor:pointer;">(loc.pick("목차", "Contents"))</button>
                    }
                </div>
                if untranslated {
                    <p style="font-size:13px; color:#92400e; background:#fffbeb; border:1px solid #fde68a; border-radius:8px; padding:10px 14px; margin:-16px 0 24px;">
                        "This chapter has not been translated into English yet. The Korean original is shown below."
                    </p>
                }

                <div class="reader-layout">
                    <div class="reader-main">
                        <div class="card" style="padding:40px 48px;">
                            <div class="markdown-body" lang=(if show_en { "en" } else { "ko" })>
                                (topcoat::view::Unescaped::new_unchecked(if show_en { ch.content_html_en.clone() } else { ch.content_html.clone() }))
                            </div>
                        </div>
                    </div>
                    <aside id="toc-wrap" style=((if toc_open { "" } else { "display:none;" }).to_string())>
                        <h2>(loc.pick("목차", "Contents"))</h2>
                        <ul id="toc-list"></ul>
                    </aside>
                </div>
                <script>{(crate::services::markdown::TOC_JS)}</script>
                <script>"buildToc();"</script>

                {AdUnit(slot_id: "chapter_bottom".into())}

                <link rel="stylesheet" href=(crate::services::markdown::HL_CSS_URL) integrity=(crate::services::markdown::HL_CSS_SRI) crossorigin="anonymous">
                <script src=(crate::services::markdown::HL_JS_URL) integrity=(crate::services::markdown::HL_JS_SRI) crossorigin="anonymous"></script>
                <script src=(crate::services::markdown::HL_RUST_URL) integrity=(crate::services::markdown::HL_RUST_SRI) crossorigin="anonymous"></script>
                <script>{(crate::services::markdown::HL_INIT_JS)}</script>
                <script>"initHighlight();"</script>

                <div style="margin-top:32px; display:flex; align-items:center; gap:12px; flex-wrap:wrap;">
                    if current_user.is_some() {
                        if completed {
                            <span style="display:inline-flex; align-items:center; gap:6px; font-size:13px; font-weight:600; color:#059669; background:#ecfdf5; border:1px solid #a7f3d0; padding:8px 16px; border-radius:8px;">(loc.pick("✓ 읽음 완료", "✓ Completed"))</span>
                        } else {
                            <form action=(format!("/manuals/{series_slug}/{chapter_slug}/complete")) method="POST" style="display:inline;">
                                <input type="hidden" name="csrf_token" value=(csrf_token.clone())>
                                <button type="submit" class="btn-primary">(loc.pick("읽음 완료로 표시", "Mark as completed"))</button>
                            </form>
                        }
                    } else {
                        <a href=(format!("/auth/login?redirect=/manuals/{series_slug}/{chapter_slug}")) class="btn-secondary">(loc.pick("로그인하고 진행률 저장", "Log in to save progress"))</a>
                    }
                    <a href=(format!("/manuals/{}", series_slug)) style="font-size:13px; color:#6b7280; transition:color 0.15s;">
                        (loc.pick("<- 시리즈 목록으로 돌아가기", "<- Back to series contents"))
                    </a>
                </div>

                <div style="display:flex; gap:12px; margin-top:24px;">
                    if let Some(nb) = prev {
                        if nb.gate == crate::auth::access::Gate::Allow {
                            <a href=(format!("/manuals/{}/{}", series_slug, nb.slug)) class="card" style="flex:1; padding:16px 20px; display:block;">
                                <div style="font-size:11px; color:#9ca3af; margin-bottom:4px;">(loc.pick("<- 이전 챕터", "<- Previous chapter"))</div>
                                <div style="font-size:14px; font-weight:600;">(nb.title.clone())</div>
                            </a>
                        } else {
                            <div class="card" style="flex:1; padding:16px 20px; background:#f8fafb;">
                                <div style="font-size:11px; color:#9ca3af; margin-bottom:4px;">(loc.pick("<- 이전 챕터 (🔒)", "<- Previous chapter (🔒)"))</div>
                                <div style="font-size:14px; font-weight:600; color:#9ca3af;">(nb.title.clone())</div>
                            </div>
                        }
                    } else {
                        <div style="flex:1;"></div>
                    }
                    if let Some(nb) = next {
                        if nb.gate == crate::auth::access::Gate::Allow {
                            <a href=(format!("/manuals/{}/{}", series_slug, nb.slug)) class="card" style="flex:1; padding:16px 20px; display:block; text-align:right;">
                                <div style="font-size:11px; color:#9ca3af; margin-bottom:4px;">(loc.pick("다음 챕터 ->", "Next chapter ->"))</div>
                                <div style="font-size:14px; font-weight:600;">(nb.title.clone())</div>
                            </a>
                        } else {
                            <div class="card" style="flex:1; padding:16px 20px; background:#f8fafb; text-align:right;">
                                <div style="font-size:11px; color:#9ca3af; margin-bottom:4px;">(loc.pick("다음 챕터 (🔒) ->", "Next chapter (🔒) ->"))</div>
                                <div style="font-size:14px; font-weight:600; color:#9ca3af;">(nb.title.clone())</div>
                            </div>
                        }
                    } else {
                        <div style="flex:1;"></div>
                    }
                </div>
            } else {
                <div style="text-align:center; padding:100px 0;">
                    if forbidden {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">(loc.pick("접근 권한이 없습니다", "Access denied"))</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">(forbidden_msg.clone())</p>
                    } else {
                        <h1 style="font-size:1.25rem; font-weight:700; margin-bottom:8px;">(loc.pick("챕터를 찾을 수 없습니다", "Chapter not found"))</h1>
                        <p style="color:#6b7280; margin-bottom:24px;">(loc.pick("요청하신 챕터가 존재하지 않습니다.", "The chapter you requested does not exist."))</p>
                    }
                    <a href="/manuals" class="btn-primary" style="display:inline-flex;">(loc.pick("매뉴얼 목록으로", "Back to manuals"))</a>
                </div>
            }
        </div>
    })
}

#[route(POST "/manuals/{series_slug}/{chapter_slug}/complete")]
pub async fn complete_chapter(
    cx: &Cx,
    Form(input): Form<crate::auth::session::CsrfForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let series_slug = path_param::<SeriesSlug>(cx);
    let chapter_slug = path_param::<ChapterSlug>(cx);
    let db: &Db = topcoat::context::app_context(cx);

    let back = format!("/manuals/{series_slug}/{chapter_slug}");
    let Some(user) = crate::auth::session::current_user(db, cx).await else {
        return Ok(topcoat::router::error::see_other(format!(
            "/auth/login?redirect={back}"
        )));
    };
    if !crate::auth::session::verify_csrf_logged(db, cx, &input.csrf_token, "manuals.complete")
        .await
    {
        return Ok(topcoat::router::error::see_other("/"));
    }

    let chapter_id: Option<String> = sqlx::query_scalar(&format!(
        "SELECT mc.id FROM manual_chapters mc
         JOIN manual_series ms ON mc.series_id = ms.id
         WHERE ms.slug = $1 AND mc.slug = $2 AND {} AND {}",
        crate::util::published_clause("mc"),
        crate::util::published_clause("ms"),
    ))
    .bind(series_slug)
    .bind(chapter_slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();
    if let Some(cid) = chapter_id {
        sqlx::query(
            "INSERT INTO reading_progress (user_id, chapter_id, scroll_pct, completed)
             VALUES ($1, $2, 100, 1)
             ON CONFLICT (user_id, chapter_id) DO UPDATE SET completed = 1, scroll_pct = 100, updated_at = NOW()::text",
        )
        .bind(&user.id)
        .bind(&cid)
        .execute(db)
        .await
        .ok();
    }
    Ok(topcoat::router::error::see_other(&back))
}
