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

## 보안 설정

- 클라이언트 IP(로그인 제한·감사 로그)는 `TRUSTED_PROXIES`에 속한 프록시가 보낸
  `X-Forwarded-For`만 신뢰하며 오른쪽부터 읽는다. 기본값은 루프백과 사설 대역이다.
  앱 포트를 인터넷에 직접 공개하지 않는다(compose는 127.0.0.1 또는 내부망만 사용).
  CloudPanel nginx vhost에 `proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;`가
  있는지 확인한다. 값을 바꾸려면 compose `environment:`에 `TRUSTED_PROXIES`를 추가한다.
- 2FA가 켜진 계정은 `/auth/login`, `/admin/login`, GitHub·Google 로그인 모두
  `/admin/2fa`를 거쳐야 정식 세션이 발급된다.

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
