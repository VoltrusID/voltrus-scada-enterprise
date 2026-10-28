// Per-site licences and annual support (SCADA-4).
//
// The console D1 (CONSOLE_DB, voltrus-console migration 0055) owns two tables this worker reads:
//   scada_sites    (id, license_code UNIQUE, name, ...)   the site a key is issued for; renamed on the console
//   scada_support  (license_code PRIMARY KEY, support_until ms, ...)   the support contract of that key
// `license_code` is the key exactly as the host sends it: the upper-cased VLT-* code of a console
// licence, or the Polar licence key string. A key with neither row is a grandfathered or plain key: no
// site, no support_until, and it activates as it always did.
//
// The signed activation carries `site_id`, `site_name` and `support_until` (Unix seconds) only when they
// exist AND the client says it can verify them (request.features has "site_support"): an older SCADA
// build cannot verify a file with fields it does not know, so it keeps receiving the old format.
// Lapsed support never refuses an activation or renewal and never shortens a licence.
import type { Env } from "./env";

export const FEATURE_SITE_SUPPORT = "site_support";
export const MAX_SITE_NAME_CHARS = 120;

export interface SiteInfo {
  id: string;
  name: string;
}

export interface SiteSupport {
  site: SiteInfo | null;
  /** End of the support contract, Unix seconds; null = no contract. */
  support_until: number | null;
}

/** What license_bindings remembers of the last activation of a Polar key. */
export interface StoredSiteSupport {
  site_id?: string | null;
  site_name?: string | null;
  support_until?: number | null;
}

/** Display name of a site as the worker signs it: no control or lone surrogate characters, trimmed, at most 120 characters. Null when nothing is left. */
export function cleanSiteName(raw: unknown): string | null {
  if (typeof raw !== "string") return null;
  const name = raw
    // eslint-disable-next-line no-control-regex
    .replace(/[\u0000-\u001f\u007f]+/g, " ")
    .replace(/[\ud800-\udbff](?![\udc00-\udfff])|(?<![\ud800-\udbff])[\udc00-\udfff]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  const capped = Array.from(name).slice(0, MAX_SITE_NAME_CHARS).join("").trim();
  return capped || null;
}

/** True when the client lists the capability to verify the per-site fields. */
export function siteAware(features: unknown): boolean {
  return Array.isArray(features) && features.includes(FEATURE_SITE_SUPPORT);
}

function seconds(ms: number | null | undefined): number | null {
  return typeof ms === "number" && Number.isFinite(ms) && ms > 0 ? Math.floor(ms / 1000) : null;
}

function storedSite(stored: StoredSiteSupport | null): SiteInfo | null {
  return stored?.site_id ? { id: stored.site_id, name: stored.site_name || stored.site_id } : null;
}

/**
 * Site and support end of a key, read from the console. The console is the source of truth, so a rename or a
 * paid renewal there reaches the next activation or renewal of the key.
 * - Console has a site: it wins (name included).
 * - Console has none but the key was already bound to one: that site is kept and written to the console, so
 *   it shows up and can be renamed there.
 * - Neither, and the request names a site: a new site is minted and written to the console. One key is one
 *   site: later requests cannot pick another site or id.
 * - The console cannot be read (not migrated yet, D1 down): fall back to what the binding remembers, mint
 *   nothing, and never fail the activation.
 */
export async function resolveSiteSupport(
  env: Env,
  licenseCode: string,
  stored: StoredSiteSupport | null,
  requestedName: string | null,
): Promise<SiteSupport> {
  let row: { site_id: string | null; site_name: string | null; support_until: number | null } | null;
  try {
    row = await env.CONSOLE_DB.prepare(
      "SELECT (SELECT id FROM scada_sites WHERE license_code = ?1) AS site_id, " +
        "(SELECT name FROM scada_sites WHERE license_code = ?1) AS site_name, " +
        "(SELECT support_until FROM scada_support WHERE license_code = ?1) AS support_until"
    ).bind(licenseCode).first();
  } catch (e) {
    console.error(`console site lookup failed for ${licenseCode}: ${e instanceof Error ? e.message : String(e)}`);
    return { site: storedSite(stored), support_until: stored?.support_until ?? null };
  }
  const support_until = seconds(row?.support_until);
  if (row?.site_id) {
    return { site: { id: row.site_id, name: row.site_name || row.site_id }, support_until };
  }
  const known = storedSite(stored);
  const candidate = known ?? (requestedName ? { id: crypto.randomUUID(), name: requestedName } : null);
  if (!candidate) return { site: null, support_until };
  try {
    const now = Date.now();
    await env.CONSOLE_DB.prepare(
      "INSERT OR IGNORE INTO scada_sites (id, license_code, name, source, created_at, updated_at) VALUES (?, ?, ?, 'activation', ?, ?)"
    ).bind(candidate.id, licenseCode, candidate.name, now, now).run();
    // A concurrent activation may have created the site first: the console row decides.
    const won = await env.CONSOLE_DB.prepare("SELECT id, name FROM scada_sites WHERE license_code = ?")
      .bind(licenseCode)
      .first<{ id: string; name: string }>();
    return { site: won ? { id: won.id, name: won.name } : null, support_until };
  } catch (e) {
    console.error(`console site write failed for ${licenseCode}: ${e instanceof Error ? e.message : String(e)}`);
    // A site the binding already carried keeps being signed; a freshly minted one is not signed unseen.
    return { site: known, support_until };
  }
}
