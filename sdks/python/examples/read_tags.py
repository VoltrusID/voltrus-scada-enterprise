"""Read tag values from Voltrus SCADA."""
import sys
from voltrus import ApiClient, Configuration
from voltrus.api.data_api import DataApi

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3000"
    api_key = sys.argv[2] if len(sys.argv) > 2 else ""

    config = Configuration(host=host)
    if api_key:
        config.api_key["api_key"] = api_key

    with ApiClient(config) as client:
        data = DataApi(client).get_data(screen="overview")
        for ds in data:
            status_icon = {"normal": "OK", "warning": "WARN", "critical": "CRIT", "offline": "OFF"}.get(ds.status, "?")
            print(f"[{status_icon}] {ds.key} = {ds.value:.2f} (warn={ds.warn}, crit={ds.crit})")

if __name__ == "__main__":
    main()
