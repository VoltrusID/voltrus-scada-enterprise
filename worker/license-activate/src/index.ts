import { etc, sign } from "@noble/ed25519";
import { sha512 } from "@noble/hashes/sha2";
import { renderDay0, renderDay3, renderDay10, sendEmail, verifyUnsubToken } from "./email";
import type { Env } from "./env";

interface LicenseRequest {
  license_key: string;
  fingerprint: { hash: string; components?: Record<string, string> };
}

interface BindingRow {
  license_key: string;
  fingerprint_hash: string;
  activated_at: string;
  revoked: number;
  tier: string | null;
  device_limit: number | null;
}

interface EmailSequenceRow {
  id: number;
  email: string;
  product_name: string;
  age_days: number;
  sent_day0: number;
  sent_day3: number;
  sent_day10: number;
}

etc.sha512Sync = (...m: Uint8Array[]) => sha512(etc.concatBytes(...m));
const CORS_HEADERS_PUBLIC = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type"
};
const CORS_HEADERS_ADMIN = {
  "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization"
};
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      const url2 = new URL(request.url);
      const isAdmin = url2.pathname.startsWith("/admin");
      return new Response(null, { headers: isAdmin ? CORS_HEADERS_ADMIN : CORS_HEADERS_PUBLIC });
    }
    const url = new URL(request.url);
    const path = url.pathname;
    try {
      if (path === "/activate" && request.method === "POST") {
        return await handleActivate(request, env);
      }
      if (path === "/admin" && request.method === "GET") {
        return await handleAdminPanel(request, env);
      }
      if (path === "/admin/transfer" && request.method === "POST") {
        return await handleTransfer(request, env);
      }
      if (path.startsWith("/releases/") && request.method === "GET") {
        return await handleReleaseDownload(request, env, path);
      }
      if (path === "/latest" && request.method === "GET") {
        return await handleLatestRedirect(request, env);
      }
      if (path === "/downloads" && request.method === "GET") {
        return await handleDownloadsJson(request, env);
      }
      if (path === "/webhook/polar" && request.method === "POST") {
        return await handlePolarWebhook(request, env);
      }
      if (path === "/email/unsubscribe" && request.method === "GET") {
        return await handleUnsubscribe(request, env);
      }
      return jsonResponse({ error: "Not found" }, 404);
    } catch (_e) {
      console.error("Unhandled error:", _e);
      return jsonResponse({ error: "Internal server error" }, 500);
    }
  },
  // Daily cron: send due day-3 / day-10 sequence emails.
  async scheduled(_controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
    ctx.waitUntil(processDueEmails(env));
  }
} satisfies ExportedHandler<Env>;

async function verifyPolarSignature(rawBody: string, signatureHeader: string | null, secret: string): Promise<boolean> {
  if (!signatureHeader) return false;
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const mac = await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(rawBody));
  const hex = [...new Uint8Array(mac)].map((b) => b.toString(16).padStart(2, "0")).join("");
  return timingSafeEqual(hex, signatureHeader.trim());
}
async function handlePolarWebhook(request: Request, env: Env): Promise<Response> {
  const raw = await request.text();
  const ok = await verifyPolarSignature(raw, request.headers.get("webhook-signature"), env.POLAR_WEBHOOK_SECRET);
  if (!ok) return jsonResponse({ error: "bad signature" }, 401);
  let event: { type?: string; data?: { customer_email?: string; customer?: { email?: string }; product_name?: string; product?: { name?: string } } };
  try {
    event = JSON.parse(raw);
  } catch {
    return jsonResponse({ error: "bad json" }, 400);
  }
  if (event?.type !== "order.paid") return jsonResponse({ ok: true, ignored: event?.type });
  const d = event.data ?? {};
  const email = d.customer_email ?? d.customer?.email;
  const productName = d.product_name ?? d.product?.name;
  if (!email) return jsonResponse({ error: "no customer email in payload" }, 200);
  const existing = await env.DB.prepare(
    "SELECT id FROM email_sequence WHERE email = ?1 AND product_name = ?2 LIMIT 1"
  ).bind(email, productName ?? "unknown").first();
  if (existing) return jsonResponse({ ok: true, duplicate: true });
  const res = await env.DB.prepare(
    `INSERT INTO email_sequence (email, product_name, purchased_at) VALUES (?1, ?2, datetime('now'))`
  ).bind(email, productName ?? "unknown").run();
  const seqId = res.meta.last_row_id;
  try {
    const msg = await renderDay0(env, email, productName ?? "Voltrus tool");
    await sendEmail(env, email, msg.subject, msg.html, msg.text);
    await env.DB.prepare("UPDATE email_sequence SET sent_day0 = 1 WHERE id = ?1").bind(seqId).run();
  } catch (e) {
    console.error("day0 email failed:", e);
  }
  return jsonResponse({ ok: true });
}
async function processDueEmails(env: Env): Promise<void> {
  const due = await env.DB.prepare(
    `SELECT id, email, product_name,
            (julianday('now') - julianday(purchased_at)) AS age_days,
            sent_day0, sent_day3, sent_day10
     FROM email_sequence WHERE unsubscribed = 0 LIMIT 200`
  ).all<EmailSequenceRow>();
  if (!due.results) return;
  for (const row of due.results) {
    try {
      if (!row.sent_day0) {
        const m = await renderDay0(env, row.email, row.product_name);
        await sendEmail(env, row.email, m.subject, m.html, m.text);
        await env.DB.prepare("UPDATE email_sequence SET sent_day0 = 1 WHERE id = ?1").bind(row.id).run();
        continue;
      }
      if (!row.sent_day3 && row.age_days >= 3) {
        const m = await renderDay3(env, row.email, row.product_name);
        await sendEmail(env, row.email, m.subject, m.html, m.text);
        await env.DB.prepare("UPDATE email_sequence SET sent_day3 = 1 WHERE id = ?1").bind(row.id).run();
        continue;
      }
      if (!row.sent_day10 && row.age_days >= 10) {
        const m = await renderDay10(env, row.email, row.product_name);
        await sendEmail(env, row.email, m.subject, m.html, m.text);
        await env.DB.prepare("UPDATE email_sequence SET sent_day10 = 1 WHERE id = ?1").bind(row.id).run();
      }
    } catch (e) {
      console.error(`email sequence failed for ${row.id}:`, e);
    }
  }
}
async function handleUnsubscribe(request: Request, env: Env): Promise<Response> {
  const token = new URL(request.url).searchParams.get("token") ?? "";
  const rows = await env.DB.prepare(
    "SELECT id, email FROM email_sequence WHERE unsubscribed = 0 LIMIT 1000"
  ).all<{ id: number; email: string }>();
  for (const row of rows.results ?? []) {
    if (await verifyUnsubToken(row.email, token, env.POLAR_WEBHOOK_SECRET)) {
      await env.DB.prepare("UPDATE email_sequence SET unsubscribed = 1 WHERE id = ?1").bind(row.id).run();
      return new Response(
        `<!DOCTYPE html><html><head><meta charset="utf-8"><title>Unsubscribed \u2014 Voltrus</title></head>
         <body style="font-family:-apple-system,sans-serif;background:#0e0f11;color:#e8e6e3;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;">
         <div style="text-align:center;"><h1>You're unsubscribed.</h1>
         <p style="color:#8a8d94;">No more product emails from Voltrus. Transactional license receipts still arrive from Polar.</p></div></body></html>`,
        { headers: { "Content-Type": "text/html; charset=utf-8" } }
      );
    }
  }
  return new Response("Invalid or expired unsubscribe link.", { status: 404 });
}
async function handleActivate(request: Request, env: Env): Promise<Response> {
  const formData = await request.formData();
  const file: unknown = formData.get("file");
  if (!(file instanceof Blob)) {
    return jsonResponse({ success: false, error: "No file uploaded" }, 400);
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  const request_data = parseRequestFile(bytes);
  if (!request_data) {
    return jsonResponse({ success: false, error: "Invalid request file" }, 400);
  }
  if (/^\s*VLT-/i.test(request_data.license_key)) {
    return activateConsoleLicense(request_data, env);
  }
  const existing = await env.DB.prepare(
    "SELECT fingerprint_hash, revoked, tier, device_limit FROM license_bindings WHERE license_key = ?"
  ).bind(request_data.license_key).first<{ fingerprint_hash: string; revoked: number; tier: string | null; device_limit: number | null }>();
  if (existing?.revoked) {
    return jsonResponse({ success: false, error: "License has been revoked" }, 403);
  }
  if (existing && existing.fingerprint_hash !== request_data.fingerprint.hash) {
    return jsonResponse(
      { success: false, error: "License already bound to a different device. Contact support to transfer." },
      403
    );
  }
  // Re-activating the bound device (lost activation file, tier upgrade) must not spend a Polar usage.
  const polarValid = await validatePolarLicense(request_data.license_key, env, !existing);
  if (!polarValid.valid) {
    return jsonResponse({ success: false, error: polarValid.error }, 403);
  }
  // An unresolved purchase (Polar lookup failed) keeps the bound tier instead of resetting it.
  const resolved = polarValid.tier !== null;
  const tier = polarValid.tier ?? existing?.tier ?? DEFAULT_TIER;
  const device_limit = resolved ? polarValid.device_limit : existing ? existing.device_limit : null;
  const { expires_at } = polarValid;
  console.log(`Activation: license_key=${request_data.license_key}, rebind=${!!existing}, resolved=${resolved}, tier=${tier}, device_limit=${device_limit}`);
  if (existing) {
    await env.DB.prepare("UPDATE license_bindings SET tier = ?, device_limit = ? WHERE license_key = ?")
      .bind(tier, device_limit, request_data.license_key)
      .run();
  } else {
    await env.DB.prepare(
      "INSERT INTO license_bindings (license_key, fingerprint_hash, tier, device_limit) VALUES (?, ?, ?, ?)"
    ).bind(request_data.license_key, request_data.fingerprint.hash, tier, device_limit).run();
  }
  const activation = await generateActivation(request_data, env, expires_at, tier, device_limit);
  return jsonResponse({ success: true, activation_file: activation }, 200);
}

// Console-issued VLT-* licenses (console.voltrus.id card and saldo purchases) live in the
// voltrus-console D1, like the MacTools apps' VLT activations. Device seats are console
// license_activations rows, so "release" on the console Licenses page frees the host.
async function activateConsoleLicense(request_data: LicenseRequest, env: Env): Promise<Response> {
  const code = request_data.license_key.trim().toUpperCase();
  const lic = await env.CONSOLE_DB.prepare(
    "SELECT id, status, expires_at, seat_limit, tier FROM licenses WHERE code = ? AND app_slug = 'scada'"
  ).bind(code).first<{ id: string; status: string; expires_at: number | null; seat_limit: number | null; tier: string | null }>();
  if (!lic) {
    return jsonResponse({ success: false, error: "License key not found" }, 403);
  }
  if (lic.status !== "active") {
    return jsonResponse({ success: false, error: "License is not active" }, 403);
  }
  if (lic.expires_at && lic.expires_at < Date.now()) {
    return jsonResponse({ success: false, error: "License is expired" }, 403);
  }
  const device = request_data.fingerprint.hash;
  const seated = await env.CONSOLE_DB.prepare(
    "SELECT id FROM license_activations WHERE license_id = ? AND device_id = ?"
  ).bind(lic.id, device).first<{ id: string }>();
  if (!seated) {
    const seats = lic.seat_limit ?? 1;
    const used = await env.CONSOLE_DB.prepare("SELECT COUNT(*) AS n FROM license_activations WHERE license_id = ?")
      .bind(lic.id)
      .first<{ n: number }>();
    if ((used?.n ?? 0) >= seats) {
      return jsonResponse(
        { success: false, error: `All ${seats} seat(s) are in use. Release a device on console.voltrus.id/licenses first.` },
        403
      );
    }
    const host = request_data.fingerprint.components?.hostname;
    await env.CONSOLE_DB.prepare(
      "INSERT OR IGNORE INTO license_activations (id, license_id, device_id, label, created_at) VALUES (?, ?, ?, ?, ?)"
    ).bind(crypto.randomUUID(), lic.id, device, `SCADA ${host || device.slice(0, 8)}`, Date.now()).run();
  }
  // VLT SCADA licenses predating tiers came from the single $249 entry (Starter) product.
  const tier = lic.tier ?? "starter";
  console.log(`Activation (console): license=${code}, rebind=${!!seated}, tier=${tier}`);
  const expiresAt = lic.expires_at ? new Date(lic.expires_at).toISOString() : null;
  // Sign the key exactly as the host sent it: the binary matches it against its own config.
  const activation = await generateActivation(request_data, env, expiresAt, tier, null);
  return jsonResponse({ success: true, activation_file: activation }, 200);
}
async function handleAdminPanel(request: Request, env: Env): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (!auth || !timingSafeEqual(auth, `Bearer ${env.ADMIN_TOKEN}`)) {
    return new Response("Unauthorized", { status: 401 });
  }
  const bindings = await env.DB.prepare(
    "SELECT license_key, fingerprint_hash, activated_at, revoked, tier, device_limit FROM license_bindings ORDER BY activated_at DESC"
  ).all<BindingRow>();
  const rows = (bindings.results || []).map((b) => `
    <tr>
      <td>${escapeHtml(b.license_key)}</td>
      <td>${escapeHtml(b.fingerprint_hash)}</td>
      <td>${b.activated_at}</td>
      <td>${escapeHtml(b.tier || "professional")}</td>
      <td>${b.device_limit != null ? b.device_limit : "Unlimited"}</td>
      <td>${b.revoked ? "REVOKED" : "Active"}</td>
      <td>
        <form method="POST" action="/admin/transfer" style="display:inline">
          <input type="hidden" name="license_key" value="${escapeHtml(b.license_key)}" />
          <button type="submit">Transfer</button>
        </form>
      </td>
    </tr>
  `).join("");
  const html = `<!DOCTYPE html>
<html>
<head><title>Voltrus License Admin</title></head>
<body>
<h1>License Bindings</h1>
<table border="1">
<tr><th>License Key</th><th>Fingerprint</th><th>Activated</th><th>Tier</th><th>Device Limit</th><th>Status</th><th>Action</th></tr>
${rows}
</table>
</body>
</html>`;
  return new Response(html, { headers: { "Content-Type": "text/html" } });
}
async function handleTransfer(request: Request, env: Env): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (!auth || !timingSafeEqual(auth, `Bearer ${env.ADMIN_TOKEN}`)) {
    return new Response("Unauthorized", { status: 401 });
  }
  const formData = await request.formData();
  const license_key = formData.get("license_key");
  if (!license_key) {
    return jsonResponse({ error: "Missing license_key" }, 400);
  }
  await env.DB.prepare("UPDATE license_bindings SET revoked = 1 WHERE license_key = ?").bind(license_key).run();
  return jsonResponse({ success: true, message: "License transfer: previous binding revoked" }, 200);
}
function parseRequestFile(bytes: Uint8Array): LicenseRequest | null {
  const magic = new TextDecoder().decode(bytes.slice(0, 7));
  if (magic !== "VOLREQ") return null;
  try {
    const json = new TextDecoder().decode(bytes.slice(7));
    const data = JSON.parse(json);
    if (!data || typeof data.license_key !== "string" || !data.license_key) return null;
    if (!data.fingerprint || typeof data.fingerprint.hash !== "string" || !data.fingerprint.hash) return null;
    return data;
  } catch {
    return null;
  }
}
// tier null: the key is valid but its purchase couldn't be resolved.
type PolarValidation =
  | { valid: true; expires_at: string | null; tier: string | null; device_limit: number | null }
  | { valid: false; error: string };

interface PolarValidatedKey {
  id: string;
  benefit_id: string;
  customer_id: string;
  status: string;
  expires_at: string | null;
}

interface PolarProduct {
  id: string;
  name: string;
  metadata?: Record<string, string | number | boolean>;
}

interface PolarGrant {
  order_id: string | null;
  subscription_id: string | null;
  properties?: { license_key_id?: string };
}

// Keys whose product carries no tier keep the pre-tier behaviour.
const DEFAULT_TIER = "professional";

function polarFetch(env: Env, path: string, init: RequestInit = {}): Promise<Response> {
  return fetch(`https://api.polar.sh${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", Authorization: `Bearer ${env.POLAR_API_KEY}` },
  });
}

async function validatePolarLicense(key: string, env: Env, incrementUsage: boolean): Promise<PolarValidation> {
  if (!env.POLAR_ORG_ID) {
    return { valid: true, expires_at: null, tier: DEFAULT_TIER, device_limit: null };
  }
  try {
    const res = await polarFetch(env, "/v1/license-keys/validate", {
      method: "POST",
      body: JSON.stringify({
        key,
        organization_id: env.POLAR_ORG_ID,
        ...(incrementUsage ? { increment_usage: 1 } : {}),
      }),
    });
    if (res.status === 404) {
      return { valid: false, error: "License key not found in Polar.sh" };
    }
    if (!res.ok) {
      const err = await res.json<{ detail?: string }>().catch(() => ({ detail: undefined }));
      return { valid: false, error: err.detail || `Polar.sh error: ${res.status}` };
    }
    const data = await res.json<PolarValidatedKey>();
    if (data.status === "revoked" || data.status === "disabled") {
      return { valid: false, error: `License key is ${data.status}` };
    }
    const product = await resolveKeyProduct(env, data);
    if (!product) {
      console.log(`Polar validate: key=${key}, status=${data.status}, product=unresolved`);
      return { valid: true, expires_at: data.expires_at, tier: null, device_limit: null };
    }
    const metadata = product.metadata ?? {};
    const tier = typeof metadata.tier === "string" && metadata.tier ? metadata.tier : DEFAULT_TIER;
    const limit = metadata.device_limit != null ? parseInt(String(metadata.device_limit), 10) : NaN;
    const device_limit = Number.isFinite(limit) ? limit : null;
    console.log(`Polar validate: key=${key}, status=${data.status}, product=${product.id} (${product.name}), tier=${tier}`);
    return { valid: true, expires_at: data.expires_at, tier, device_limit };
  } catch (e) {
    return { valid: false, error: `Polar.sh validation failed: ${e instanceof Error ? e.message : String(e)}` };
  }
}

// Polar's validate response names the benefit, not the product, and one license-key benefit can be
// shared by several products (the SCADA tiers share one). The purchase decides the tier:
// key -> its benefit grant -> the order (or subscription) that granted it -> that product.
async function resolveKeyProduct(env: Env, key: PolarValidatedKey): Promise<PolarProduct | null> {
  const grantsRes = await polarFetch(
    env,
    `/v1/benefits/${key.benefit_id}/grants?customer_id=${key.customer_id}&is_granted=true&limit=100`
  );
  if (!grantsRes.ok) {
    console.error(`Polar grants lookup failed: ${grantsRes.status} ${await grantsRes.text()}`);
    return null;
  }
  const grants = await grantsRes.json<{ items: PolarGrant[] }>();
  const grant = grants.items.find((g) => g.properties?.license_key_id === key.id);
  const source = grant?.order_id
    ? `/v1/orders/${grant.order_id}`
    : grant?.subscription_id
      ? `/v1/subscriptions/${grant.subscription_id}`
      : null;
  if (!source) {
    console.error(`No order or subscription grants license key ${key.id}`);
    return null;
  }
  const purchaseRes = await polarFetch(env, source);
  if (!purchaseRes.ok) {
    console.error(`Polar purchase lookup failed: ${source} ${purchaseRes.status} ${await purchaseRes.text()}`);
    return null;
  }
  return (await purchaseRes.json<{ product: PolarProduct | null }>()).product;
}
async function generateActivation(
  request_data: LicenseRequest,
  env: Env,
  polarExpiresAt: string | null | undefined,
  tier: string,
  device_limit: number | null,
): Promise<string> {
  const expiresAt = polarExpiresAt ? new Date(polarExpiresAt).toISOString() : null;
  const hasTier = !!tier;
  const payload: Record<string, unknown> = {
    version: hasTier ? 2 : 1,
    license_key: request_data.license_key,
    fingerprint_hash: request_data.fingerprint.hash,
    issued_at: (new Date()).toISOString(),
    expires_at: expiresAt
  };
  if (hasTier) {
    payload.tier = tier;
    payload.device_limit = device_limit ?? null;
  }
  const payload_json = new TextEncoder().encode(JSON.stringify(payload));
  const private_key = hexToBytes(env.ED25519_PRIVATE_KEY);
  const signature = sign(payload_json, private_key);
  const activation = {
    ...payload,
    signature: bytesToHex(signature)
  };
  const json = JSON.stringify(activation);
  const magic = new TextEncoder().encode("VOLACT");
  const result = new Uint8Array(magic.length + json.length);
  result.set(magic);
  result.set(new TextEncoder().encode(json), magic.length);
  return btoa(String.fromCharCode(...result));
}
async function handleReleaseDownload(_request: Request, env: Env, path: string): Promise<Response> {
  const key = decodeURIComponent(path.replace("/releases/", ""));
  if (!env.RELEASES_BUCKET) {
    return jsonResponse({ error: "R2 not configured" }, 500);
  }
  const object = await env.RELEASES_BUCKET.get(key);
  if (!object) {
    return new Response(`Not found: ${key}`, { status: 404, headers: CORS_HEADERS_PUBLIC });
  }
  return new Response(object.body, {
    headers: {
      "Content-Type": object.httpMetadata?.contentType || "application/octet-stream",
      "Content-Disposition": `attachment; filename="${key.split("/").pop()}"`,
      "Cache-Control": "public, max-age=86400",
      ...CORS_HEADERS_PUBLIC
    }
  });
}
async function handleLatestRedirect(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const platform = url.searchParams.get("platform") || "macos-arm64";
  if (!env.RELEASES_DB) {
    return jsonResponse({ error: "Release DB not configured" }, 500);
  }
  try {
    const file = await env.RELEASES_DB.prepare(
      "SELECT r2_key FROM release_files WHERE version = (SELECT MAX(version) FROM release_files WHERE platform = ?) AND platform = ?"
    ).bind(platform, platform).first<{ r2_key: string }>();
    if (file?.r2_key) {
      const base = new URL(request.url).origin;
      return Response.redirect(`${base}/releases/${encodeURIComponent(file.r2_key)}`, 302);
    }
  } catch (e) {
    console.error("Latest redirect error:", e);
  }
  return new Response("No release found", { status: 404 });
}
async function handleDownloadsJson(request: Request, env: Env): Promise<Response> {
  if (!env.RELEASES_DB) {
    return jsonResponse({ error: "Release DB not configured" }, 500);
  }
  try {
    const versionRow = await env.RELEASES_DB.prepare(
      "SELECT MAX(version) as v FROM release_files"
    ).first<{ v: number | null }>();
    if (!versionRow?.v) {
      return jsonResponse({ error: "No releases" }, 404);
    }
    const { results } = await env.RELEASES_DB.prepare(
      "SELECT platform, filename, r2_key, size FROM release_files WHERE version = ?"
    ).bind(versionRow.v).all<{ platform: string; filename: string; r2_key: string; size: number }>();
    const base = new URL(request.url).origin;
    const files: Record<string, { url: string; filename: string; size: number }> = {};
    for (const f of results ?? []) {
      files[f.platform] = { url: `${base}/releases/${encodeURIComponent(f.r2_key)}`, filename: f.filename, size: f.size };
    }
    return Response.json(
      { version: versionRow.v, files },
      { headers: { ...CORS_HEADERS_PUBLIC, "Cache-Control": "public, max-age=300" } }
    );
  } catch (e) {
    console.error("Downloads error:", e);
    return jsonResponse({ error: "Error" }, 500);
  }
}
function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json", ...CORS_HEADERS_PUBLIC }
  });
}
function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}
function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
}
function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#039;");
}
function timingSafeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  const aBuf = new TextEncoder().encode(a);
  const bBuf = new TextEncoder().encode(b);
  let r = 0;
  for (let i = 0; i < aBuf.length; i++) r |= aBuf[i] ^ bBuf[i];
  return r === 0;
}
