-- notifications
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    noti_type TEXT NOT NULL,
    from_user_id TEXT,
    post_id TEXT,
    comment_id TEXT,
    message TEXT NOT NULL,
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, is_read, created_at DESC);

-- messages
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    from_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subject TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);
CREATE INDEX IF NOT EXISTS idx_messages_from ON messages(from_user_id);
CREATE INDEX IF NOT EXISTS idx_messages_to ON messages(to_user_id, is_read, created_at DESC);

-- badges
CREATE TABLE IF NOT EXISTS badges (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon TEXT NOT NULL DEFAULT '',
    color TEXT NOT NULL DEFAULT '#6b7280',
    points_required INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);

CREATE TABLE IF NOT EXISTS user_badges (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_id TEXT NOT NULL REFERENCES badges(id) ON DELETE CASCADE,
    awarded_at TEXT NOT NULL DEFAULT (NOW()::text),
    PRIMARY KEY (user_id, badge_id)
);

-- point history
CREATE TABLE IF NOT EXISTS point_history (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    points INTEGER NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    reference_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);
CREATE INDEX IF NOT EXISTS idx_point_history_user ON point_history(user_id);

-- reports
CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    reporter_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    admin_note TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    resolved_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_reports_status ON reports(status, created_at DESC);

-- events
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    creator_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    location TEXT NOT NULL DEFAULT '',
    event_date TEXT NOT NULL,
    event_time TEXT NOT NULL DEFAULT '',
    max_participants INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);
CREATE INDEX IF NOT EXISTS idx_events_date ON events(event_date DESC);

CREATE TABLE IF NOT EXISTS event_registrations (
    event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    registered_at TEXT NOT NULL DEFAULT (NOW()::text),
    PRIMARY KEY (event_id, user_id)
);

-- uploads
CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    original_name TEXT NOT NULL,
    mime_type TEXT NOT NULL DEFAULT '',
    size INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);
CREATE INDEX IF NOT EXISTS idx_uploads_user ON uploads(user_id);

-- email subscriptions
CREATE TABLE IF NOT EXISTS email_subscriptions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    email TEXT NOT NULL,
    sub_type TEXT NOT NULL DEFAULT 'all',
    is_active INTEGER NOT NULL DEFAULT 1,
    token TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    UNIQUE(email, sub_type)
);

-- rss cache
CREATE TABLE IF NOT EXISTS rss_cache (
    id BIGSERIAL PRIMARY KEY,
    feed_type TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- dashboard indexes
CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comments_created ON comments(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_users_created ON users(created_at DESC);
