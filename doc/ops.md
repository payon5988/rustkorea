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

## 배포

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

## 비밀키 교체

- DB: `ALTER USER postgres PASSWORD '...'` + `.env` + `docker compose up -d`
- OAuth: 공급자 콘솔에서 롤 후 `.env` 교체 + 재시작
- 관리자 비번: `/admin/profile`에서 변경 (전 세션 파기됨)
- 2FA 분실: `TOTP_RESET_EMAIL=<이메일>` 설정 후 재시작 → 로그인 → **즉시 변수 제거 후 재시작**
