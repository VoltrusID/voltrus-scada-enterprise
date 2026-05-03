import { Configuration, DataApi } from "voltrus";

const host = process.argv[2] || "http://localhost:3000";
const apiKey = process.argv[3] || "";

const config = new Configuration({
  basePath: host,
  apiKey,
});

async function main() {
  const api = new DataApi(config);
  const data = await api.getData({ screen: "overview" });

  for (const ds of data) {
    const icon = { normal: "OK", warning: "WARN", critical: "CRIT", offline: "OFF" }[ds.status] || "?";
    console.log(`[${icon}] ${ds.key} = ${ds.value.toFixed(2)} (warn=${ds.warn}, crit=${ds.crit})`);
  }
}

main().catch(console.error);
