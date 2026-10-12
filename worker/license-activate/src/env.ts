export interface Env {
  DB: D1Database;
  /** voltrus-console D1: console-issued VLT-* licenses and their device seats. */
  CONSOLE_DB: D1Database;
  /** voltrus-console LicenseTiers entrypoint: tier of a Polar key, resolved with the console's Polar token. */
  CONSOLE: Fetcher & { polarKeyTier(polarKeyId: string): Promise<string | null> };
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
