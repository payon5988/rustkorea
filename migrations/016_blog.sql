-- Blog (Rust news / OSS curation). Same publishing conventions:
-- NULL published_at = released immediately; visibility tiers identical.
CREATE TABLE blog_posts (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    content_md TEXT NOT NULL DEFAULT '',
    content_html TEXT NOT NULL DEFAULT '',
    thumbnail_url TEXT NOT NULL DEFAULT '',
    source_url TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '',
    visibility TEXT NOT NULL DEFAULT 'public'
        CHECK (visibility IN ('public', 'members', 'staff', 'premium')),
    published_at TEXT NULL,
    created_at TEXT NOT NULL DEFAULT NOW()::text,
    updated_at TEXT NOT NULL DEFAULT NOW()::text
);
CREATE INDEX IF NOT EXISTS idx_blog_posts_slug ON blog_posts(slug);
CREATE INDEX IF NOT EXISTS idx_blog_posts_published ON blog_posts(published_at);
