CREATE TABLE IF NOT EXISTS license_bindings (
    license_key TEXT PRIMARY KEY,
    fingerprint_hash TEXT NOT NULL,
    activated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    customer_email TEXT,
    revoked INTEGER DEFAULT 0
);
