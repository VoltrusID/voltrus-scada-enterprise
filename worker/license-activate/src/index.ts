import { sign } from "@noble/ed25519";

export interface Env {
  DB: D1Database;
  POLAR_API_KEY: string;
  ED25519_PRIVATE_KEY: string;
  ADMIN_TOKEN: string;
}

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization",
};

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      return new Response(null, { headers: CORS_HEADERS });
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
      return jsonResponse({ error: "Not found" }, 404);
    } catch (e: any) {
      return jsonResponse({ error: e.message || "Internal error" }, 500);
    }
  },
};

async function handleActivate(request: Request, env: Env): Promise<Response> {
  const formData = await request.formData();
  const file = formData.get("file") as File | null;
  if (!file) {
    return jsonResponse({ success: false, error: "No file uploaded" }, 400);
  }

  const bytes = new Uint8Array(await file.arrayBuffer());
  const request_data = parseRequestFile(bytes);
  if (!request_data) {
    return jsonResponse({ success: false, error: "Invalid request file" }, 400);
  }

  const polarValid = await validatePolarLicense(request_data.license_key, env.POLAR_API_KEY);
  if (!polarValid.valid) {
    return jsonResponse({ success: false, error: polarValid.error || "Invalid license key" }, 403);
  }

  const existing = await env.DB.prepare(
    "SELECT fingerprint_hash, revoked FROM license_bindings WHERE license_key = ?"
  ).bind(request_data.license_key).first<{ fingerprint_hash: string; revoked: number }>();

  if (existing) {
    if (existing.revoked) {
      return jsonResponse({ success: false, error: "License has been revoked" }, 403);
    }
    if (existing.fingerprint_hash === request_data.fingerprint.hash) {
      const activation = await generateActivation(request_data, env);
      return jsonResponse({ success: true, activation_file: activation }, 200);
    }
    return jsonResponse(
      { success: false, error: "License already bound to a different device. Contact support to transfer." },
      403
    );
  }

  await env.DB.prepare(
    "INSERT INTO license_bindings (license_key, fingerprint_hash) VALUES (?, ?)"
  ).bind(request_data.license_key, request_data.fingerprint.hash).run();

  const activation = await generateActivation(request_data, env);
  return jsonResponse({ success: true, activation_file: activation }, 200);
}

async function handleAdminPanel(request: Request, env: Env): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (!auth || auth !== `Bearer ${env.ADMIN_TOKEN}`) {
    return new Response("Unauthorized", { status: 401 });
  }

  const bindings = await env.DB.prepare(
    "SELECT license_key, fingerprint_hash, activated_at, revoked FROM license_bindings ORDER BY activated_at DESC"
  ).all<{ license_key: string; fingerprint_hash: string; activated_at: string; revoked: number }>();

  const rows = (bindings.results || [])
    .map((b) => `
    <tr>
      <td>${escapeHtml(b.license_key)}</td>
      <td>${escapeHtml(b.fingerprint_hash)}</td>
      <td>${b.activated_at}</td>
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
<tr><th>License Key</th><th>Fingerprint</th><th>Activated</th><th>Status</th><th>Action</th></tr>
${rows}
</table>
</body>
</html>`;

  return new Response(html, { headers: { "Content-Type": "text/html" } });
}

async function handleTransfer(request: Request, env: Env): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (!auth || auth !== `Bearer ${env.ADMIN_TOKEN}`) {
    return new Response("Unauthorized", { status: 401 });
  }

  const formData = await request.formData();
  const license_key = formData.get("license_key") as string;
  if (!license_key) {
    return jsonResponse({ error: "Missing license_key" }, 400);
  }

  await env.DB.prepare("DELETE FROM license_bindings WHERE license_key = ?")
    .bind(license_key).run();

  return jsonResponse({ success: true, message: "License transferred" }, 200);
}

function parseRequestFile(bytes: Uint8Array): any {
  const magic = new TextDecoder().decode(bytes.slice(0, 7));
  if (magic !== "VOLREQ\x01") return null;
  try {
    const json = new TextDecoder().decode(bytes.slice(7));
    return JSON.parse(json);
  } catch {
    return null;
  }
}

async function validatePolarLicense(key: string, apiKey: string): Promise<{ valid: boolean; error?: string }> {
  // TODO: Replace with actual Polar.sh API validation
  if (!key.startsWith("polar_")) {
    return { valid: false, error: "Invalid license key format" };
  }
  return { valid: true };
}

async function generateActivation(request_data: any, env: Env): Promise<string> {
  const payload = {
    version: 1,
    license_key: request_data.license_key,
    fingerprint_hash: request_data.fingerprint.hash,
    issued_at: new Date().toISOString(),
    expires_at: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString(),
  };

  const payload_json = new TextEncoder().encode(JSON.stringify(payload));
  const private_key = hexToBytes(env.ED25519_PRIVATE_KEY);
  const signature = await sign(payload_json, private_key);

  const activation = {
    ...payload,
    signature: bytesToHex(signature),
  };

  const json = JSON.stringify(activation);
  const magic = new TextEncoder().encode("VOLACT\x01");
  const result = new Uint8Array(magic.length + json.length);
  result.set(magic);
  result.set(new TextEncoder().encode(json), magic.length);
  return btoa(String.fromCharCode(...result));
}

function jsonResponse(data: any, status: number = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json", ...CORS_HEADERS },
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
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
