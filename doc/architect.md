# ARCHITECT: Rust Korea Hub (현행)

초기 기획(Redis 세션·YouTube 캐시, Meilisearch, tower-http RateLimit)은 채택하지 않았다.
현재는 **앱 1개 + PostgreSQL 1개**로 동작하며, 세션·캐시·검색·빈도 제한을 모두 PostgreSQL로 처리한다.

## 1. 컨테이너 구성

```
[브라우저 / Android TWA]
      │ HTTPS
      ▼
[CloudPanel nginx]  ← TLS(Let's Encrypt) 종단, 80/443 점유
      │ http://127.0.0.1:3005   (docker-compose.cloudpanel.yml)
      ▼
[app: Topcoat 0.10 SSR (Rust, tokio)]
  ├─ #[layout]/#[page]/#[route] 자동 등록(discover), #[shard] 부분 갱신(검색)
  ├─ 레이어: SecurityHeaders(CSP 등) · BodyLimit(/admin/products/upload 6MiB, 그 외 2MiB)
  ├─ TrustedProxies(TRUSTED_PROXIES) → client_ip
  └─ 백그라운드: YouTube RSS 워커(YOUTUBE_POLL_SECS, 기본 900초)
      │ 내부망 (compose)
      ▼
[db: PostgreSQL 16]  데이터·세션·로그인 시도·감사 로그·업로드 기록·검색(pg_trgm)
[volume uploads_data → /app/data/uploads]  상품/에디터 이미지
```

Caddy가 직접 TLS를 종단하는 구성(`docker-compose.prod.yml` + `Caddyfile`)은 CloudPanel을 쓰지 않는 호스트에서만 사용한다.

## 2. 모듈

| 경로 | 역할 |
|---|---|
| `src/main.rs` | DB 연결 → 마이그레이션 → 시드 → 워커 → 라우터 |
| `src/app.rs` | 루트 레이아웃, 메타·OG, 전역 CSS 토큰 |
| `src/auth/` | 세션(`session.rs`), 쿠키 저장소, RBAC(`access.rs`), GitHub·Google OAuth, TOTP |
| `src/services/` | 마크다운(ammonia), 메뉴(`navigation`), 업로드, 외부 요청(`outbound`), YouTube, 시드, 정적 라우트 |
| `src/pages/` | 공개 화면, `admin/` 관리자 콘솔 |
| `migrations/` | SQLx 마이그레이션(001~017), 앱 시작 시 자동 적용 |

## 3. 보안 구조

- **인증**: 서버측 세션(토큰은 SHA-256 해시만 저장)이다. 로그인할 때마다 토큰을 새로 발급하고, 30일 뒤 만료된다. 보안 모드에서는 `__Host-session` 쿠키만 인정한다.
- **2FA**: TOTP 계정은 모든 로그인 경로(`/auth/login`, `/admin/login`, OAuth)에서 대기 세션을 받고, `/admin/2fa`를 통과해야 정식 세션이 된다.
- **권한**: `Access::can(perm)`을 쓴다(superuser는 모두 통과). 관리자 화면과 변경 POST는 모두 `require_perm`/`require_admin_area`를 거친다.
- **CSRF**: 세션별 토큰을 쓰고, 익명 폼은 double-submit seed 쿠키를 쓴다. 로그아웃도 POST + CSRF다.
- **XSS**: 마크다운은 pulldown-cmark → ammonia 화이트리스트로 처리한다(링크 스킴 http/https/mailto, `rel=noopener noreferrer`).
- **SSRF**: 관리자가 URL을 입력해 가져오는 기능은 `services::outbound`만 쓴다. 이 모듈은 내부 대역을 거부하고, 검사한 주소로 연결을 고정하며, 리다이렉트를 단계마다 다시 검사한다.
- **빈도 제한**(PostgreSQL)
  - 로그인과 2FA: 5분에 10회, IP 또는 계정 기준
  - 상품 문의: IP 기준
  - 에디터 업로드: 회원별 시간당 30회
- **HTTP 헤더**: CSP, nosniff, SAMEORIGIN, Referrer-Policy, Permissions-Policy. 인라인 스크립트 때문에 `'unsafe-inline'`이 남아 있다.
- **감사 로그**: 관리자 로그인, CSRF 차단, 권한 변경, 차단된 외부 요청 등을 `audit_logs`에 남긴다.

## 4. 확장성·내결함성

- 앱은 상태를 갖지 않는다. 세션과 데이터는 모두 PostgreSQL에 있어서, 앱을 여러 개 띄우고 같은 DB를 쓰게 할 수 있다.
- 검색은 PostgreSQL `pg_trgm` 인덱스로 처리한다. 별도 검색 엔진이 없으므로 따로 장애 날 지점도 없다.
- **운영 안전망**
  - 매일 04:00 `pg_dump` 백업(14일 보관)
  - 배포 전 이미지를 태그하고, 롤백 스크립트로 되돌릴 수 있다(`doc/ops.md`).
