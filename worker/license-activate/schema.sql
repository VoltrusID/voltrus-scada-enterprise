-- voltrus-licenses (fdbe7ce9-aa43-479a-b026-26cd2f2d2bb9), as deployed.
CREATE TABLE IF NOT EXISTS license_bindings (
  license_key TEXT PRIMARY KEY,
  fingerprint_hash TEXT NOT NULL,
  activated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  revoked INTEGER DEFAULT 0,
  tier TEXT DEFAULT 'professional',
  device_limit INTEGER DEFAULT NULL
);

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
