
## 📄 4. ARCHITECT.md (시스템 아키텍처) - 심화판

- 마크다운 렌더링 시 `ammonia` 크레이트로 XSS 방지
ARCHITECT: Rust Korea Hub
1. C4 모델 - 컨테이너 다이어그램
[사용자 (Browser)] 
       │ HTTPS (Cloudflare CDN)
       ▼
[Reverse Proxy: Nginx] 
  ├─ TLS Termination (Let's Encrypt)
  ├─ 정적 자산(/_topcoat/) → 직접 서빙 또는 CDN 캐시 히트
  └─ 동적 요청(/) → Topcoat Web Server
       │
       ▼
[Topcoat Web Server (Rust)]
  ├─ SSR 렌더링 엔진 (view! macro)
  ├─ API 라우터 (Axum 호환 계층)
  └─ Background Worker (tokio::spawn)
       │
       ├─ [PostgreSQL 16] (주 데이터: 사용자, 매뉴얼, 진행률)
       ├─ [Redis 7] (세션 저장소, YouTube API 응답 캐시 TTL 3600s)
       └─ [Meilisearch 1.7] (매뉴얼/게시글 전체 텍스트 검색)

2. 보안 아키텍처 (Security Posture)
XSS 방지: 마크다운 렌더링 파이프라인에 ammonia 크레이트를 사용하여 허용된 태그(p, code, pre, a 등)만 화이트리스트 필터링.
CSRF 방지: Topcoat 내장 세션 관리 활용, 민감한 POST 요청에는 숨겨진 CSRF 토큰 검증.
Rate Limiting: tower-http의 RateLimitLayer를 사용하여 로그인/게시글 작성 API에 IP당 분당 10회 제한 적용.
CSP (Content Security Policy):
default-src 'self'; script-src 'self' 'unsafe-inline' https://www.youtube.com; style-src 'self' 'unsafe-inline';
3. 확장성 및 내결함성
Stateless Design: Topcoat 서버는 상태를 가지지 않음 (세션은 Redis, 데이터는 Postgres). 수평 확장(Horizontal Scaling) 시 Nginx Round-Robin으로 트래픽 분산 가능.
Graceful Degradation: Meilisearch 다운 시, PostgreSQL의 pg_trgm 기반 LIKE 쿼리로 폴백하여 검색 기능 유지.

