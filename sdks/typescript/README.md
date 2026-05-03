# Voltrus TypeScript SDK

Official TypeScript SDK for the Voltrus SCADA system.

## Install

```bash
npm install voltrus
```

## Quick Start

```typescript
import { Configuration, DataApi, AlarmsApi } from "voltrus";

const config = new Configuration({
  basePath: "http://192.168.1.100:3000",
  apiKey: "vt_sk_abc123",
});

const dataApi = new DataApi(config);
const data = await dataApi.getData({ screen: "overview" });
data.forEach((ds) => console.log(`${ds.key}: ${ds.value}`));

const alarmsApi = new AlarmsApi(config);
const alarms = await alarmsApi.listAlarms({ state: "active" });
console.log(`Active alarms: ${alarms.length}`);
```

## License

MIT
