"""Subscribe to real-time alarm events via SSE."""
import sys
import json
import urllib.request

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3000"
    api_key = sys.argv[2] if len(sys.argv) > 2 else ""
    url = f"{host}/api/v1/stream"

    req = urllib.request.Request(url)
    if api_key:
        req.add_header("X-API-Key", api_key)

    print(f"Connecting to {url}...")
    with urllib.request.urlopen(req) as response:
        for line in response:
            line = line.decode("utf-8").strip()
            if line.startswith("data:"):
                payload = line[5:].strip()
                try:
                    event = json.loads(payload)
                    if event.get("type") == "alarm":
                        alarm = event.get("data", {})
                        print(f"[ALARM] {alarm.get('severity', '?').upper()}: {alarm.get('message', 'N/A')}")
                except json.JSONDecodeError:
                    pass

if __name__ == "__main__":
    main()
