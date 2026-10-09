# TWA (Trusted Web Activity) runbook — Rust Korea Hub Android app

TWA wraps the PWA (`https://rustkorea.kr`) in a Play-store app without
writing native code. Server side is ready; the remaining steps need an
Android keystore + Play Console.

## 0. Prerequisites (server — done)

- `/.well-known/assetlinks.json` served from `TWA_PACKAGE_NAME` /
  `TWA_SHA256_FINGERPRINTS` (`src/services/static_routes.rs`)
- PWA installability: PNG icons 192/512 + maskable 512, `display:
  standalone`, `start_url: /`, `theme_color`, `/sw.js` + `/offline`
- Manifest shortcuts: Manuals, Community

## 1. HTTPS first (hard requirement)

Digital Asset Links are only honored over HTTPS. Finish TLS for
`rustkorea.cloud`/`rustkorea.kr` (Cloudflare Tunnel or Caddy + Let's
Encrypt) before step 2.

## 2. Keystore + fingerprints

```bash
# install bubblewrap
npm i -g @bubblewrap/cli

# generate upload key (or reuse the Play App Signing key)
keytool -genkeypair -v -keystore android.keystore -alias android \
  -keyalg RSA -keysize 2048 -validity 10000

# SHA-256 fingerprint for assetlinks
keytool -list -v -keystore android.keystore -alias android | grep SHA256
```

Set on the server and restart:

```bash
TWA_PACKAGE_NAME=kr.rustkorea.hub
TWA_SHA256_FINGERPRINTS=AA:BB:CC:...   # comma-separated if several
```

Verify: `curl https://rustkorea.kr/.well-known/assetlinks.json`

## 3. Build the app

```bash
bubblewrap init --manifest https://rustkorea.kr/twa-manifest.json
# or: bubblewrap build  (uses twa-manifest.json in repo root)
```

This produces `app-release-signed.apk` / `.aab`. Google's statement
checker validates the assetlinks file automatically.

## 4. Play Console

- Play App Signing enrolls its own upload key: after first upload, copy
  the **App signing key** SHA-256 from Setup > App integrity into
  `TWA_SHA256_FINGERPRINTS` (keep both fingerprints during rotation).
- Required store assets: feature graphic 1024x500, icon 512, 2+
  screenshots, privacy policy URL (`/privacy`), content rating
  questionnaire (everyone), data safety form (no data collected beyond
  account email).

## 5. Updates

PWA updates deploy server-side (no app update). Native bumps only for
`splashScreen*`, shortcuts, or `appVersionCode` changes. Keep
`twa-manifest.json` version in sync with `Cargo.toml`.
