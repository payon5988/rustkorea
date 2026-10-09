HARNESS: 테스트 전략 및 자동화
1. 테스트 피라미드 및 도구
단위 테스트 (70%): cargo nextest (병렬 실행으로 속도 향상), mockall (외부 의존성 목킹).
통합 테스트 (25%): testcontainers (PostgreSQL, Redis 컨테이너를 테스트마다 격리하여 기동).
E2E 테스트 (5%): playwright (Rust 바인딩 또는 TS 스크립트)로 핵심 사용자 여정 검증.
2. 통합 테스트 구현 예시 (Topcoat + Testcontainers)
   // tests/manual_integration.rs
use sqlx::PgPool;
use testcontainers::{clients::Cli, RunnableImage};
use testcontainers_modules::postgres::Postgres;

async fn setup_db() -> PgPool {
    let docker = Cli::default();
    let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
    let container = docker.run(image);
    let conn_str = format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        container.get_host_port_ipv4(5432),
        container.get_host_port_ipv4(5432)
    );
    sqlx::PgPool::connect(&conn_str).await.unwrap()
}

#[sqlx::test]
async fn test_create_and_fetch_manual_series(pool: PgPool) {
    // 1. Given: DB에 시리즈 삽입
    let series_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO manual_series (id, slug, title) VALUES ($1, $2, $3)"#,
        series_id, "topcoat-basics", "Topcoat 기초"
    )
    .execute(&pool)
    .await
    .unwrap();

    // 2. When: Topcoat Cx를 모킹하여 페칭 함수 호출 (실제 구현체에 따라 다름)
    // let result = fetch_manual_series(&mock_cx, &pool).await;

    // 3. Then: 결과 검증
    // assert_eq!(result.unwrap().title, "Topcoat 기초");
}
 3. CI/CD 파이프라인 (GitHub Actions)
name: Rust Korea Hub CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  quality-and-test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_PASSWORD: postgres
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Install Topcoat CLI
        run: cargo install topcoat-cli

      - name: Format check
        run: topcoat fmt --check

      - name: Clippy (Deny warnings)
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests with nextest
        run: cargo nextest run
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/postgres

      - name: Build release binary
        run: cargo build --release
4. 성능 테스트 (k6)
시나리오: 50명의 가상 사용자가 1분 동안 /manuals/topcoat-basics 페이지에 지속적으로 접근.
목표: p95 응답 시간 < 200ms, 에러율 0%.
실행: k6 run scripts/performance.js (C
---


