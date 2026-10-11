import type { Env } from "./env";

export interface EmailMessage {
  subject: string;
  html: string;
  text: string;
}

const FROM_EMAIL = "noreply@mail.voltrus.id";
const FROM_NAME = "Voltrus";
const REPLY_TO = "support@voltrus.id";
function unsubBase(env: Env): string {
  return `${(env.EMAIL_PUBLIC_BASE ?? "https://voltrus-license-activate.dzulfikar-at.workers.dev").replace(/\/+$/, "")}/email/unsubscribe`;
}
async function unsubToken(email: string, secret: string): Promise<string> {
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(`voltrus-email-unsub:${secret}:${email.toLowerCase()}`)
  );
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}
export async function verifyUnsubToken(email: string, token: string, secret: string): Promise<boolean> {
  const expected = await unsubToken(email, secret);
  return expected === token;
}
const CROSS_SELL: Record<string, { name: string; url: string; why: string }> = {
  "modbus-poll": { name: "Serial Terminal", url: "https://voltrus.id/serial-terminal/", why: "RS485 adapters, hex views, and the same Modbus RTU wire you are already polling" },
  "serial-terminal": { name: "Modbus Poll", url: "https://voltrus.id/modbus-poll/", why: "poll registers and coils over the serial links you are debugging" },
  "bacnet-explorer": { name: "Modbus Poll", url: "https://voltrus.id/modbus-poll/", why: "most BACnet sites also speak Modbus on the same controller" },
  "enip-explorer": { name: "DNP3 Explorer", url: "https://voltrus.id/dnp3-explorer/", why: "utilities running Allen-Bradley usually run DNP3 on the SCADA side" },
  "dnp3-explorer": { name: "EtherNet/IP Explorer", url: "https://voltrus.id/enip-explorer/", why: "the same substations usually have Allen-Bradley gear on the LAN" },
  "opcua-explorer": { name: "S7 Explorer", url: "https://voltrus.id/s7-explorer/", why: "Siemens shops expose both OPC-UA and S7 \u2014 cover both without TIA Portal" },
  "s7-explorer": { name: "OPC-UA Explorer", url: "https://voltrus.id/opcua-explorer/", why: "modern S7-1500s expose OPC-UA servers you can browse natively" },
  "mqtt-explorer": { name: "Modbus Poll", url: "https://voltrus.id/modbus-poll/", why: "poll the devices feeding your MQTT broker, then publish the values" },
  "mqtt-client": { name: "Modbus Poll", url: "https://voltrus.id/modbus-poll/", why: "poll the devices feeding your MQTT broker, then publish the values" },
  "nats-explorer": { name: "MQTT Explorer", url: "https://voltrus.id/mqtt-explorer/", why: "the edge devices upstream of NATS usually speak MQTT first" },
  "ble-inspector": { name: "Serial Terminal", url: "https://voltrus.id/serial-terminal/", why: "wired and wireless debugging in the same toolbox" },
  "pastehere": { name: "SQLTUI", url: "https://voltrus.id/sqltui/", why: "a terminal SQLite browser from the same team" },
  "postgrestui": { name: "SQLTUI", url: "https://voltrus.id/sqltui/", why: "same terminal workflow for SQLite" },
  "sqltui": { name: "PostgresTUI", url: "https://voltrus.id/postgrestui/", why: "same terminal workflow for Postgres" },
  "mssqltui": { name: "SQLTUI", url: "https://voltrus.id/sqltui/", why: "same terminal workflow for SQLite" },
  "voltrus-plc": { name: "Voltrus SCADA", url: "https://voltrus.id/voltrus-scada/", why: "watch what your PLC does, continuously, with alarms and history" },
  "voltrus-scada": { name: "Voltrus PLC", url: "https://voltrus.id/voltrus-plc/", why: "program the logic AND monitor it \u2014 native Mac toolchain end to end" },
  "voltrus-hmi": { name: "Voltrus SCADA", url: "https://voltrus.id/voltrus-scada/", why: "dashboards plus alarms, history, and drivers in one binary" },
  "voltrus-gateway": { name: "Voltrus SCADA", url: "https://voltrus.id/voltrus-scada/", why: "the gateway feeds SCADA \u2014 monitor everything it proxies" }
};
const BUNDLE_URL = "https://voltrus.id/mactools/";
function layout(title: string, bodyHtml: string, token: string, base: string): { html: string } {
  const unsubUrl = `${base}?token=${token}`;
  const html = `<!DOCTYPE html><html><body style="margin:0;padding:0;background:#0e0f11;font-family:-apple-system,Segoe UI,Roboto,Helvetica,Arial,sans-serif;color:#e8e6e3;">
  <table role="presentation" width="100%" cellpadding="0" cellspacing="0"><tr><td align="center" style="padding:32px 16px;">
    <table role="presentation" width="560" cellpadding="0" cellspacing="0" style="max-width:560px;background:#16181b;border:1px solid #2a2d33;border-radius:12px;">
      <tr><td style="padding:28px 32px;border-bottom:1px solid #2a2d33;">
        <span style="font-family:ui-monospace,Menlo,monospace;font-size:13px;letter-spacing:0.12em;color:#6ea8fe;">VOLTRUS</span>
      </td></tr>
      <tr><td style="padding:28px 32px;">
        <h1 style="margin:0 0 16px;font-size:20px;line-height:1.35;color:#f2f0ed;">${title}</h1>
        ${bodyHtml}
      </td></tr>
      <tr><td style="padding:18px 32px;border-top:1px solid #2a2d33;font-size:12px;color:#8a8d94;">
        Voltrus \u2014 native industrial automation tools \xB7 <a href="${unsubUrl}" style="color:#8a8d94;">unsubscribe</a>
      </td></tr>
    </table>
  </td></tr></table></body></html>`;
  return { html };
}
async function render(env: Env, email: string, title: string, bodyHtml: string, textBody: string): Promise<{ html: string; text: string }> {
  const token = await unsubToken(email, env.POLAR_WEBHOOK_SECRET);
  const base = unsubBase(env);
  const { html } = layout(title, bodyHtml, token, base);
  const unsubUrl = `${base}?token=${token}`;
  return { html, text: `${textBody}

--
Voltrus \u2014 native industrial automation tools
Unsubscribe: ${unsubUrl}` };
}
export async function sendEmail(env: Env, to: string, subject: string, html: string, text: string): Promise<void> {
  await env.EMAIL.send({ to, from: { email: FROM_EMAIL, name: FROM_NAME }, replyTo: REPLY_TO, subject, html, text });
}
export async function renderDay0(env: Env, email: string, productName: string): Promise<EmailMessage> {
  const bodyHtml = `<p style="margin:0 0 12px;font-size:15px;line-height:1.6;color:#c9c6c2;">Your ${productName} license is confirmed. Your license key is in the Polar receipt email \u2014 paste it on first launch and you're running.</p>
<p style="margin:0 0 20px;font-size:15px;line-height:1.6;color:#c9c6c2;">Everything you need \u2014 downloads, activation steps, and docs \u2014 is on your thank-you page.</p>
<p style="margin:0;"><a href="https://voltrus.id/" style="display:inline-block;padding:11px 22px;background:#6ea8fe;color:#101216;border-radius:8px;font-size:14px;font-weight:600;text-decoration:none;">Open Voltrus</a></p>`;
  const { html, text } = await render(
    env,
    email,
    `Your ${productName} license`,
    bodyHtml,
    `Your ${productName} license is confirmed. Your license key is in the Polar receipt email \u2014 paste it on first launch and you're running.`
  );
  return { subject: `Your ${productName} license is ready`, html, text };
}
export async function renderDay3(env: Env, email: string, productName: string): Promise<EmailMessage> {
  const cross = CROSS_SELL[productName] ?? { name: "the MacTools suite", url: "https://voltrus.id/mactools/", why: "one toolbox for every protocol on your site" };
  const bodyHtml = `<p style="margin:0 0 12px;font-size:15px;line-height:1.6;color:#c9c6c2;">Hope ${productName} is earning its keep. A heads-up from the same toolbox:</p>
<p style="margin:0 0 12px;font-size:16px;line-height:1.5;color:#f2f0ed;"><strong>${cross.name}</strong> \u2014 ${cross.why}.</p>
<p style="margin:0 0 20px;font-size:15px;line-height:1.6;color:#c9c6c2;">Same native Mac build, same one-time pricing.</p>
<p style="margin:0;"><a href="${cross.url}" style="display:inline-block;padding:11px 22px;background:#6ea8fe;color:#101216;border-radius:8px;font-size:14px;font-weight:600;text-decoration:none;">See ${cross.name}</a></p>`;
  const { html, text } = await render(
    env,
    email,
    `Pairs well with ${productName}`,
    bodyHtml,
    `Hope ${productName} is earning its keep. ${cross.name} pairs well \u2014 ${cross.why}. Same native Mac build, same one-time pricing: ${cross.url}`
  );
  return { subject: `Pairs well with your ${productName}`, html, text };
}
export async function renderDay10(env: Env, email: string, productName: string): Promise<EmailMessage> {
  const bodyHtml = `<p style="margin:0 0 12px;font-size:15px;line-height:1.6;color:#c9c6c2;">If ${productName} is now part of your daily kit, the rest of the MacTools suite likely is too \u2014 Modbus, BACnet, EtherNet/IP, DNP3, OPC-UA, S7, MQTT, serial, in one native toolbox.</p>
<p style="margin:0 0 20px;font-size:15px;line-height:1.6;color:#c9c6c2;">Grab the tools you're missing on the suite page \u2014 bundles are on the way for multi-tool buyers, and existing customers hear first.</p>
<p style="margin:0;"><a href="${BUNDLE_URL}" style="display:inline-block;padding:11px 22px;background:#6ea8fe;color:#101216;border-radius:8px;font-size:14px;font-weight:600;text-decoration:none;">Browse the MacTools suite</a></p>`;
  const { html, text } = await render(
    env,
    email,
    "The rest of the toolbox",
    bodyHtml,
    `If ${productName} is part of your daily kit, see the rest of the MacTools suite \u2014 every industrial protocol, one native toolbox: ${BUNDLE_URL}`
  );
  return { subject: "The rest of the Voltrus toolbox", html, text };
}
