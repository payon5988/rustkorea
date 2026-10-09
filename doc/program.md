# PROGRAM — 구현 로드맵 (12주)

## 🟢 M1: 기반 구축 (1~2주차)
- [ ] `cargo install topcoat-cli` 및 `topcoat new rust-korea-hub`
- [ ] PostgreSQL + SQLx 스키마 정의
- [ ] GitHub OAuth 연동 및 `Cx` 기반 세션 관리 구현
- [ ] `topcoat ui add`로 버튼, 카드, 입력창 등 기본 컴포넌트 세팅

## 🟡 M2: YouTube 연동 (3~4주차)
- [ ] YouTube RSS 파싱 백그라운드 워커 구현
- [ ] `#[shard]`를 활용한 영상 목록 실시간 업데이트 UI
- [ ] Redis 캐시 레이어 적용

## 🟠 M3: 매뉴얼 시스템 (5~7주차)
- [ ] 마크다운 파싱 (`pulldown-cmark` + `syntect`)
- [ ] 모듈 기반 라우팅(`/manuals/{slug}`) 구현
- [ ] **매뉴얼 1호 시리즈 집필**: "Topcoat로 만드는 풀스택 Rust 웹개발" (전 12챕터)

## 🔵 M4: 커뮤니티 및 검색 (8~9주차)
- [ ] Meilisearch 연동 및 통합 검색 바 구현
- [ ] 마크다운 에디터 기반 게시글 작성/수정/삭제
- [ ] 댓글 및 태그 시스템

## 🟣 M5: 런칭 준비 (10~12주차)
- [ ] `asset!` 매크로를 활용한 정적 자산 최적화
- [ ] SEO 메타태그 및 `sitemap.xml` 자동 생성
- [ ] VPS 배포 (Docker Compose + Cloudflare)
- [ ] 베타 테스터 20명 모집 및 1.0 공식 런칭

## 6. 예산 (월간 추정)
- 도메인 (rustkorea.kr): ₩20,000/년
- VPS (4vCPU/8GB): ₩30,000/월
- NCP Object Storage: ₩5,000/월
- **총계**: 약 ₩40,000/월 (초기 비용 최소화)