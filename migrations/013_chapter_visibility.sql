-- Per-chapter visibility tiers (public/members/staff/premium).
-- Effective gate for chapter content is the chapter's own tier, so a series
-- can mix e.g. public previews with members/staff-only chapters.
-- Backfill existing chapters from their series tier.
ALTER TABLE manual_chapters ADD COLUMN visibility TEXT NOT NULL DEFAULT 'public'
    CHECK (visibility IN ('public', 'members', 'staff', 'premium'));
UPDATE manual_chapters mc SET visibility = ms.visibility
    FROM manual_series ms WHERE mc.series_id = ms.id;
CREATE INDEX IF NOT EXISTS idx_manual_chapters_visibility ON manual_chapters(visibility);
