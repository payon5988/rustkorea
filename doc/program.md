# PROGRAM — 구현 현황 (2026-10-19 기준)

초기 12주 로드맵을 실제 구현 상태로 갱신했다. 처음 계획과 다르게 구현한 항목은 "→"로 적었다.

## 🟢 M1: 기반 구축
- [x] Topcoat 0.10 프로젝트, PostgreSQL + SQLx 마이그레이션(001~017)
- [x] GitHub·Google OAuth, 서버측 세션(`Cx` 기반), 이메일·비밀번호 가입
- [x] 공통 컴포넌트(헤더·푸터·카드·버튼) → `topcoat ui add` 대신 자체 CSS 클래스

## 🟡 M2: YouTube 연동
- [x] YouTube RSS 백그라운드 워커(기본 15분 주기, quick-xml)
- [x] 영상 목록·임베드, 매뉴얼 본문 `:::youtube[ID]` 임베드
- [ ] ~~Redis 캐시~~ → 채택하지 않음(PostgreSQL 저장으로 충분)
- [ ] `#[shard]`로 영상 목록 실시간 갱신(현재는 검색에만 shard 사용)

## 🟠 M3: 매뉴얼 시스템
- [x] 마크다운(pulldown-cmark + ammonia), 코드 강조 → syntect 대신 highlight.js
- [x] `/manuals/{slug}` 라우팅, 공개 범위(전체·회원·스태프·유료 준비중), 예약 발행
- [x] 읽기 진행률 저장, PDF·HTML 가져오기
- [ ] 매뉴얼 1호 시리즈 "Topcoat로 만드는 풀스택 Rust 웹개발" 전 12챕터 집필

## 🔵 M4: 커뮤니티 및 검색
- [x] 통합 검색(`pg_trgm` + shard) → Meilisearch는 채택하지 않음
- [x] 게시글 작성·수정·삭제(마크다운 에디터), 댓글, 태그, 좋아요, 북마크, 신고
- [x] 쪽지, 알림, 배지·포인트, 랭킹, 이벤트(참가 신청)

## 🟣 M5: 런칭 준비
- [x] 정적 자산 번들, SEO 메타·OG·`sitemap.xml`·RSS·robots
- [x] 배포: Docker Compose + CloudPanel nginx(TLS), 매일 DB 백업, 롤백 절차
- [x] PWA(오프라인·설치) + TWA 서버 측 준비(assetlinks)
- [ ] TWA 앱 빌드·Play 등록(`doc/twa.md` 2~4단계)
- [ ] 베타 테스터 20명 모집 및 1.0 공식 런칭

## ➕ 기획 외 추가 구현
- [x] RBAC(역할·권한), 관리자 2FA(모든 로그인 경로 강제), 감사 로그
- [x] 관리자 콘솔: 사용자·역할·카테고리·메뉴·사이트 설정·블로그·상품(쇼케이스)·문의
- [x] 블로그, 바이브코딩 쇼케이스(상품·이미지·문의)
- [x] 보안 강화: 관리자 POST 권한, SSRF 차단, 신뢰 프록시 IP, 업로드 제한, 로그아웃 CSRF

## 6. 예산 (월간 추정, 변동 없음)
- 도메인 (rustkorea.kr): ₩20,000/년
- VPS (4vCPU/8GB): ₩30,000/월
- NCP Object Storage: ₩5,000/월 (현재 미사용, 업로드는 서버 볼륨)
- **총계**: 약 ₩40,000/월
