import { Configuration, CommandsApi } from "voltrus";

const host = process.argv[2] || "http://localhost:3000";
const apiKey = process.argv[3] || "";
const tagId = parseInt(process.argv[4] || "1");
const value = process.argv[5] || "1";

const config = new Configuration({
  basePath: host,
  apiKey,
});

async function main() {
  const api = new CommandsApi(config);
  const result = await api.writeTag(tagId, { value });
  const commandId = (result.data as any).command_id;
  console.log(`Command submitted: ${commandId}`);

  for (let i = 0; i < 30; i++) {
    const status = await api.getCommandStatus(commandId);
    const state = (status.data as any).status;
    console.log(`  Status: ${state}`);

    if (state === "acked" || state === "resolved") {
      console.log("Command acknowledged!");
      return;
    }
    if (state === "failed") {
      console.log(`Command failed: ${(status.data as any).error_message}`);
      return;
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  console.log("Timeout waiting for acknowledgment");
}

main().catch(console.error);
