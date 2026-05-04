"""Subscribe to real-time alarms from Voltrus.

Usage: python subscribe_alarms.py [host] [api_key]

Note: This uses polling since SSE is browser-native. For real-time,
use the TypeScript SDK or connect to the WebSocket endpoint directly.
"""

import sys
import time
from voltrus import VoltrusClient

host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
api_key = sys.argv[2] if len(sys.argv) > 2 else ""

client = VoltrusClient(host, api_key=api_key or None)

# Get alarm summary
summary = client.get_alarm_summary()
print("=== Alarm Summary ===")
print(f"  Active: {summary.active} (Critical: {summary.critical}, Warning: {summary.warning}, Info: {summary.info})")
print(f"  Acknowledged: {summary.acknowledged}")
print(f"  Shelved: {summary.shelved}")

# List active alarms
alarms = client.list_alarms(state="active", limit=10)
print("\n=== Recent Active Alarms ===")
for alarm in alarms:
    print(f"  [{alarm.severity.upper()}] {alarm.source_name}: {alarm.message}")

# Poll for new alarms
print("\n=== Polling for new alarms (Ctrl+C to exit) ===")
seen_ids = {a.id for a in alarms}
try:
    while True:
        time.sleep(5)
        current = client.list_alarms(state="active", limit=50)
        for alarm in current:
            if alarm.id not in seen_ids:
                seen_ids.add(alarm.id)
                print(f"⚡ [{alarm.severity.upper()}] {alarm.source_name}: {alarm.message}")
except KeyboardInterrupt:
    print("\nStopped.")
