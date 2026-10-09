-- Security hardening: server-side sessions, rate limits, view dedup, search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Server-side sessions: token_hash = SHA256(session token), never the raw token
CREATE TABLE IF NOT EXISTS sessions (
    token_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    csrf_token TEXT NOT NULL DEFAULT '',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);

-- Login rate limiting (per IP + identifier)
CREATE TABLE IF NOT EXISTS login_attempts (
    id BIGSERIAL PRIMARY KEY,
    ip TEXT NOT NULL DEFAULT '',
    identifier TEXT NOT NULL DEFAULT '',
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_login_attempts_ip_time ON login_attempts(ip, attempted_at DESC);

-- Post view dedup (one count per viewer hash per 24h)
CREATE TABLE IF NOT EXISTS post_views (
    post_id TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    viewer_hash TEXT NOT NULL,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, viewer_hash)
);

-- Trigram search indexes (pg_trgm)
CREATE INDEX IF NOT EXISTS idx_manual_series_title_trgm ON manual_series USING gin (title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_posts_title_trgm ON posts USING gin (title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_youtube_title_trgm ON youtube_videos USING gin (title gin_trgm_ops);
