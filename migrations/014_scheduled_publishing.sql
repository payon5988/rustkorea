-- Scheduled publishing: NULL = published immediately (existing rows keep
-- working untouched). A row is publicly visible only when published_at
-- IS NULL or not in the future (compared as timestamptz at read time,
-- so no worker/cron is needed for timed release).
ALTER TABLE posts ADD COLUMN published_at TEXT NULL;
ALTER TABLE manual_series ADD COLUMN published_at TEXT NULL;
ALTER TABLE manual_chapters ADD COLUMN published_at TEXT NULL;
CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(published_at);
CREATE INDEX IF NOT EXISTS idx_manual_series_published ON manual_series(published_at);
CREATE INDEX IF NOT EXISTS idx_manual_chapters_published ON manual_chapters(published_at);
