-- voltrus-licenses (fdbe7ce9-aa43-479a-b026-26cd2f2d2bb9): the full current schema, for a fresh database.
-- The deployed database was created before the per-site columns: bring it up to date with
-- `wrangler d1 migrations apply voltrus-licenses --remote` (migrations/0001_per_site_support.sql); do not
-- run this file's CREATE TABLE against it expecting the columns to appear (IF NOT EXISTS skips it).
CREATE TABLE IF NOT EXISTS license_bindings (
  license_key TEXT PRIMARY KEY,
  fingerprint_hash TEXT NOT NULL,
  activated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  revoked INTEGER DEFAULT 0,
  tier TEXT DEFAULT 'professional',
  device_limit INTEGER DEFAULT NULL,
  -- SCADA-4 per-site licences. Cache of what the worker last signed for this Polar key; the console is
  -- the source of truth (see below). NULL = no site / no support contract (grandfathered key).
  site_id TEXT,
  site_name TEXT,
  support_until INTEGER  -- Unix seconds
);
CREATE INDEX IF NOT EXISTS idx_license_bindings_site ON license_bindings(site_id);

CREATE TABLE IF NOT EXISTS email_sequence (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT NOT NULL,
  product_name TEXT NOT NULL,
  purchased_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  sent_day0 INTEGER DEFAULT 0,
  sent_day3 INTEGER DEFAULT 0,
  sent_day10 INTEGER DEFAULT 0,
  unsubscribed INTEGER DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_email_sequence_due ON email_sequence(unsubscribed, purchased_at);

-- Which signing key produced the latest activation of each licence (binding or console seat).
-- A licence with no row here was last signed before the 2026-10 rotation, i.e. with the old key.
CREATE TABLE IF NOT EXISTS activation_signings (
  license_key TEXT PRIMARY KEY,
  key_id TEXT NOT NULL,
  signed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- CONSOLE_DB contract (voltrus-console D1, migration 0055_scada_per_site_support.sql). Read-mostly from this
-- worker; the tables live in the console database, not in this one:
--
--   scada_sites   (id TEXT PK, license_code TEXT UNIQUE, name TEXT, source TEXT, created_at INTEGER ms, updated_at INTEGER ms)
--   scada_support (license_code TEXT PK, support_until INTEGER ms, source TEXT, ...)
--
-- license_code is the key exactly as the host sends it: the upper-cased VLT-* code of a console licence, or
-- the Polar licence key string. Reads: src/site-support.ts (resolveSiteSupport). The worker's only write to
-- the console for this feature is `INSERT OR IGNORE INTO scada_sites ... source = 'activation'`, when an
-- activation names a site for a key that has none (or when a binding already carried a site the console lacks).
-- support_until is converted to Unix seconds (floor) for the signed activation. A key without a scada_support
-- row has no contract: the field is omitted from the signed payload.
