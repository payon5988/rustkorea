UPDATE categories SET write_permission = 'admin' WHERE slug = 'announce';
UPDATE categories SET write_permission = 'editor' WHERE slug IN ('jobs');
