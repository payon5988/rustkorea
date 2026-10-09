-- site settings
CREATE TABLE IF NOT EXISTS site_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

-- nav items
CREATE TABLE IF NOT EXISTS nav_items (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    visible INTEGER NOT NULL DEFAULT 1,
    section TEXT NOT NULL DEFAULT 'header' CHECK (section IN ('header', 'footer')),
    created_at TEXT NOT NULL DEFAULT (NOW()::text),
    updated_at TEXT NOT NULL DEFAULT (NOW()::text)
);

CREATE INDEX IF NOT EXISTS idx_nav_items_section ON nav_items(section, position);
