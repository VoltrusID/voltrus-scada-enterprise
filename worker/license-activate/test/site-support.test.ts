// SCADA-4: the signed `site_id`, `site_name` and `support_until` (from the console), grandfathered keys,
// the renewal that picks up a paid support renewal, and the wire format the SCADA binary verifies.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Env } from "../src/env";
import { cleanSiteName, resolveSiteSupport, siteAware } from "../src/site-support";
import {
  DAY,
  FP,
  KEY,
  fakeDb,
  openActivation,
  openActivationRaw,
  post,
  reply,
  setup,
  volreq,
  type Binding,
  type ConsoleSite,
  type FakeDb,
  type Harness,
} from "./helpers";

const SITE = { id: "0b0e8a5c-1f3a-4f6e-9d0a-5d1d8f4c7a11", name: "Plant 1" };
const FEATURES = { features: ["site_support"] };
const bound: Binding = { fingerprint_hash: FP, revoked: 0, tier: "integrator", device_limit: null };
const lifetime = { tier: "integrator", expires_at: null, recurring: false };

beforeEach(() => {
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(console, "error").mockImplementation(() => {});
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

const bindingWrite = (t: Harness) =>
  t.bindings.calls.find((c) => c.sql.startsWith("UPDATE license_bindings") || c.sql.startsWith("INSERT INTO license_bindings"))!;
/** [site_id, site_name, support_until] the worker wrote to license_bindings, for an INSERT or an UPDATE. */
const siteColumns = (t: Harness) => {
  const w = bindingWrite(t);
  return w.sql.startsWith("INSERT") ? w.args.slice(4, 7) : w.args.slice(2, 5);
};

describe("signed site and support", () => {
  it("a Polar key with a console site and support contract signs all three fields, support in Unix seconds", async () => {
    const support = Date.UTC(2027, 0, 1) + 999; // sub-second remainder is floored
    const t = setup({ info: lifetime, consoleSite: { site: SITE, support_until_ms: support } });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP, FEATURES)));
    expect(out).toMatchObject({ success: true, tier: "integrator", site: SITE, support_until: 1_798_761_600 });
    const opened = await openActivationRaw(out.activation_file!);
    expect(opened.payload).toMatchObject({ site_id: SITE.id, site_name: SITE.name, support_until: 1_798_761_600 });
    // The order Rust's ActivationPayload serialises in: the per-site fields trail device_limit.
    expect(opened.keys).toEqual([
      "version", "license_key", "fingerprint_hash", "issued_at", "expires_at", "tier", "device_limit",
      "site_id", "site_name", "support_until",
    ]);
    // The binding remembers what was signed.
    expect(bindingWrite(t).sql).toMatch(/^INSERT INTO license_bindings/);
    expect(bindingWrite(t).args).toEqual([KEY, FP, "integrator", null, SITE.id, SITE.name, 1_798_761_600]);
  });

  it("a key with no console rows signs exactly what it always did: no per-site fields", async () => {
    const t = setup({ info: lifetime, consoleSite: { site: null, support_until_ms: null } });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP, FEATURES)));
    expect(out).toMatchObject({ success: true, site: null, support_until: null });
    expect((await openActivationRaw(out.activation_file!)).keys).toEqual([
      "version", "license_key", "fingerprint_hash", "issued_at", "expires_at", "tier", "device_limit",
    ]);
    expect(bindingWrite(t).args).toEqual([KEY, FP, "integrator", null, null, null, null]);
  });

  it("a client that cannot verify the per-site fields keeps the old format; the console data is still recorded", async () => {
    const t = setup({ info: lifetime, consoleSite: { site: SITE, support_until_ms: Date.UTC(2027, 0, 1) } });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP)));
    expect(out.success).toBe(true);
    expect((await openActivationRaw(out.activation_file!)).keys).not.toContain("support_until");
    expect((await openActivationRaw(out.activation_file!)).keys).not.toContain("site_id");
    expect(siteColumns(t)).toEqual([SITE.id, SITE.name, 1_798_761_600]);
  });

  it("support contract without a site signs support_until only, and the reverse", async () => {
    const supportOnly = setup({ info: lifetime, consoleSite: { site: null, support_until_ms: Date.UTC(2027, 0, 1) } });
    const a = await openActivationRaw((await reply(await post(supportOnly.env, "/activate", volreq(KEY, FP, FEATURES)))).activation_file!);
    expect(a.keys.slice(-1)).toEqual(["support_until"]);
    expect(a.keys).not.toContain("site_id");

    const siteOnly = setup({ info: lifetime, consoleSite: { site: SITE, support_until_ms: null } });
    const b = await openActivationRaw((await reply(await post(siteOnly.env, "/activate", volreq(KEY, FP, FEATURES)))).activation_file!);
    expect(b.keys.slice(-2)).toEqual(["site_id", "site_name"]);
    expect(b.keys).not.toContain("support_until");
  });

  it("a lapsed support contract never refuses an activation or renewal and keeps the tier", async () => {
    const lapsed = Date.now() - 400 * DAY;
    const t = setup({ binding: bound, info: lifetime, consoleSite: { site: SITE, support_until_ms: lapsed } });
    for (const path of ["/activate", "/renew"] as const) {
      const res = await post(t.env, path, volreq(KEY, FP, FEATURES));
      expect(res.status).toBe(200);
      const out = await reply(res);
      expect(out).toMatchObject({ success: true, tier: "integrator", expires_at: null });
      expect((await openActivation(out.activation_file!)).support_until).toBe(Math.floor(lapsed / 1000));
    }
  });
});

describe("renewal picks up console changes without re-binding", () => {
  it("a paid support renewal reaches the next /renew of the bound device", async () => {
    const before = Date.now() + 5 * DAY;
    const after = Date.now() + 370 * DAY;
    const old = setup({ binding: { ...bound, site_id: SITE.id, site_name: SITE.name, support_until: Math.floor(before / 1000) }, info: lifetime, consoleSite: { site: SITE, support_until_ms: before } });
    expect((await openActivation((await reply(await post(old.env, "/renew", volreq(KEY, FP, FEATURES)))).activation_file!)).support_until).toBe(Math.floor(before / 1000));

    const renewed = setup({ binding: { ...bound, site_id: SITE.id, site_name: SITE.name, support_until: Math.floor(before / 1000) }, info: lifetime, consoleSite: { site: SITE, support_until_ms: after } });
    const out = await reply(await post(renewed.env, "/renew", volreq(KEY, FP, FEATURES)));
    expect(out.support_until).toBe(Math.floor(after / 1000));
    expect((await openActivation(out.activation_file!)).support_until).toBe(Math.floor(after / 1000));
    // Same machine, no new binding, no Polar usage spent.
    expect(renewed.bindings.calls.some((c) => c.sql.startsWith("INSERT INTO license_bindings"))).toBe(false);
    expect(renewed.polarBodies[0]).not.toHaveProperty("increment_usage");
    expect(siteColumns(renewed)).toEqual([SITE.id, SITE.name, Math.floor(after / 1000)]);
  });

  it("a rename on the console is signed on the next renewal", async () => {
    const t = setup({ binding: { ...bound, site_id: SITE.id, site_name: "Plant 1" }, info: lifetime, consoleSite: { site: { ...SITE, name: "Plant 1, Line A" }, support_until_ms: null } });
    const out = await openActivation((await reply(await post(t.env, "/renew", volreq(KEY, FP, FEATURES)))).activation_file!);
    expect(out).toMatchObject({ site_id: SITE.id, site_name: "Plant 1, Line A" });
  });

  it("a grandfathered lifetime key whose console has no support entitlement gets support_until NULL", async () => {
    // The binding remembered a contract the console no longer has: the console is the source of truth.
    const t = setup({ binding: { ...bound, support_until: 1_700_000_000 }, info: lifetime, consoleSite: { site: null, support_until_ms: null } });
    const out = await reply(await post(t.env, "/renew", volreq(KEY, FP, FEATURES)));
    expect((await openActivationRaw(out.activation_file!)).keys).not.toContain("support_until");
    expect(siteColumns(t)).toEqual([null, null, null]);
  });

  it("an unmigrated console falls back to what the binding remembers and never fails the activation", async () => {
    const t = setup({
      binding: { ...bound, site_id: SITE.id, site_name: SITE.name, support_until: 1_798_761_600 },
      info: lifetime,
      consoleSite: undefined,
    });
    const res = await post(t.env, "/renew", volreq(KEY, FP, FEATURES));
    expect(res.status).toBe(200);
    expect(await openActivation((await reply(res)).activation_file!)).toMatchObject({ site_id: SITE.id, support_until: 1_798_761_600 });
  });

  it("an unmigrated console with a fresh key signs no per-site fields and mints nothing", async () => {
    const t = setup({ info: lifetime, consoleSite: undefined });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP, { ...FEATURES, site_name: "Plant 1" })));
    expect(out).toMatchObject({ success: true, site: null });
    expect((await openActivationRaw(out.activation_file!)).keys).not.toContain("site_id");
  });
});

describe("naming a site at activation", () => {
  it("a key without a site gets one from the request, written to the console as source 'activation'", async () => {
    const t = setup({ info: lifetime, consoleSite: { site: null, support_until_ms: null } });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP, { ...FEATURES, site_name: "  North  plant\n" })));
    expect(out.site?.name).toBe("North plant");
    expect(out.site?.id).toMatch(/^[0-9a-f-]{36}$/);
    const insert = t.consoleDb.calls.find((c) => c.sql.startsWith("INSERT OR IGNORE INTO scada_sites"))!;
    expect(insert.sql).toContain("'activation'");
    expect(insert.args.slice(0, 3)).toEqual([out.site!.id, KEY, "North plant"]);
    expect(await openActivation(out.activation_file!)).toMatchObject({ site_id: out.site!.id, site_name: "North plant" });
  });

  it("a key that already has a site keeps it: the request cannot pick another site or rename", async () => {
    const t = setup({ info: lifetime, consoleSite: { site: SITE, support_until_ms: null } });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP, { ...FEATURES, site_name: "Somewhere else" })));
    expect(out.site).toEqual(SITE);
    expect(t.consoleDb.calls.some((c) => c.sql.startsWith("INSERT OR IGNORE INTO scada_sites"))).toBe(false);
  });

  it("a site the binding already carried is written to the console when the console lacks it", async () => {
    const t = setup({ binding: { ...bound, site_id: SITE.id, site_name: SITE.name }, info: lifetime, consoleSite: { site: null, support_until_ms: null } });
    const out = await reply(await post(t.env, "/renew", volreq(KEY, FP, FEATURES)));
    const insert = t.consoleDb.calls.find((c) => c.sql.startsWith("INSERT OR IGNORE INTO scada_sites"))!;
    expect(insert.args.slice(0, 3)).toEqual([SITE.id, KEY, SITE.name]);
    expect(out.site?.id).toBe(SITE.id);
  });

  it("a site name with non-ASCII characters survives the signed file (UTF-8, not UTF-16 length)", async () => {
    const t = setup({ info: lifetime, consoleSite: { site: { id: SITE.id, name: "Pabrik Bekasi \u00e9\u4e2d\u6587 \u{1F3ED}" }, support_until_ms: null } });
    const out = await reply(await post(t.env, "/activate", volreq(KEY, FP, FEATURES)));
    expect(out.success).toBe(true);
    expect((await openActivation(out.activation_file!)).site_name).toBe("Pabrik Bekasi \u00e9\u4e2d\u6587 \u{1F3ED}");
  });
});

describe("console VLT licences", () => {
  const code = "VLT-AAAAA-BBBBB-CCCCC";
  const lic = { id: "lic-1", status: "active", expires_at: null, seat_limit: 1, tier: "enterprise" };

  it("reads the site and support by the upper-cased code and signs them", async () => {
    const t = setup({ consoleLicense: lic, seated: true, consoleSite: { site: SITE, support_until_ms: Date.UTC(2027, 0, 1) } });
    const out = await reply(await post(t.env, "/renew", volreq(code.toLowerCase(), FP, FEATURES)));
    expect(out).toMatchObject({ success: true, tier: "enterprise", site: SITE, support_until: 1_798_761_600 });
    const lookup = t.consoleDb.calls.find((c) => c.sql.startsWith("SELECT (SELECT id FROM scada_sites"))!;
    expect(lookup.args).toEqual([code]);
    expect(await openActivation(out.activation_file!)).toMatchObject({ site_id: SITE.id, support_until: 1_798_761_600 });
  });

  it("a console licence without rows is signed as before", async () => {
    const t = setup({ consoleLicense: lic, seated: true, consoleSite: { site: null, support_until_ms: null } });
    const out = await reply(await post(t.env, "/activate", volreq(code, FP, FEATURES)));
    expect(out).toMatchObject({ success: true, site: null, support_until: null });
  });

  it("lapsed support does not refuse a VLT licence", async () => {
    const t = setup({ consoleLicense: lic, seated: true, consoleSite: { site: SITE, support_until_ms: Date.now() - 900 * DAY } });
    expect((await post(t.env, "/renew", volreq(code, FP, FEATURES))).status).toBe(200);
  });
});

describe("resolveSiteSupport", () => {
  const envWith = (console: FakeDb) => ({ CONSOLE_DB: console }) as unknown as Env;

  it("the console row wins when a concurrent activation minted the site first", async () => {
    const console = fakeDb({
      "SELECT (SELECT id FROM scada_sites": () => ({ site_id: null, site_name: null, support_until: null }),
      "INSERT OR IGNORE INTO scada_sites": () => null,
      "SELECT id, name FROM scada_sites": () => ({ id: "winner", name: "First writer" }),
    });
    const out = await resolveSiteSupport(envWith(console), KEY, null, "Second writer");
    expect(out.site).toEqual({ id: "winner", name: "First writer" });
  });

  it("a failed console write signs a site the binding already had, but not a freshly minted one", async () => {
    const failing = () =>
      fakeDb({
        "SELECT (SELECT id FROM scada_sites": () => ({ site_id: null, site_name: null, support_until: null }),
        "INSERT OR IGNORE INTO scada_sites": () => {
          throw new Error("D1 down");
        },
      });
    expect((await resolveSiteSupport(envWith(failing()), KEY, null, "New")).site).toBeNull();
    expect((await resolveSiteSupport(envWith(failing()), KEY, { site_id: "s1", site_name: "Old" }, null)).site).toEqual({ id: "s1", name: "Old" });
  });

  it("an unreadable console returns the stored values", async () => {
    const out = await resolveSiteSupport(envWith(fakeDb({})), KEY, { site_id: "s1", site_name: null, support_until: 42 }, "Ignored");
    expect(out).toEqual({ site: { id: "s1", name: "s1" }, support_until: 42 });
  });

  it("a missing or non-positive support_until is no contract", async () => {
    for (const ms of [null, 0, -5]) {
      const console = fakeDb({ "SELECT (SELECT id FROM scada_sites": () => ({ site_id: null, site_name: null, support_until: ms }) });
      expect((await resolveSiteSupport(envWith(console), KEY, null, null)).support_until).toBeNull();
    }
  });
});

describe("request parsing", () => {
  it("cleans the site name", () => {
    expect(cleanSiteName("  Plant\t1 \n")).toBe("Plant 1");
    expect(cleanSiteName("")).toBeNull();
    expect(cleanSiteName("   ")).toBeNull();
    expect(cleanSiteName(42)).toBeNull();
    expect(cleanSiteName("a\u0000b")).toBe("a b");
    expect(cleanSiteName("x".repeat(500))).toHaveLength(120);
    expect(cleanSiteName("lone \ud800 surrogate")).toBe("lone surrogate");
    expect(Array.from(cleanSiteName("\u{1F3ED}".repeat(200))!)).toHaveLength(120);
  });

  it("only the site_support feature opts in to the new fields", () => {
    expect(siteAware(["site_support"])).toBe(true);
    expect(siteAware(["other"])).toBe(false);
    expect(siteAware(undefined)).toBe(false);
    expect(siteAware("site_support")).toBe(false);
  });

  it("requests from older builds (no site_name, no features) still parse and activate", async () => {
    const t = setup({ info: lifetime, consoleSite: { site: null, support_until_ms: null } });
    const res = await post(t.env, "/activate", { license_key: KEY, fingerprint: { hash: FP } });
    expect(res.status).toBe(200);
  });
});

// A VOLACT file for a fixed request, clock and key (Ed25519 is deterministic). The same two files are
// verified by the SCADA binary in enterprise/src/licensing/activation.rs (VECTOR_*): if either side's
// serialisation changes, one of the two suites fails.
describe("wire vectors shared with the SCADA binary", () => {
  const NOW = new Date("2026-10-28T00:00:00.000Z");
  const issue = async (consoleSite: ConsoleSite | undefined, extra: Record<string, unknown>) => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(NOW);
    const t = setup({ info: lifetime, consoleSite });
    const out = await reply(await post(t.env, "/activate", volreq("VOLTRUS-VECTOR", "sha256:vector", extra)));
    return out.activation_file!;
  };

  it("issued without per-site fields", async () => {
    expect(await issue(undefined, FEATURES)).toBe(VECTOR_OLD_FORMAT);
  });

  it("issued with a site and a support contract", async () => {
    const site = { id: "0b0e8a5c-1f3a-4f6e-9d0a-5d1d8f4c7a11", name: "Plant \"1\" \u00e9" };
    expect(await issue({ site, support_until_ms: 1_798_761_600_000 }, FEATURES)).toBe(VECTOR_PER_SITE);
  });
});

const VECTOR_OLD_FORMAT =
  "Vk9MQUNUAXsidmVyc2lvbiI6MiwibGljZW5zZV9rZXkiOiJWT0xUUlVTLVZFQ1RPUiIsImZpbmdlcnByaW50X2hhc2giOiJzaGEyNTY6dmVjdG9yIiwiaXNzdWVkX2F0IjoiMjAyNi0xMC0yOFQwMDowMDowMC4wMDBaIiwiZXhwaXJlc19hdCI6bnVsbCwidGllciI6ImludGVncmF0b3IiLCJkZXZpY2VfbGltaXQiOm51bGwsInNpZ25hdHVyZSI6IjA3ZjUwMWNlYzk2MjU1ZTM5ZmZlOThmZWE1Y2ViOTNhZDczNGI0OWVlZTVmNDZhMGY3ZWM2Mjc3NWNlMDJlN2ZkODIwNWIwMzYxYmU1NjA2NTg2ZTExMDU3MTI0ZTdjNDE2MzdjYjQyMTNlOTBmZmIxMjZiMGY0YmIyNzZlNDA2In0=";
const VECTOR_PER_SITE =
  "Vk9MQUNUAXsidmVyc2lvbiI6MiwibGljZW5zZV9rZXkiOiJWT0xUUlVTLVZFQ1RPUiIsImZpbmdlcnByaW50X2hhc2giOiJzaGEyNTY6dmVjdG9yIiwiaXNzdWVkX2F0IjoiMjAyNi0xMC0yOFQwMDowMDowMC4wMDBaIiwiZXhwaXJlc19hdCI6bnVsbCwidGllciI6ImludGVncmF0b3IiLCJkZXZpY2VfbGltaXQiOm51bGwsInNpdGVfaWQiOiIwYjBlOGE1Yy0xZjNhLTRmNmUtOWQwYS01ZDFkOGY0YzdhMTEiLCJzaXRlX25hbWUiOiJQbGFudCBcIjFcIiDDqSIsInN1cHBvcnRfdW50aWwiOjE3OTg3NjE2MDAsInNpZ25hdHVyZSI6IjExZmQ2MDA3ZGExYzk3ZDQ0YWVmZjRlZjcxNDAwN2I5ZjdmZDI3ZmUyMzQzYmMwNTMzNzk1YTM3NGM3ZTFjMDg4OGM3ZTAxMjA2YTZlNGIzZjhlZmJkMTJmZGUxMGQ5OWU4NzBlNjNhMWQyMDViNDUxYjk5YTZmZTBmNDY5MTA4In0=";
