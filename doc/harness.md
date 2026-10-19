# HARNESS: 테스트 전략 및 자동화 (현행)

초기 기획(nextest, mockall, testcontainers, Playwright, k6)은 아직 쓰지 않는다.
현재 테스트는 **`cargo test` 단위 테스트**와 **실제 라우터로 보내는 DB 회귀 테스트**의 두 층이다.

## 1. 테스트 종류

| 종류 | 위치 | 실행 조건 |
|---|---|---|
| 단위 테스트 | 각 모듈 `#[cfg(test)]` (markdown, navigation, outbound, uploads, youtube, util, access, totp …) | 항상 |
| DB 회귀 테스트 | `#[sqlx::test]` + `#[ignore]`: `pages/admin/authz_tests.rs`, `products_tests.rs`, `services/navigation_tests.rs` | `DATABASE_URL` + 자산 번들 + `--include-ignored` |
| 네트워크 테스트 | `services::outbound::fetches_public_page_and_caps_size` | `RUN_NETWORK_TESTS=1` |

DB 회귀 테스트는 `Router::builder()...discover()`로 실제 앱 라우터를 만들고, `router.handle(Request)`로 HTTP 요청을 흘려보낸다.
`#[sqlx::test]`는 테스트마다 격리된 DB를 만들고 마이그레이션을 적용한 뒤, 끝나면 지운다.
따라서 접속 계정에 DB 생성 권한이 있어야 한다.

### 권한·보안 회귀 테스트 (`authz_tests.rs`)

- 일반 회원이 관리자 변경 POST 9개를 호출하면 거부되고 DB가 그대로인지 확인한다(관리자는 정상 동작).
- TOTP 계정이 `/auth/login`으로 로그인하면 대기 세션만 받는지 확인한다.
- 로그아웃: GET이나 잘못된 CSRF로는 세션이 유지되는지 확인한다.
- 이름만 `session`으로 바꾼 쿠키가 거부되는지, 에디터 업로드 기록·회원 제한·관리자 예외가 동작하는지 확인한다.

## 2. 로컬 실행

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo audit                       # 예외는 .cargo/audit.toml (근거 주석 필수)

# DB 회귀 테스트: 운영 DB 대신 임시 컨테이너 사용
docker run -d --rm --name rk-test-pg -e POSTGRES_PASSWORD=postgres \
  -p 127.0.0.1:55435:5432 postgres:16-alpine
cargo build && topcoat asset bundle
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55435/postgres \
  UPLOAD_DIR=/tmp/rk-test-uploads cargo test -- --include-ignored
docker stop rk-test-pg
```

## 3. CI (`.github/workflows/ci.yml`)

`main`/`develop`에 push하거나 `main` 대상 PR을 올리면 PostgreSQL 16 서비스 컨테이너와 함께 아래 순서로 실행된다.

1. 포맷 검사
2. clippy(경고 시 실패)
3. `cargo audit`
4. topcoat-cli 설치
5. `cargo build` → 자산 번들 → `cargo test -- --include-ignored`
6. release 빌드와 release 자산 번들
7. 스모크 테스트: 바이너리를 띄워 `/healthz`, robots, sitemap, manifest, sw.js, assetlinks 확인

## 4. 남은 과제

- E2E(브라우저) 테스트가 없다. 모바일 메뉴 dialog, 에디터 업로드 같은 클라이언트 동작은 수동으로 확인한다.
- 성능 목표(p95 < 200ms)를 측정하는 자동 부하 테스트가 없다.
