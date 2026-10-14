-- Showcase (vibecoding) products + categories + gallery + inquiries.
-- Products reuse the published_at / visibility conventions (014/011):
-- NULL published_at = released immediately; visibility tiers identical.
CREATE TABLE product_categories (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT NOW()::text
);
CREATE TABLE products (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    category_id TEXT REFERENCES product_categories(id) ON DELETE SET NULL,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    description_md TEXT NOT NULL DEFAULT '',
    description_html TEXT NOT NULL DEFAULT '',
    price BIGINT NULL,
    price_visible INTEGER NOT NULL DEFAULT 1,
    preview_url TEXT NOT NULL DEFAULT '',
    video_id TEXT NOT NULL DEFAULT '',
    techs TEXT NOT NULL DEFAULT '',
    thumbnail_url TEXT NOT NULL DEFAULT '',
    main_image_url TEXT NOT NULL DEFAULT '',
    visibility TEXT NOT NULL DEFAULT 'public'
        CHECK (visibility IN ('public', 'members', 'staff', 'premium')),
    published_at TEXT NULL,
    created_at TEXT NOT NULL DEFAULT NOW()::text,
    updated_at TEXT NOT NULL DEFAULT NOW()::text
);
CREATE INDEX IF NOT EXISTS idx_products_slug ON products(slug);
CREATE INDEX IF NOT EXISTS idx_products_visibility ON products(visibility);
CREATE INDEX IF NOT EXISTS idx_products_published ON products(published_at);
CREATE TABLE product_images (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT NOW()::text
);
CREATE INDEX IF NOT EXISTS idx_product_images_product ON product_images(product_id);
-- Purchase inquiries (no PG): lead capture with a status workflow.
CREATE TABLE inquiries (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    product_id TEXT REFERENCES products(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    contact TEXT NOT NULL,
    message TEXT NOT NULL DEFAULT '',
    ip TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'new'
        CHECK (status IN ('new', 'read', 'replied')),
    created_at TEXT NOT NULL DEFAULT NOW()::text
);
CREATE INDEX IF NOT EXISTS idx_inquiries_status ON inquiries(status);
CREATE INDEX IF NOT EXISTS idx_inquiries_created ON inquiries(created_at DESC);
