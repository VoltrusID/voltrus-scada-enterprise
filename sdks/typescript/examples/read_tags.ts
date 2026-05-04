import { VoltrusClient } from "voltrus";

const host = process.argv[2] || "http://localhost:8080";
const apiKey = process.argv[3] || "";

const client = new VoltrusClient({ baseURL: host, apiKey });

async function main() {
  // List screens
  const screens = await client.listScreens();
  console.log(`Found ${screens.length} screens`);

  if (screens.length === 0) {
    console.log("No screens found. Create one in the Voltrus UI first.");
    return;
  }

  // Read live data from first screen
  const data = await client.getLiveData(screens[0].id);

  for (const tag of data) {
    const icon = tag.quality === "good" ? "OK" : tag.quality === "bad" ? "BAD" : "???";
    console.log(`[${icon}] ${tag.name} = ${tag.value}`);
  }
}

main().catch(console.error);
