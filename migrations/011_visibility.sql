-- Content visibility tiers (public/members/staff/premium).
-- premium is dormant until the paid policy arrives; it requires the
-- `premium.access` permission which no role holds by default.
ALTER TABLE manual_series ADD COLUMN visibility TEXT NOT NULL DEFAULT 'public'
    CHECK (visibility IN ('public', 'members', 'staff', 'premium'));
CREATE INDEX IF NOT EXISTS idx_manual_series_visibility ON manual_series(visibility);
