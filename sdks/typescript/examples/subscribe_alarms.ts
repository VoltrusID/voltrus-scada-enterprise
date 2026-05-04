import { VoltrusClient } from "voltrus";

const host = process.argv[2] || "http://localhost:8080";
const apiKey = process.argv[3] || "";

const client = new VoltrusClient({ baseURL: host, apiKey });

async function main() {
  // Get alarm summary
  const summary = await client.getAlarmSummary();
  console.log("=== Alarm Summary ===");
  console.log(`  Active: ${summary.active} (Critical: ${summary.critical}, Warning: ${summary.warning}, Info: ${summary.info})`);
  console.log(`  Acknowledged: ${summary.acknowledged}`);
  console.log(`  Shelved: ${summary.shelved}`);

  // List active alarms
  const alarms = await client.listAlarms({ state: "active", limit: 10 });
  console.log("\n=== Recent Active Alarms ===");
  for (const alarm of alarms) {
    console.log(`  [${alarm.severity.toUpperCase()}] ${alarm.source_name}: ${alarm.message}`);
  }

  // Subscribe to real-time alarms via SSE
  console.log("\n=== Subscribing to real-time alarms (Ctrl+C to exit) ===");
  const es = client.subscribeAlarms((alarm) => {
    console.log(`⚡ [${alarm.severity.toUpperCase()}] ${alarm.source_name}: ${alarm.message}`);
  });

  process.on("SIGINT", () => {
    es.close();
    process.exit(0);
  });
}

main().catch(console.error);
