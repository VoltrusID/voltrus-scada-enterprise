# Voltrus Python SDK

Official Python SDK for the Voltrus SCADA system.

## Install

```bash
pip install voltrus
```

## Quick Start

```python
from voltrus import ApiClient, Configuration
from voltrus.api.data_api import DataApi
from voltrus.api.alarms_api import AlarmsApi

config = Configuration(host="http://192.168.1.100:3000")
config.api_key["api_key"] = "vt_sk_abc123"

with ApiClient(config) as client:
    data = DataApi(client).get_data(screen="overview")
    for ds in data:
        print(f"{ds.key}: {ds.value}")

    alarms = AlarmsApi(client).list_alarms(state="active")
    print(f"Active alarms: {len(alarms)}")
```

## License

MIT
