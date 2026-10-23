// Activation and renewal against a fake D1 / Polar / console: the signed `tier` and `expires_at`, the
// renewal path (same key + same fingerprint, never binds, never spends a Polar usage) and refusals.
import { getPublicKey, verify } from "@noble/ed25519";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import worker from "../src/index";
import type { Env, PolarKeyInfo } from "../src/env";

const PRIVATE_KEY = "11".repeat(32);
const FP = "fp-device-1";
const KEY = "VOLTRUS-AAAA-BBBB";
const DAY = 86_400_000;

type Binding = { fingerprint_hash: string; revoked: number; tier: string | null; device_limit: number | null };
type ConsoleLicense = { id: string; status: string; expires_at: number | null; seat_limit: number | null; tier: string | null };

// Just enough D1: statements are routed by their leading SQL.
function fakeDb(routes: Record<string, (args: unknown[]) => unknown>) {
  const calls: { sql: string; args: unknown[] }[] = [];
  const db = {
    calls,
    prepare(sql: string) {
      return {
        bind(...args: unknown[]) {
          const run = () => {
            calls.push({ sql, args });
            const route = Object.entries(routes).find(([prefix]) => sql.startsWith(prefix));
            if (!route) throw new Error(`unrouted SQL: ${sql}`);
            return route[1](args);
          };
          return {
            first: async () => run() ?? null,
            run: async () => ({ meta: { changes: 1, last_row_id: 1 }, result: run() }),
            all: async () => ({ results: [run()] }),
          };
        },
      };
    },
  };
  return db;
}

function setup(opts: {
  binding?: Binding | null;
  info?: PolarKeyInfo | null;
  polarStatus?: string;
  polarHttp?: number;
  consoleLicense?: ConsoleLicense | null;
  seated?: boolean;
}) {
  const bindings = fakeDb({
    "SELECT fingerprint_hash": () => opts.binding ?? null,
    "UPDATE license_bindings": () => null,
    "INSERT INTO license_bindings": () => null,
    "INSERT INTO activation_signings": () => null,
  });
  const consoleDb = fakeDb({
    "SELECT id, status, expires_at": () => opts.consoleLicense ?? null,
    "SELECT id FROM license_activations": () => (opts.seated ? { id: "seat-1" } : null),
    "SELECT COUNT(*)": () => ({ n: opts.seated ? 1 : 0 }),
    "INSERT OR IGNORE INTO license_activations": () => null,
  });
  const polarBodies: Record<string, unknown>[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (_url: string, init: RequestInit) => {
      polarBodies.push(JSON.parse(String(init.body)));
      if (opts.polarHttp && opts.polarHttp !== 200) return new Response(JSON.stringify({ detail: "polar says no" }), { status: opts.polarHttp });
      return new Response(JSON.stringify({ id: "polar-key-1", status: opts.polarStatus ?? "granted", expires_at: null }));
    })
  );
  const env = {
    DB: bindings,
    CONSOLE_DB: consoleDb,
    CONSOLE: { polarKeyInfo: async () => opts.info ?? null },
    ED25519_PRIVATE_KEY: PRIVATE_KEY,
    POLAR_API_KEY: "polar-key",
    POLAR_ORG_ID: "org-1",
  } as unknown as Env;
  return { env, bindings, consoleDb, polarBodies };
}

// The VOLREQ file the app uploads.
function volreq(key = KEY, fingerprint = FP): Blob {
  return new Blob(["VOLREQ\x01" + JSON.stringify({ license_key: key, fingerprint: { hash: fingerprint, components: { hostname: "plant-1" } } })]);
}
function post(env: Env, path: "/activate" | "/renew", body: Blob | Record<string, unknown>): Promise<Response> {
  if (body instanceof Blob) {
    const form = new FormData();
    form.set("file", body);
    return worker.fetch(new Request(`https://activate.test${path}`, { method: "POST", body: form }), env);
  }
  return worker.fetch(
    new Request(`https://activate.test${path}`, { method: "POST", body: JSON.stringify(body), headers: { "Content-Type": "application/json" } }),
    env
  );
}

type Reply = { success: boolean; error?: string; code?: string; activation_file?: string; tier?: string; expires_at?: string | null };
async function reply(res: Response): Promise<Reply> {
  return res.json();
}

// Decode a VOLACT file and check its Ed25519 signature the way the SCADA server does.
async function openActivation(file: string) {
  const bytes = Uint8Array.from(atob(file), (c) => c.charCodeAt(0));
  expect(new TextDecoder().decode(bytes.slice(0, 7))).toBe("VOLACT\x01");
  const { signature, ...payload } = JSON.parse(new TextDecoder().decode(bytes.slice(7))) as Record<string, unknown> & { signature: string };
  const sig = Uint8Array.from(signature.match(/../g)!.map((h) => parseInt(h, 16)));
  const pub = getPublicKey(Uint8Array.from("11".repeat(32).match(/../g)!.map((h) => parseInt(h, 16))));
  expect(verify(sig, new TextEncoder().encode(JSON.stringify(payload)), pub)).toBe(true);
  return payload as { tier: string; expires_at: string | null; license_key: string; fingerprint_hash: string; device_limit: number | null };
}

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
