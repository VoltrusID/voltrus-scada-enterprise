"""Write a setpoint to a tag and track command acknowledgment."""
import sys
import time
from voltrus import ApiClient, Configuration
from voltrus.api.commands_api import CommandsApi

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3000"
    api_key = sys.argv[2] if len(sys.argv) > 2 else ""
    tag_id = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    value = sys.argv[4] if len(sys.argv) > 4 else "1"

    config = Configuration(host=host)
    if api_key:
        config.api_key["api_key"] = api_key

    with ApiClient(config) as client:
        api = CommandsApi(client)
        result = api.write_tag(tag_id, {"value": value})
        command_id = result.get("command_id")
        print(f"Command submitted: {command_id}")

        for _ in range(30):
            status = api.get_command_status(command_id)
            state = status.status
            print(f"  Status: {state}")
            if state in ("acked", "resolved"):
                print("Command acknowledged!")
                return
            if state == "failed":
                print(f"Command failed: {status.error_message}")
                return
            time.sleep(1)

        print("Timeout waiting for acknowledgment")

if __name__ == "__main__":
    main()
