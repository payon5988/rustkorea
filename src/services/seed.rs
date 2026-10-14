use crate::db::Db;

use super::markdown::render_markdown;

pub async fn seed_database(db: &Db) {
    // Guest user for anonymous posts
    sqlx::query(
        "INSERT INTO users (id, username, display_name, created_at, updated_at)
         VALUES ('guest', 'guest', '방문자', NOW()::text, NOW()::text) ON CONFLICT (id) DO NOTHING",
    )
    .execute(db)
    .await
    .ok();

    // Admin user is bootstrapped from env (ADMIN_EMAIL / ADMIN_PASSWORD) on
    // first boot. No default credentials are seeded.
    ensure_admin_from_env(db).await;
    reset_totp_from_env(db).await;

    // Default site settings (idempotent)
    let channel_id = crate::config::youtube_channel_id();
    let defaults = [
        ("youtube_channel_id", channel_id.as_str()),
        ("site_title", "Rust Korea Hub"),
        ("site_description", "한국 Rust 생태계의 단일 진실 공급원"),
        ("site_keywords", "rust, 러스트, 커뮤니티, topcoat"),
        (
            "footer_text",
            "2026 Rust Korea. Built with Rust and Topcoat.",
        ),
        ("posts_per_page", "20"),
        ("logo_url", ""),
        ("adsense_enabled", "false"),
        ("adsense_client", ""),
        ("require_approval", "false"),
        ("youtube_visibility", "public"),
        ("banner_enabled", "false"),
        ("banner_until", ""),
        ("banner_kicker", "저서 출간"),
        ("banner_title", "《미래를 묻는 인간》"),
        (
            "banner_text",
            "AI 시대의 초생산성 공식을 Rust와 바이브 코딩으로 실증합니다.",
        ),
        ("banner_cta", "실증 철학 보기"),
    ];
    for (k, v) in defaults {
        sqlx::query(
            "INSERT INTO site_settings (key, value) VALUES ($1, $2) ON CONFLICT (key) DO NOTHING",
        )
        .bind(k)
        .bind(v)
        .execute(db)
        .await
        .ok();
    }

    // Default header nav items
    let nav_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM nav_items")
        .fetch_one(db)
        .await
        .unwrap_or((0,));
    if nav_count.0 == 0 {
        let default_navs = vec![
            ("매뉴얼", "/manuals", 0),
            ("영상", "/youtube", 1),
            ("커뮤니티", "/community", 2),
            ("검색", "/search", 3),
        ];
        for (label, href, position) in default_navs {
            sqlx::query("INSERT INTO nav_items (label, href, position, visible, section) VALUES ($1, $2, $3, 1, 'header')")
                .bind(label)
                .bind(href)
                .bind(position)
                .execute(db).await.ok();
        }
    }

    // Default categories
    let cat_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories")
        .fetch_one(db)
        .await
        .unwrap_or((0,));
    if cat_count.0 == 0 {
        let cats = vec![
            ("general", "일반 토론", "#6b7280", "자유로운 주제의 토론", 0),
            ("qna", "질문과 답변", "#2563EB", "Rust 및 개발 관련 질문", 1),
            (
                "showcase",
                "프로젝트 쇼케이스",
                "#7C3AED",
                "자신의 프로젝트를 공유하세요",
                2,
            ),
            ("jobs", "구인구직", "#059669", "개발자 구인/구직 게시판", 3),
            ("announce", "공지사항", "#DC2626", "사이트 공지사항", 4),
        ];
        for (slug, name, color, desc, pos) in cats {
            sqlx::query("INSERT INTO categories (id, slug, name, color, description, position, visible) VALUES ($1, $2, $3, $4, $5, $6, 1) ON CONFLICT (id) DO NOTHING")
                .bind(slug).bind(slug).bind(name).bind(color).bind(desc).bind(pos)
                .execute(db).await.ok();
        }
        sqlx::query("UPDATE categories SET write_permission = 'admin' WHERE slug = 'announce'")
            .execute(db)
            .await
            .ok();
        sqlx::query("UPDATE categories SET write_permission = 'editor' WHERE slug = 'jobs'")
            .execute(db)
            .await
            .ok();
    }

    // One-shot stale-video cleanup (opt-in: empty by default so a
    // misconfigured channel id can never wipe the table on boot).
    if std::env::var("YOUTUBE_PRUNE_MISMATCH").as_deref() == Ok("1") {
        sqlx::query("DELETE FROM youtube_videos WHERE channel_id NOT LIKE $1 || '%'")
            .bind(crate::config::youtube_channel_id())
            .execute(db)
            .await
            .ok();
    }

    // Check if seed data already exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM manual_series")
        .fetch_one(db)
        .await
        .unwrap_or((0,));
    if count.0 > 0 {
        ensure_badges(db).await;
        return;
    }

    // Seed manual series 1: Topcoat 입문
    sqlx::query(
        "INSERT INTO manual_series (slug, title, description, difficulty, estimated_minutes)
         VALUES ('topcoat-getting-started', 'Topcoat으로 만드는 풀스택 Rust 웹개발', 'Topcoat 프레임워크를 사용하여 Rust로 풀스택 웹 애플리케이션을 만드는 방법을 단계별로 배웁니다.', 'beginner', 360)
         ON CONFLICT (slug) DO NOTHING"
    ).execute(db).await.ok();

    let series_id: Option<(String,)> =
        sqlx::query_as("SELECT id FROM manual_series WHERE slug = 'topcoat-getting-started'")
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
    let Some(series_id) = series_id else {
        eprintln!("[Seed] manual series seed skipped (lookup failed)");
        ensure_badges(db).await;
        return;
    };

    // Chapter 1
    let ch1_md = "# Topcoat이란?\n\nTopcoat은 **Rust**로 작성된 풀스택 웹 프레임워크입니다.\n\n## 주요 특징\n\n- **서버사이드 렌더링**: HTML을 서버에서 렌더링하여 SEO와 성능을 확보합니다\n- **React式的 뷰 시스템**: `view!` 매크로로 HTML 템플릿을 작성합니다\n- **타입 안전성**: Rust의 타입 시스템으로 런타임 에러를 최소화합니다\n- **Tailwind CSS 통합**: 내장된 Tailwind CSS 빌드 시스템\n\n## Hello World\n\n```rust\n#[page(\"/\")]\npub async fn home() -> Result {\n    view! {\n        <h1>\"Hello, Rust Korea!\"</h1>\n    }\n}\n```\n\n다음 장에서는 프로젝트를 세팅해봅시다!";
    let ch1_html = render_markdown(ch1_md);
    sqlx::query(
        "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, chapter_index)
         VALUES ($1, '01-what-is-topcoat', '1장: Topcoat이란?', $2, $3, 0)
         ON CONFLICT DO NOTHING"
    )
    .bind(&series_id.0)
    .bind(ch1_md)
    .bind(ch1_html)
    .execute(db).await.ok();

    // Chapter 2
    let ch2_md = "# 프로젝트 세팅\n\n## 필수 도구 설치\n\n```bash\n# Rust 설치\ncurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n\n# Topcoat CLI 설치\ncargo install topcoat-cli\n```\n\n## 새 프로젝트 만들기\n\n```bash\ntopcoat new my-app\ncd my-app\ncargo run\n```\n\n## 프로젝트 구조\n\n```\nmy-app/\n  src/\n    main.rs    # 엔트리포인트\n    app.rs     # 레이아웃\n    pages/     # 페이지 컴포넌트\n  migrations/  # DB 마이그레이션\n  assets/      # 정적 파일\n```\n\n## 다음 단계\n\n서버를 실행하면 `http://localhost:3000`에서 확인할 수 있습니다!";
    let ch2_html = render_markdown(ch2_md);
    sqlx::query(
        "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, chapter_index)
         VALUES ($1, '02-project-setup', '2장: 프로젝트 세팅', $2, $3, 1)
         ON CONFLICT DO NOTHING"
    )
    .bind(&series_id.0)
    .bind(ch2_md)
    .bind(ch2_html)
    .execute(db).await.ok();

    // Chapter 3
    let ch3_md = "# 뷰와 컴포넌트\n\n## view! 매크로\n\nTopcoat의 핵심은 `view!` 매크로입니다:\n\n```rust\nuse topcoat::view::view;\n\n#[page(\"/\")]\npub async fn home() -> Result {\n    view! {\n        <div class=\"container\">\n            <h1>\"제목\"</h1>\n            <p>\"설명\"</p>\n        </div>\n    }\n}\n```\n\n## 컴포넌트\n\n```rust\n#[component]\npub async fn Greeting(name: String) -> Result {\n    view! {\n        <p>(format!(\"안녕하세요, {name}!\"))</p>\n    }\n}\n```\n\n## 조건문과 반복문\n\n```rust\nview! {\n    if show_detail {\n        <p>\"상세 정보\"</p>\n    }\n    \n    for item in items {\n        <li>(item.name)</li>\n    }\n}\n```\n\n## 레이아웃\n\n`#[layout]`으로 페이지 간 레이아웃을 공유할 수 있습니다.";
    let ch3_html = render_markdown(ch3_md);
    sqlx::query(
        "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, chapter_index)
         VALUES ($1, '03-views-and-components', '3장: 뷰와 컴포넌트', $2, $3, 2)
         ON CONFLICT DO NOTHING"
    )
    .bind(&series_id.0)
    .bind(ch3_md)
    .bind(ch3_html)
    .execute(db).await.ok();

    // Seed manual series 2: Rust 기초
    sqlx::query(
        "INSERT INTO manual_series (slug, title, description, difficulty, estimated_minutes)
         VALUES ('rust-basics', 'Rust 기초 문법 정리', 'Rust를 처음 접하는 개발자를 위한 기초 문법 가이드', 'beginner', 240)
         ON CONFLICT (slug) DO NOTHING"
    ).execute(db).await.ok();

    let series2_id: Option<(String,)> =
        sqlx::query_as("SELECT id FROM manual_series WHERE slug = 'rust-basics'")
            .fetch_optional(db)
            .await
            .ok()
            .flatten();
    let Some(series2_id) = series2_id else {
        eprintln!("[Seed] manual series seed skipped (lookup failed)");
        ensure_badges(db).await;
        return;
    };

    let rb1_md = "# 변수와 타입\n\n## 변수 선언\n\n```rust\nlet x = 5;          // 불변\nlet mut y = 10;     // 가변\nlet z: i32 = 20;    // 타입 명시\n```\n\n## 기본 타입\n\n| 타입 | 설명 |\n|------|------|\n| `i32`, `u32` | 정수 |\n| `f64` | 부동소수점 |\n| `bool` | 불리언 |\n| `String` | 문자열 |\n| `Vec<T>` | 벡터 |\n| `Option<T>` | 선택적 값 |\n| `Result<T, E>` | 결과 타입 |\n\n## 소유권 (Ownership)\n\nRust의 가장 중요한 개념:\n\n```rust\nlet s1 = String::from(\"hello\");\nlet s2 = s1;  // s1은 더 이상 유효하지 않음\n\nlet s3 = s2.clone();  // 깊은 복사\n```\n\n## 참조 (References)\n\n```rust\nfn length(s: &String) -> usize {\n    s.len()\n}\n\nlet s = String::from(\"hello\");\nlet len = length(&s);\n```\n\n다음 장에서는 함수와 모듈에 대해 배웁니다!";
    let rb1_html = render_markdown(rb1_md);
    sqlx::query(
        "INSERT INTO manual_chapters (series_id, slug, title, content_md, content_html, chapter_index)
         VALUES ($1, '01-variables', '1장: 변수와 타입', $2, $3, 0)
         ON CONFLICT DO NOTHING"
    )
    .bind(&series2_id.0)
    .bind(rb1_md)
    .bind(rb1_html)
    .execute(db).await.ok();

    // Seed manual series 3: DB 연동
    sqlx::query(
        "INSERT INTO manual_series (slug, title, description, difficulty, estimated_minutes)
         VALUES ('database-integration', '데이터베이스 연동 실습', 'sqlx를 사용한 SQLite/PostgreSQL 연동 방법', 'intermediate', 180)
         ON CONFLICT (slug) DO NOTHING"
    ).execute(db).await.ok();

    println!("[Seed] 샘플 데이터 삽입 완료");

    ensure_badges(db).await;
}

async fn ensure_badges(db: &Db) {
    let badge_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM badges")
        .fetch_one(db)
        .await
        .unwrap_or((0,));
    if badge_count.0 == 0 {
        let badges = vec![
            ("newcomer", "새내기", "가입과 첫 글 작성", 0),
            ("regular", "정회원", "100P 달성", 100),
            ("contributor", "기여자", "500P 달성", 500),
            ("veteran", "베테랑", "1000P 달성", 1000),
            ("legend", "전설", "5000P 달성", 5000),
        ];
        for (id, name, desc, points) in badges {
            sqlx::query("INSERT INTO badges (id, name, description, points_required) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING")
                .bind(id).bind(name).bind(desc).bind(points).execute(db).await.ok();
        }
        println!("[Seed] 배지 시드 완료");
    }
}

/// Bootstrap the admin account from environment on first boot.
/// Requires ADMIN_EMAIL and ADMIN_PASSWORD (>= 10 chars). Refuses to run
/// with the historical default password.
async fn ensure_admin_from_env(db: &Db) {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users u JOIN roles r ON r.id = u.role_id WHERE r.is_superuser = 1",
    )
    .fetch_one(db)
    .await
    .unwrap_or((1,));
    if count.0 > 0 {
        if std::env::var("ADMIN_PASSWORD").is_ok() || std::env::var("TOTP_RESET_EMAIL").is_ok() {
            eprintln!(
                "[Seed] 경고: ADMIN_PASSWORD/TOTP_RESET_EMAIL이 설정된 채로 실행 중입니다. 제거하세요."
            );
        }
        return;
    }
    let email = std::env::var("ADMIN_EMAIL").unwrap_or_default();
    let password = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
    if email.is_empty() || password.is_empty() {
        println!("[Seed] ADMIN_EMAIL/ADMIN_PASSWORD 미설정: 관리자 계정 생성을 건너뜁니다");
        return;
    }
    if password.len() < 10 || password == "admin123" {
        eprintln!(
            "[Seed] ADMIN_PASSWORD가 너무 약합니다(10자 이상, admin123 금지). 관리자 계정 생성을 거부합니다"
        );
        return;
    }
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();
    if hash.is_empty() {
        return;
    }
    sqlx::query(
        "INSERT INTO users (email, username, display_name, password_hash, role_id, created_at, updated_at)
         VALUES ($1, 'admin', '관리자', $2, 'admin', NOW()::text, NOW()::text) ON CONFLICT (email) DO NOTHING",
    )
    .bind(&email)
    .bind(&hash)
    .execute(db)
    .await
    .ok();
    println!("[Seed] 환경변수 기반 관리자 계정 생성: {email}");
}

/// Emergency TOTP recovery: set TOTP_RESET_EMAIL to disable 2FA for that
/// account on boot (use when the authenticator device is lost, then unset).
async fn reset_totp_from_env(db: &Db) {
    let email = std::env::var("TOTP_RESET_EMAIL").unwrap_or_default();
    if email.is_empty() {
        return;
    }
    let done = sqlx::query("UPDATE users SET totp_secret = '', totp_enabled = 0 WHERE email = $1")
        .bind(&email)
        .execute(db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);
    if done > 0 {
        println!("[Seed] TOTP 해제됨: {email} (TOTP_RESET_EMAIL 정리 요망)");
    }
}
