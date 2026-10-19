# 운영 런북 — 백업/복구/배포

## 백업 (매일 크론 권장)

```bash
# 호스트에서 실행 (컨테이너 이름 확인: docker ps)
docker exec rustkorea-db-1 pg_dump -U postgres rustkorea \
  | gzip > /backup/rustkorea-$(date +%F).sql.gz

# 7일 이상 보관 + 주 1회 복구 테스트(스테이징 DB에 restore 후 /healthz 확인)
```

## 복구

```bash
gunzip -c /backup/rustkorea-<DATE>.sql.gz \
  | docker exec -i rustkorea-db-1 psql -U postgres rustkorea
```

## 배포 (CloudPanel 리버스 프록시 — 현재 표준)

TLS는 CloudPanel nginx가 종단(80/443 점유)하므로 Caddy 오버레이는 쓰지 않는다.

```bash
cp .env.example .env   # 작성 후 chmod 600 .env (POSTGRES_PASSWORD 필수)
docker compose -f docker-compose.yml -f docker-compose.cloudpanel.yml up -d --build
docker compose logs -f app            # 마이그레이션/시드 확인
curl -sf http://127.0.0.1:3005/healthz  # 200 {"status":"ok"}
```

CloudPanel UI (CLI 미지원 항목, 수동 1회): Sites > Add Site > Reverse Proxy:
domain `rustkorea.kr` + alias `www.rustkorea.kr` → target
`http://127.0.0.1:3005`, SSL(Let's Encrypt) + Force HTTPS ON.
이후 `curl -sf https://rustkorea.kr/healthz` 확인.

### 상품 이미지 업로드 확인

- CloudPanel의 rustkorea.kr nginx vhost에 `client_max_body_size 6m;` 이상을 설정한다.
  앱은 파일 5MiB, multipart 요청 전체 6MiB까지 허용한다.
- 앱 컨테이너 사용자 `appuser`가 `/app/data/uploads`에 쓸 수 있어야 하며,
  `uploads_data` 볼륨을 배포·재시작 시 유지한다.
- 썸네일과 메인 이미지를 각각 업로드한 뒤 `변경 사항 저장`을 누르고 새로고침한다.
  `/uploads/...` URL과 미리보기가 유지되고 공개 상품 화면에서 표시되는지 확인한다.
- 이미 지워진 이미지 URL은 수정 배포 후 재업로드한다.

DB까지 포함하는 회귀 테스트는 운영 DB 대신 개발 PostgreSQL을 사용한다.
SQLx가 별도 테스트 DB를 생성하고 성공 시 삭제하므로 DB 생성 권한이 필요하다.

```bash
cargo build
topcoat asset bundle
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5435/rustkorea \
  UPLOAD_DIR=/tmp/rustkorea-product-upload-regression \
  cargo test uploaded_product_images_survive_save_and_are_served -- --ignored
```

## 배포 (Caddy 직접 종단 — CloudPanel 미사용 호스트 전용)

```bash
cp .env.example .env   # 작성 후 chmod 600 .env
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
docker compose logs -f app            # 마이그레이션/시드 확인
curl -sf https://<도메인>/healthz     # 200 {"status":"ok"}
```

배포 전 체크리스트:

- [ ] `POSTGRES_PASSWORD`가 기본값이 아님
- [ ] `SITE_URL`이 실제 HTTPS 도메인
- [ ] `SECURE_COOKIES=1`, `OAUTH_SECURE_COOKIES=1`
- [ ] `ADMIN_PASSWORD`/`TOTP_RESET_EMAIL` 제거됨 (시드 경고 로그 확인)
- [ ] 백업이 24시간 이내에 성공함

## 메뉴 관리

`/admin/menus`에서 상단 메뉴, 푸터 메뉴, 모바일 하단 탭을 각각 관리한다.
상단 메뉴는 PC 내비게이션과 모바일 전체 메뉴에 함께 반영된다.
푸터 메뉴는 서비스·링크·참여 그룹으로 묶어 PC와 모바일에 출력한다.
모바일 하단 탭은 항목이 많으면 가로 스크롤되며 로그인·프로필 버튼은 자동 표시된다.
표시 여부와 필요 권한은 모든 공개 메뉴 영역에 동일하게 적용한다.

마이그레이션 017은 기존 메뉴를 보존하고, 비어 있는 푸터와 신규 모바일 탭에
기본값을 한 번 등록한다. 이후 삭제·숨김 설정은 앱 재시작으로 복원되지 않는다.
배포 전 DB 백업과 이전 앱 이미지 태그를 확보한다. 이전 앱으로 롤백해도
추가된 컬럼은 호환되므로 마이그레이션을 되돌릴 필요가 없다.

메뉴 회귀 테스트는 위와 같이 자산을 빌드한 뒤 개발 PostgreSQL에서 실행한다.

```bash
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5435/rustkorea \
  cargo test services::navigation -- --include-ignored
```

## 보안 설정

- 클라이언트 IP(로그인 제한·감사 로그)는 `TRUSTED_PROXIES`에 속한 프록시가 보낸
  `X-Forwarded-For`만 신뢰하며 오른쪽부터 읽는다. 기본값은 루프백과 사설 대역이다.
  앱 포트를 인터넷에 직접 공개하지 않는다(compose는 127.0.0.1 또는 내부망만 사용).
  CloudPanel nginx vhost에 `proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;`가
  있는지 확인한다. 값을 바꾸려면 compose `environment:`에 `TRUSTED_PROXIES`를 추가한다.
- 2FA가 켜진 계정은 `/auth/login`, `/admin/login`, GitHub·Google 로그인 모두
  `/admin/2fa`를 거쳐야 정식 세션이 발급된다.
- 로그아웃은 POST + CSRF만 받는다(`/auth/logout`, `/admin/logout`). GET 링크로는 로그아웃되지 않는다.
- `SECURE_COOKIES=1`이면 `__Host-session` 쿠키만 세션으로 인정한다. 모드를 바꾸면 기존 세션은 재로그인이 필요하다.
- 블로그 "가져오기"는 루프백·사설·링크로컬·메타데이터 주소(169.254.169.254 등)로의 요청을 거부하고
  감사 로그(`admin.blog.fetch_blocked`)에 남긴다. 리다이렉트는 최대 3회, 단계마다 다시 검사한다.
- 에디터 이미지 업로드(`/admin/uploads`)는 `uploads` 테이블에 기록되며, 관리자 콘솔 권한이 없는
  회원은 시간당 30회로 제한된다.

## 영어 버전 (다국어)

- 공개 사이트는 한국어(기본)와 영어를 지원한다. 관리자 화면은 한국어만 쓴다.
- 언어 결정 순서: `?lang=ko|en` → `locale` 쿠키 → 한국어. 브라우저 언어(Accept-Language)는
  쓰지 않으므로 `?lang` 없는 URL은 검색엔진에 항상 한국어로 보인다. 영어 페이지는 `?lang=en`이며
  모든 공개 페이지에 hreflang(ko/en/x-default)이 붙는다.
- 화면 고정 문구는 코드(`loc.pick("한국어", "English")`)에 있다. DB 콘텐츠는 `_en` 칸이 비어 있으면
  한국어 원문을 보여 주며, 매뉴얼 챕터·블로그 본문은 "아직 번역되지 않음" 안내가 함께 표시된다.
- 영어 내용 입력 위치:
  - 메뉴: `/admin/menus` 각 항목의 English 칸
  - 카테고리: `/admin/categories` 영어 이름·설명
  - 사이트 제목·설명·푸터·배너: `/admin/settings` "영어 버전 (English)" 섹션
  - 매뉴얼 시리즈·챕터: 생성·수정 화면의 영어 제목·설명·본문(Markdown)
  - 블로그: 작성·수정 화면의 "영어 버전" 펼침 영역
- 마이그레이션 018은 영어 칸만 추가(기본 메뉴·카테고리 영어 이름 채움)하므로 이전 버전으로 롤백해도
  DB 변경 없이 동작한다. 커뮤니티 글·댓글, 알림 문구, RSS 피드, 상품(쇼케이스) 본문은 번역 대상이 아니다.

## 백업 자동화 (운영 서버)

- `~/backups/rustkorea/backup.sh`가 crontab으로 매일 04:00 실행된다. 덤프 완료 표시를 확인한 파일만
  남기고 14일 지난 `*-daily.sql.gz`는 지운다. 결과는 `~/backups/rustkorea/backup.log`에 남는다.
- 배포 전에는 같은 스크립트를 수동 실행하고, 현재 이미지를 `rustkorea-app:before-<작업>-<날짜>`로 태그한다.

권한·2FA 회귀 테스트(개발 PostgreSQL, 자산 빌드 후):

```bash
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5435/rustkorea \
  cargo test authz_tests -- --include-ignored
```

## 비밀키 교체

- DB: `ALTER USER postgres PASSWORD '...'` + `.env` + `docker compose up -d`
- OAuth: 공급자 콘솔에서 롤 후 `.env` 교체 + 재시작
- 관리자 비번: `/admin/profile`에서 변경 (전 세션 파기됨)
- 2FA 분실: `TOTP_RESET_EMAIL=<이메일>` 설정 후 재시작 → 로그인 → **즉시 변수 제거 후 재시작**
