-- RBAC: roles, permissions, user status, approval settings.

CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    is_system INTEGER NOT NULL DEFAULT 0,
    is_superuser INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);

CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL DEFAULT 'general',
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (NOW()::text)
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- System roles (ids double as stable keys).
INSERT INTO roles (id, name, description, is_system, is_superuser, position) VALUES
    ('admin', 'admin', '최고 관리자 (모든 권한)', 1, 1, 0),
    ('editor', 'editor', '편집자 (콘텐츠 관리)', 1, 0, 1),
    ('member', 'member', '일반 회원', 1, 0, 2)
ON CONFLICT (id) DO NOTHING;

-- Permission catalog.
INSERT INTO permissions (id, key, category, description) VALUES
    ('manuals.view_staff', 'manuals.view_staff', 'manuals', '스태프 전용 매뉴얼 열람'),
    ('manuals.manage', 'manuals.manage', 'manuals', '매뉴얼 시리즈/챕터 관리'),
    ('videos.view_staff', 'videos.view_staff', 'videos', '스태프 전용 영상 열람'),
    ('youtube.manage', 'youtube.manage', 'videos', 'YouTube 동기화/영상 관리'),
    ('posts.moderate', 'posts.moderate', 'community', '게시글/댓글 중재 (삭제·블라인드)'),
    ('community.manage', 'community.manage', 'community', '카테고리/신고 관리'),
    ('users.manage', 'users.manage', 'users', '사용자 조회·수정·정지'),
    ('roles.manage', 'roles.manage', 'users', '역할/권한 매트릭스 관리'),
    ('events.manage', 'events.manage', 'events', '이벤트 관리'),
    ('settings.manage', 'settings.manage', 'settings', '사이트 설정 변경'),
    ('premium.access', 'premium.access', 'premium', '유료(프리미엄) 콘텐츠 접근 — 유료화 시 부여')
ON CONFLICT (id) DO NOTHING;

-- Default grants.
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'admin', p.id FROM permissions p
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id) VALUES
    ('editor', 'manuals.view_staff'),
    ('editor', 'manuals.manage'),
    ('editor', 'videos.view_staff'),
    ('editor', 'youtube.manage'),
    ('editor', 'posts.moderate'),
    ('editor', 'community.manage'),
    ('editor', 'events.manage')
ON CONFLICT DO NOTHING;

-- Users: role reference + lifecycle status.
ALTER TABLE users ADD COLUMN role_id TEXT REFERENCES roles(id);
UPDATE users SET role_id = CASE role
    WHEN 'admin' THEN 'admin'
    WHEN 'editor' THEN 'editor'
    ELSE 'member'
END;
ALTER TABLE users ALTER COLUMN role_id SET NOT NULL;
ALTER TABLE users ALTER COLUMN role_id SET DEFAULT 'member';
ALTER TABLE users DROP COLUMN role;

ALTER TABLE users ADD COLUMN status TEXT NOT NULL DEFAULT 'active'
    CHECK (status IN ('active', 'suspended', 'pending'));
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role_id);

-- Access-control settings.
INSERT INTO site_settings (key, value, updated_at) VALUES
    ('require_approval', 'false', NOW()::text),
    ('youtube_visibility', 'public', NOW()::text)
ON CONFLICT (key) DO NOTHING;
