export interface PolarKeyInfo {
  /** Tier of the purchase (products.tier); null when the purchase couldn't be resolved. */
  tier: string | null;
  /** Monthly keys: end of the paid subscription period (ISO). Lifetime and legacy keys: null. */
  expires_at: string | null;
  /** Monthly key: expires_at null then means the subscription could not be read. */
  recurring: boolean;
}

export interface Env {
  DB: D1Database;
  /** voltrus-console D1: console-issued VLT-* licenses and their device seats. */
  CONSOLE_DB: D1Database;
  /** voltrus-console LicenseTiers entrypoint: tier and (monthly keys) paid period end of a Polar key, resolved with the console's Polar token. */
  CONSOLE: Fetcher & { polarKeyInfo(polarKeyId: string): Promise<PolarKeyInfo | null> };
  RELEASES_DB: D1Database;
  RELEASES_BUCKET: R2Bucket;
  EMAIL: SendEmail;
  ADMIN_TOKEN: string;
  ED25519_PRIVATE_KEY: string;
  POLAR_API_KEY: string;
  POLAR_ORG_ID: string;
  POLAR_WEBHOOK_SECRET: string;
  EMAIL_PUBLIC_BASE?: string;
}
