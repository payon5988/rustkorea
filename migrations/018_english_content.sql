-- English (public site) fields. Empty = show the Korean original.
-- Additive only: the previous app ignores these columns, so rollback needs no DB change.
ALTER TABLE nav_items ADD COLUMN label_en TEXT NOT NULL DEFAULT '';
ALTER TABLE categories
    ADD COLUMN name_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN description_en TEXT NOT NULL DEFAULT '';
ALTER TABLE manual_series
    ADD COLUMN title_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN description_en TEXT NOT NULL DEFAULT '';
ALTER TABLE manual_chapters
    ADD COLUMN title_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN content_md_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN content_html_en TEXT NOT NULL DEFAULT '';
ALTER TABLE blog_posts
    ADD COLUMN title_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN summary_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN content_md_en TEXT NOT NULL DEFAULT '',
    ADD COLUMN content_html_en TEXT NOT NULL DEFAULT '';

-- English labels for the default menu entries (installed by 017).
UPDATE nav_items SET label_en = CASE label
        WHEN '홈' THEN 'Home'
        WHEN '매뉴얼' THEN 'Manuals'
        WHEN '영상' THEN 'Videos'
        WHEN '커뮤니티' THEN 'Community'
        WHEN '블로그' THEN 'Blog'
        WHEN '검색' THEN 'Search'
        WHEN '바이브코딩' THEN 'Vibe Coding'
        WHEN '이벤트' THEN 'Events'
        WHEN '글쓰기' THEN 'Write a Post'
        WHEN '관리자' THEN 'Admin'
        WHEN '개인정보처리방침' THEN 'Privacy Policy'
        WHEN '이용약관' THEN 'Terms of Service'
        WHEN 'Rust 공식' THEN 'Rust Official'
        ELSE label_en END
WHERE label_en = '';

UPDATE categories SET name_en = CASE slug
        WHEN 'general' THEN 'General Discussion'
        WHEN 'qna' THEN 'Q&A'
        WHEN 'showcase' THEN 'Project Showcase'
        WHEN 'jobs' THEN 'Jobs'
        WHEN 'announce' THEN 'Announcements'
        ELSE name_en END
WHERE name_en = '';

-- English site settings (`<key>_en`; empty falls back to `<key>`).
INSERT INTO site_settings (key, value) VALUES
    ('site_title_en', ''),
    ('site_description_en', 'Build high-performance full-stack apps in 10 minutes, even without knowing Rust — Rust Korea'),
    ('footer_text_en', ''),
    ('banner_kicker_en', 'New Book'),
    ('banner_title_en', ''),
    ('banner_text_en', 'Demonstrating the hyper-productivity formula of the AI era with Rust and vibe coding.'),
    ('banner_cta_en', 'Read the philosophy')
ON CONFLICT (key) DO NOTHING;
