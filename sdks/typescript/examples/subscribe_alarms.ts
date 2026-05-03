const host = process.argv[2] || "http://localhost:3000";
const apiKey = process.argv[3] || "";
const url = `${host}/api/v1/stream`;

console.log(`Connecting to ${url}...`);

const headers: Record<string, string> = {};
if (apiKey) headers["X-API-Key"] = apiKey;

fetch(url, { headers })
  .then((res) => {
    const reader = res.body?.getReader();
    const decoder = new TextDecoder();

    function read(): Promise<void> {
      return reader!.read().then(({ done, value }) => {
        if (done) return;
        const text = decoder.decode(value);
        for (const line of text.split("\n")) {
          if (line.startsWith("data:")) {
            try {
              const event = JSON.parse(line.slice(5));
              if (event.type === "alarm") {
                const alarm = event.data;
                console.log(`[ALARM] ${(alarm.severity || "?").toUpperCase()}: ${alarm.message || "N/A"}`);
              }
            } catch {}
          }
        }
        return read();
      });
    }
    return read();
  })
  .catch(console.error);
