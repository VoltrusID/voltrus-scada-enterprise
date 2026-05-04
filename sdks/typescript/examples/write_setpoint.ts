import { VoltrusClient } from "voltrus";

const host = process.argv[2] || "http://localhost:8080";
const apiKey = process.argv[3] || "";

const client = new VoltrusClient({ baseURL: host, apiKey });

async function main() {
  const tagId = process.argv[4] || "sensor-001";
  const value = parseFloat(process.argv[5] || "42.5");

  // Write a setpoint
  const result = await client.writeTag(tagId, value);
  console.log(`Write accepted: command_id=${result.command_id}, status=${result.status}`);

  // Poll for acknowledgement
  for (let i = 0; i < 10; i++) {
    await new Promise((r) => setTimeout(r, 1000));
    const status = await client.getCommandStatus(result.command_id);
    console.log(`  [${i + 1}s] status=${status.status}`);
    if (status.status === "acked" || status.status === "failed") {
      break;
    }
  }
}

main().catch(console.error);
