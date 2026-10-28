// Activation and renewal against a fake D1 / Polar / console: the signed `tier` and `expires_at`, the
// renewal path (same key + same fingerprint, never binds, never spends a Polar usage) and refusals.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  DAY,
  FP,
  KEY,
  openActivation,
  post,
  reply,
  setup,
  volreq,
  type Binding,
  type ConsoleLicense,
} from "./helpers";

beforeEach(() => {
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(console, "error").mockImplementation(() => {});
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("activate: signed tier and expiry", () => {
  it.each([
    ["integrator", "integrator"],
    ["enterprise", "enterprise"],
    ["edge", "edge"],
    ["starter", "starter"],
    ["professional", "professional"],
  ])("a lifetime key bought as %s is signed as %s with no expiry", async (bought, signed) => {
    const t = setup({ info: { tier: bought, expires_at: null, recurring: false } });
    const out = await reply(await post(t.env, "/activate", volreq()));
    expect(out).toMatchObject({ success: true, tier: signed, expires_at: null });
    const payload = await openActivation(out.activation_file!);
    expect(payload).toMatchObject({ tier: signed, expires_at: null, license_key: KEY, fingerprint_hash: FP });
    expect(t.bindings.calls.some((c) => c.sql.startsWith("INSERT INTO license_bindings"))).toBe(true);
    // The first activation spends the Polar usage.
    expect(t.polarBodies[0]).toMatchObject({ increment_usage: 1 });
  });

  it("records which signing key issued the activation", async () => {
    const t = setup({ info: { tier: "integrator", expires_at: null, recurring: false } });
    await post(t.env, "/activate", volreq());
    const recorded = t.bindings.calls.find((c) => c.sql.startsWith("INSERT INTO activation_signings"));
    expect(recorded?.args).toEqual([KEY, "2026-10"]);
  });

  it("a failed signing record does not block the activation", async () => {
    const t = setup({ info: { tier: "integrator", expires_at: null, recurring: false } });
    const prepare = t.bindings.prepare.bind(t.bindings);
    t.bindings.prepare = (sql: string) => {
      if (sql.startsWith("INSERT INTO activation_signings")) throw new Error("no such table");
      return prepare(sql);
    };
    const out = await reply(await post(t.env, "/activate", volreq()));
    expect(out.success).toBe(true);
  });

  it("a monthly key expires at the subscription period end", async () => {
    const end = new Date(Date.now() + 20 * DAY).toISOString();
    const t = setup({ info: { tier: "enterprise", expires_at: end, recurring: true } });
    const out = await reply(await post(t.env, "/activate", volreq()));
    expect(out).toMatchObject({ success: true, tier: "enterprise", expires_at: end });
    expect(await openActivation(out.activation_file!)).toMatchObject({ tier: "enterprise", expires_at: end });
  });

  it("a monthly key whose period can't be read is refused as transient, not signed as lifetime", async () => {
    const t = setup({ info: { tier: "integrator", expires_at: null, recurring: true } });
    const res = await post(t.env, "/activate", volreq());
    expect(res.status).toBe(503);
    expect(await reply(res)).toMatchObject({ success: false, code: "unavailable" });
    expect(t.bindings.calls.some((c) => c.sql.startsWith("INSERT INTO license_bindings"))).toBe(false);
  });

  it("an unresolved purchase keeps the bound tier", async () => {
    const t = setup({
      binding: { fingerprint_hash: FP, revoked: 0, tier: "enterprise", device_limit: null },
      info: null,
    });
    const out = await reply(await post(t.env, "/activate", volreq()));
    expect(out).toMatchObject({ success: true, tier: "enterprise" });
  });
});

describe("renew", () => {
  const bound: Binding = { fingerprint_hash: FP, revoked: 0, tier: "integrator", device_limit: null };

  it("signs a fresh activation with the new period end for the bound device, without spending a Polar usage", async () => {
    const end = new Date(Date.now() + 31 * DAY).toISOString();
    const t = setup({ binding: bound, info: { tier: "integrator", expires_at: end, recurring: true } });
    const out = await reply(await post(t.env, "/renew", volreq()));
    expect(out).toMatchObject({ success: true, tier: "integrator", expires_at: end });
    expect(await openActivation(out.activation_file!)).toMatchObject({ expires_at: end, fingerprint_hash: FP });
    expect(t.polarBodies[0]).not.toHaveProperty("increment_usage");
    expect(t.bindings.calls.some((c) => c.sql.startsWith("INSERT INTO license_bindings"))).toBe(false);
  });

  it("takes the same request as JSON", async () => {
    const end = new Date(Date.now() + 31 * DAY).toISOString();
    const t = setup({ binding: bound, info: { tier: "integrator", expires_at: end, recurring: true } });
    const out = await reply(await post(t.env, "/renew", { license_key: KEY, fingerprint: { hash: FP } }));
    expect(out).toMatchObject({ success: true, expires_at: end });
  });

  it("picks up an upgrade to a higher tier", async () => {
    const t = setup({ binding: bound, info: { tier: "enterprise", expires_at: null, recurring: false } });
    expect(await reply(await post(t.env, "/renew", volreq()))).toMatchObject({ success: true, tier: "enterprise" });
    expect(t.bindings.calls.find((c) => c.sql.startsWith("UPDATE license_bindings"))!.args[0]).toBe("enterprise");
  });

  it("never binds: an unbound key answers 404 not_activated", async () => {
    const t = setup({ binding: null, info: { tier: "integrator", expires_at: null, recurring: false } });
    const res = await post(t.env, "/renew", volreq());
    expect(res.status).toBe(404);
    expect(await reply(res)).toMatchObject({ success: false, code: "not_activated" });
    expect(t.polarBodies).toHaveLength(0);
    expect(t.bindings.calls.some((c) => c.sql.startsWith("INSERT"))).toBe(false);
  });

  it("refuses another device", async () => {
    const t = setup({ binding: bound, info: { tier: "integrator", expires_at: null, recurring: false } });
    const res = await post(t.env, "/renew", volreq(KEY, "other-device"));
    expect(res.status).toBe(403);
    expect(await reply(res)).toMatchObject({ code: "fingerprint_mismatch" });
  });

  it("refuses a revoked binding and a key Polar revoked (cancel or refund)", async () => {
    const revokedBinding = setup({ binding: { ...bound, revoked: 1 } });
    const a = await post(revokedBinding.env, "/renew", volreq());
    expect(a.status).toBe(403);
    expect(await reply(a)).toMatchObject({ code: "revoked" });

    const revokedByPolar = setup({ binding: bound, polarStatus: "revoked" });
    const b = await post(revokedByPolar.env, "/renew", volreq());
    expect(b.status).toBe(403);
    expect(await reply(b)).toMatchObject({ code: "revoked", error: "License key is revoked" });
  });

  it("answers 503 unavailable when Polar is down, so the install retries later", async () => {
    const t = setup({ binding: bound, polarHttp: 502 });
    const res = await post(t.env, "/renew", volreq());
    expect(res.status).toBe(503);
    expect(await reply(res)).toMatchObject({ code: "unavailable" });
  });

  it("rejects malformed requests", async () => {
    const t = setup({});
    const res = await post(t.env, "/renew", { license_key: "", fingerprint: {} });
    expect(res.status).toBe(400);
    expect(await reply(res)).toMatchObject({ code: "invalid_request" });
    const junk = await post(t.env, "/renew", new Blob(["not a request"]));
    expect(junk.status).toBe(400);
  });
});

describe("console VLT licenses", () => {
  const code = "VLT-AAAAA-BBBBB-CCCCC";
  const lic = (over: Partial<ConsoleLicense> = {}): ConsoleLicense => ({
    id: "lic-1",
    status: "active",
    expires_at: null,
    seat_limit: 1,
    tier: "enterprise",
    ...over,
  });

  it("signs the license tier and, for a monthly license, its period end", async () => {
    const end = Date.now() + 25 * DAY;
    const t = setup({ consoleLicense: lic({ expires_at: end }), seated: true });
    const out = await reply(await post(t.env, "/renew", volreq(code)));
    expect(out).toMatchObject({ success: true, tier: "enterprise", expires_at: new Date(end).toISOString() });
    expect(await openActivation(out.activation_file!)).toMatchObject({ tier: "enterprise", expires_at: new Date(end).toISOString() });
  });

  it("a lifetime license has no expiry and a tierless one stays starter", async () => {
    const t = setup({ consoleLicense: lic({ tier: null }), seated: true });
    expect(await reply(await post(t.env, "/activate", volreq(code)))).toMatchObject({ tier: "starter", expires_at: null });
  });

  it("renew never seats a new device", async () => {
    const t = setup({ consoleLicense: lic(), seated: false });
    const res = await post(t.env, "/renew", volreq(code));
    expect(res.status).toBe(404);
    expect(await reply(res)).toMatchObject({ code: "not_activated" });
    expect(t.consoleDb.calls.some((c) => c.sql.startsWith("INSERT"))).toBe(false);
  });

  it("refuses revoked, expired and unknown licenses", async () => {
    const revoked = await post(setup({ consoleLicense: lic({ status: "revoked" }), seated: true }).env, "/renew", volreq(code));
    expect(revoked.status).toBe(403);
    expect(await reply(revoked)).toMatchObject({ code: "revoked" });

    const expired = await post(setup({ consoleLicense: lic({ expires_at: Date.now() - DAY }), seated: true }).env, "/renew", volreq(code));
    expect(await reply(expired)).toMatchObject({ code: "expired" });

    const unknown = await post(setup({ consoleLicense: null }).env, "/renew", volreq(code));
    expect(await reply(unknown)).toMatchObject({ code: "invalid_key" });
  });
});
