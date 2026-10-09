-- Rust Korea Hub database schema (PostgreSQL)

-- users (email/password + GitHub/Google OAuth)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    email TEXT UNIQUE,
    password_hash TEXT,
    github_id BIGINT UNIQUE,
    google_id TEXT UNIQUE,
    username TEXT NOT NULL,
    display_name TEXT,
    avatar_url TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    bio TEXT NOT NULL DEFAULT '',
    theme_preference TEXT NOT NULL DEFAULT 'light',
    points INTEGER NOT NULL DEFAULT 0,
    locale TEXT NOT NULL DEFAULT 'ko',
    last_login_at TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- manual series
CREATE TABLE IF NOT EXISTS manual_series (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    thumbnail_url TEXT,
    difficulty TEXT NOT NULL DEFAULT 'beginner' CHECK (difficulty IN ('beginner', 'intermediate', 'advanced')),
    estimated_minutes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- manual chapters
CREATE TABLE IF NOT EXISTS manual_chapters (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    series_id TEXT NOT NULL REFERENCES manual_series(id) ON DELETE CASCADE,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    content_md TEXT NOT NULL DEFAULT '',
    content_html TEXT NOT NULL DEFAULT '',
    chapter_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text),
    UNIQUE(series_id, slug)
);

-- reading progress
CREATE TABLE IF NOT EXISTS reading_progress (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chapter_id TEXT NOT NULL REFERENCES manual_chapters(id) ON DELETE CASCADE,
    scroll_pct INTEGER NOT NULL DEFAULT 0,
    completed INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (NOW()::text),
    UNIQUE(user_id, chapter_id)
);

-- youtube videos
CREATE TABLE IF NOT EXISTS youtube_videos (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    video_id TEXT UNIQUE NOT NULL,
    channel_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    thumbnail_url TEXT NOT NULL DEFAULT '',
    published_at TEXT NOT NULL DEFAULT (NOW()::text),
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- community posts
CREATE TABLE IF NOT EXISTS posts (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content_md TEXT NOT NULL DEFAULT '',
    content_html TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    category TEXT NOT NULL DEFAULT 'general',
    view_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- comments
CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    post_id TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- indexes
CREATE INDEX IF NOT EXISTS idx_manual_chapters_series_id ON manual_chapters(series_id);
CREATE INDEX IF NOT EXISTS idx_reading_progress_user_id ON reading_progress(user_id);
CREATE INDEX IF NOT EXISTS idx_youtube_videos_published_at ON youtube_videos(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comments_post_id ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_posts_category ON posts(category);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
