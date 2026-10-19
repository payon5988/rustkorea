ALTER TABLE nav_items DROP CONSTRAINT nav_items_section_check;
ALTER TABLE nav_items ADD CONSTRAINT nav_items_section_check
    CHECK (section IN ('header', 'footer', 'mobile'));
ALTER TABLE nav_items ADD COLUMN footer_group TEXT NOT NULL DEFAULT 'service'
    CHECK (footer_group IN ('service', 'links', 'participate'));

-- Defaults are installed once by this migration, never on application restart.
INSERT INTO nav_items (label, href, position, section, footer_group)
SELECT label, href, position, 'footer', footer_group
FROM (VALUES
    ('매뉴얼', '/manuals', 0, 'service'),
    ('영상', '/youtube', 1, 'service'),
    ('커뮤니티', '/community', 2, 'service'),
    ('블로그', '/blog', 3, 'service'),
    ('검색', '/search', 4, 'service'),
    ('Rust 공식', 'https://www.rust-lang.org', 5, 'links'),
    ('GitHub', 'https://github.com/rust-lang', 6, 'links'),
    ('Topcoat Docs', 'https://docs.rs/topcoat', 7, 'links'),
    ('글쓰기', '/community/new', 8, 'participate'),
    ('관리자', '/admin', 9, 'participate'),
    ('개인정보처리방침', '/privacy', 10, 'participate'),
    ('이용약관', '/terms', 11, 'participate')
) AS defaults(label, href, position, footer_group)
WHERE NOT EXISTS (SELECT 1 FROM nav_items WHERE section = 'footer');

INSERT INTO nav_items (label, href, position, section)
VALUES ('홈', '/', 0, 'mobile'),
       ('매뉴얼', '/manuals', 1, 'mobile'),
       ('커뮤니티', '/community', 2, 'mobile'),
       ('블로그', '/blog', 3, 'mobile'),
       ('검색', '/search', 4, 'mobile');

-- Move first-install header defaults out of the repeated seed routine too.
INSERT INTO nav_items (label, href, position, section)
SELECT label, href, position, 'header'
FROM (VALUES ('매뉴얼', '/manuals', 0), ('영상', '/youtube', 1),
    ('커뮤니티', '/community', 2), ('블로그', '/blog', 3), ('검색', '/search', 4)
) AS defaults(label, href, position)
WHERE NOT EXISTS (SELECT 1 FROM nav_items WHERE section = 'header');
