-- SCADA-4: per-site licences and annual support (voltrus-licenses D1, binding DB).
--
-- Additive and nullable: bindings made before this migration keep NULL here, which means "no site, no
-- support contract" (grandfathered lifetime keys), and the previous worker version ignores the columns.
-- Apply this BEFORE deploying the worker that writes them:
--   bunx wrangler d1 migrations apply voltrus-licenses --remote
--
-- The columns cache what the worker last signed for a Polar licence key; the voltrus-console D1
-- (CONSOLE_DB: scada_sites, scada_support) is the source of truth, and every activation or renewal re-reads it.
ALTER TABLE license_bindings ADD COLUMN site_id TEXT;           -- issuer-side site UUID (scada_sites.id)
ALTER TABLE license_bindings ADD COLUMN site_name TEXT;         -- display name last signed
ALTER TABLE license_bindings ADD COLUMN support_until INTEGER;  -- Unix seconds of the support contract end; NULL = none

CREATE INDEX IF NOT EXISTS idx_license_bindings_site ON license_bindings(site_id);
