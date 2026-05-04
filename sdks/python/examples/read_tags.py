"""Read current tag values from Voltrus.

Usage: python read_tags.py [host] [api_key]
"""

import sys
from voltrus import VoltrusClient

host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
api_key = sys.argv[2] if len(sys.argv) > 2 else ""

client = VoltrusClient(host, api_key=api_key or None)

# List screens
screens = client.list_screens()
print(f"Found {len(screens)} screens")

if not screens:
    print("No screens found. Create one in the Voltrus UI first.")
    sys.exit(0)

# Read live data from first screen
data = client.get_live_data(screens[0].id)
for tag in data:
    icon = "OK" if tag.quality == "good" else "BAD" if tag.quality == "bad" else "???"
    print(f"[{icon}] {tag.name} = {tag.value}")
