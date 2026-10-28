// Shared by the activation tests: a fake D1 routed by leading SQL, a fake Polar / console, and the
// VOLREQ / VOLACT file helpers.
import { getPublicKey, verify } from "@noble/ed25519";
import { expect, vi } from "vitest";
import worker from "../src/index";
import type { Env, PolarKeyInfo } from "../src/env";

export const PRIVATE_KEY = "11".repeat(32);
export const FP = "fp-device-1";
export const KEY = "VOLTRUS-AAAA-BBBB";
export const DAY = 86_400_000;

export type Binding = {
  fingerprint_hash: string;
  revoked: number;
  tier: string | null;
  device_limit: number | null;
  site_id?: string | null;
  site_name?: string | null;
  support_until?: number | null;
};
export type ConsoleLicense = { id: string; status: string; expires_at: number | null; seat_limit: number | null; tier: string | null };
/** What the console D1 holds for a key: scada_sites row and scada_support row (support_until in ms). */
export type ConsoleSite = { site?: { id: string; name: string } | null; support_until_ms?: number | null };

export interface DbCall {
  sql: string;
  args: unknown[];
}
export interface FakeDb {
  calls: DbCall[];
  prepare(sql: string): {
    bind(...args: unknown[]): { first(): Promise<unknown>; run(): Promise<unknown>; all(): Promise<unknown> };
  };
}
/** What `setup` returns: the env to hand the worker and the fakes behind it. */
export interface Harness {
  env: Env;
  bindings: FakeDb;
  consoleDb: FakeDb;
  polarBodies: Record<string, unknown>[];
}

// Just enough D1: statements are routed by their leading SQL.
export function fakeDb(routes: Record<string, (args: unknown[]) => unknown>): FakeDb {
  const calls: DbCall[] = [];
  return {
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
}

export function setup(opts: {
  binding?: Binding | null;
  info?: PolarKeyInfo | null;
  polarStatus?: string;
  polarHttp?: number;
  consoleLicense?: ConsoleLicense | null;
  seated?: boolean;
  /** Console site / support rows; leave undefined for a console that has not been migrated (the tables do not exist). */
  consoleSite?: ConsoleSite;
}): Harness {
  const bindings = fakeDb({
    "SELECT fingerprint_hash": () => opts.binding ?? null,
    "UPDATE license_bindings": () => null,
    "INSERT INTO license_bindings": () => null,
    "INSERT INTO activation_signings": () => null,
  });
  const cs = opts.consoleSite;
  // The site row the console would hold after an INSERT OR IGNORE of this worker.
  let minted: { id: string; name: string } | null = null;
  const siteRoutes: Record<string, (args: unknown[]) => unknown> = cs
    ? {
        "SELECT (SELECT id FROM scada_sites": () => ({
          site_id: cs.site?.id ?? null,
          site_name: cs.site?.name ?? null,
          support_until: cs.support_until_ms ?? null,
        }),
        "INSERT OR IGNORE INTO scada_sites": (args) => {
          if (!cs.site && !minted) minted = { id: String(args[0]), name: String(args[2]) };
          return null;
        },
        "SELECT id, name FROM scada_sites": () => cs.site ?? minted,
      }
    : {};
  const consoleDb = fakeDb({
    "SELECT id, status, expires_at": () => opts.consoleLicense ?? null,
    "SELECT id FROM license_activations": () => (opts.seated ? { id: "seat-1" } : null),
    "SELECT COUNT(*)": () => ({ n: opts.seated ? 1 : 0 }),
    "INSERT OR IGNORE INTO license_activations": () => null,
    ...siteRoutes,
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

// The VOLREQ file the app uploads. `extra` adds request fields (site_name, features).
export function volreq(key = KEY, fingerprint = FP, extra: Record<string, unknown> = {}): Blob {
  return new Blob([
    "VOLREQ\x01" + JSON.stringify({ license_key: key, fingerprint: { hash: fingerprint, components: { hostname: "plant-1" } }, ...extra }),
  ]);
}
export function post(env: Env, path: "/activate" | "/renew", body: Blob | Record<string, unknown>): Promise<Response> {
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

export type Reply = {
  success: boolean;
  error?: string;
  code?: string;
  activation_file?: string;
  tier?: string;
  expires_at?: string | null;
  site?: { id: string; name: string } | null;
  support_until?: number | null;
};
export async function reply(res: Response): Promise<Reply> {
  return res.json();
}

export type Opened = {
  tier: string;
  expires_at: string | null;
  license_key: string;
  fingerprint_hash: string;
  device_limit: number | null;
  site_id?: string;
  site_name?: string;
  support_until?: number;
};

// Decode a VOLACT file and check its Ed25519 signature the way the SCADA server does: the payload is
// re-serialised with JSON.stringify in file order, which is what enterprise's ActivationPayload does.
export async function openActivation(file: string): Promise<Opened> {
  return (await openActivationRaw(file)).payload;
}

export async function openActivationRaw(file: string) {
  const bytes = Uint8Array.from(atob(file), (c) => c.charCodeAt(0));
  expect(new TextDecoder().decode(bytes.slice(0, 7))).toBe("VOLACT\x01");
  const { signature, ...payload } = JSON.parse(new TextDecoder().decode(bytes.slice(7))) as Record<string, unknown> & { signature: string };
  const sig = Uint8Array.from(signature.match(/../g)!.map((h) => parseInt(h, 16)));
  const pub = getPublicKey(Uint8Array.from("11".repeat(32).match(/../g)!.map((h) => parseInt(h, 16))));
  expect(verify(sig, new TextEncoder().encode(JSON.stringify(payload)), pub)).toBe(true);
  return { payload: payload as Opened, keys: Object.keys(payload), bytes };
}
