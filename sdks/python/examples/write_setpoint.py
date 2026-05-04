"""Write a setpoint to a Voltrus tag.

Usage: python write_setpoint.py [host] [api_key] [tag_id] [value]
"""

import sys
import time
from voltrus import VoltrusClient

host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
api_key = sys.argv[2] if len(sys.argv) > 2 else ""
tag_id = sys.argv[3] if len(sys.argv) > 3 else "sensor-001"
value = float(sys.argv[4]) if len(sys.argv) > 4 else 42.5

client = VoltrusClient(host, api_key=api_key or None)

# Write a setpoint
result = client.write_tag(tag_id, value)
print(f"Write accepted: command_id={result.command_id}, status={result.status}")

# Poll for acknowledgement
for i in range(10):
    time.sleep(1)
    status = client.get_command_status(result.command_id)
    print(f"  [{i + 1}s] status={status.status}")
    if status.status in ("acked", "failed"):
        break
