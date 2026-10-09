-- OAuth settings (google_id already in 001 as TEXT)
INSERT INTO site_settings (key, value, updated_at) VALUES ('github_oauth_enabled', 'true', NOW())
ON CONFLICT (key) DO NOTHING;
INSERT INTO site_settings (key, value, updated_at) VALUES ('google_oauth_enabled', 'false', NOW())
ON CONFLICT (key) DO NOTHING;
