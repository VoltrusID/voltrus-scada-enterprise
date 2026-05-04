import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';

// ── Types ──────────────────────────────────────────────────────

export interface VoltrusConfig {
  baseURL: string;
  apiKey?: string;
  token?: string;
  timeout?: number;
}

export interface TagValue {
  id: string;
  name: string;
  value: number | string | boolean | null;
  quality: 'good' | 'bad' | 'unknown';
  timestamp: string;
  device_id: string;
}

export interface WriteResult {
  command_id: string;
  status: 'pending' | 'sent' | 'acked' | 'failed';
  message: string;
}

export interface CommandStatus {
  command_id: string;
  tag_id: string;
  value: number | string | boolean;
  status: 'pending' | 'sent' | 'acked' | 'failed' | 'cancelled';
  created_at: string;
  updated_at: string;
  performed_by: string;
  plc_response?: string;
}

export interface Alarm {
  id: string;
  alarm_id: string;
  source_key: string;
  source_name: string;
  severity: 'critical' | 'warning' | 'info';
  message: string;
  state: 'active' | 'acknowledged' | 'shelved';
  triggered_at: string;
  acknowledged_at?: string;
  acknowledged_by?: string;
}

export interface AlarmSummary {
  active: number;
  acknowledged: number;
  shelved: number;
  critical: number;
  warning: number;
  info: number;
}

export interface Device {
  id: string;
  name: string;
  protocol: 'modbus' | 'opcua' | 's7' | 'enip' | 'dnp3' | 'bacnet' | 'mqtt';
  enabled: boolean;
  online: boolean;
  ip?: string;
  port?: number;
}

export interface Screen {
  id: string;
  name: string;
  created_at: string;
}

export interface OeeMetrics {
  availability: number;
  performance: number;
  quality: number;
  oee: number;
  period_start: string;
  period_end: string;
}

export interface DataPoint {
  timestamp: string;
  value: number | string | boolean;
  quality: 'good' | 'bad' | 'unknown';
}

export interface PaginatedResult<T> {
  data: T[];
  total: number;
  offset: number;
  limit: number;
}

// ── Client ─────────────────────────────────────────────────────

export class VoltrusClient {
  private http: AxiosInstance;

  constructor(config: VoltrusConfig) {
    const headers: Record<string, string> = {};
    if (config.apiKey) {
      headers['X-API-Key'] = config.apiKey;
    }
    if (config.token) {
      headers['Authorization'] = `Bearer ${config.token}`;
    }

    this.http = axios.create({
      baseURL: config.baseURL.replace(/\/$/, ''),
      timeout: config.timeout ?? 10000,
      headers,
    });
  }

  // ── Auth ───────────────────────────────────────────────────

  /** Login and store the returned token for subsequent requests. */
  async login(username: string, password: string): Promise<string> {
    const res = await this.http.post('/api/v1/auth/login', { username, password });
    const token = res.data.token;
    this.http.defaults.headers.common['Authorization'] = `Bearer ${token}`;
    return token;
  }

  // ── Tags / Data ───────────────────────────────────────────

  /** Read current tag values for a screen. */
  async getLiveData(screenId: string): Promise<TagValue[]> {
    const res = await this.http.get(`/api/v1/screens/${screenId}/data-sources`);
    return res.data;
  }

  /** Write a value to a tag. Returns a command_id for tracking. */
  async writeTag(tagId: string, value: number | string | boolean): Promise<WriteResult> {
    const res = await this.http.post(`/api/v1/tags/${tagId}/write`, { value });
    return res.data;
  }

  /** Get command status by command_id. */
  async getCommandStatus(commandId: string): Promise<CommandStatus> {
    const res = await this.http.get(`/api/v1/commands/${commandId}`);
    return res.data;
  }

  /** Get command history with optional filters. */
  async getCommands(params?: { tag_id?: string; status?: string; limit?: number }): Promise<CommandStatus[]> {
    const res = await this.http.get('/api/v1/commands', { params });
    return res.data;
  }

  /** Cancel a pending command. */
  async cancelCommand(commandId: string): Promise<void> {
    await this.http.post(`/api/v1/commands/${commandId}/cancel`);
  }

  /** Query historical data for a tag. */
  async getHistory(params: {
    device_id: string;
    tag_name: string;
    from: string;
    to: string;
    limit?: number;
  }): Promise<DataPoint[]> {
    const res = await this.http.get('/api/v1/history', { params });
    return res.data;
  }

  /** Export data as CSV. */
  async exportCsv(params: {
    device_id: string;
    tag_name: string;
    from: string;
    to: string;
  }): Promise<string> {
    const res = await this.http.get('/api/v1/export/csv', { params, responseType: 'text' });
    return res.data;
  }

  // ── Alarms ────────────────────────────────────────────────

  /** List alarms with optional filters. */
  async listAlarms(params?: { state?: string; severity?: string; limit?: number }): Promise<Alarm[]> {
    const res = await this.http.get('/api/v1/alarms', { params });
    return res.data;
  }

  /** Get alarm summary counts. */
  async getAlarmSummary(): Promise<AlarmSummary> {
    const res = await this.http.get('/api/v1/alarms/summary');
    return res.data;
  }

  /** Acknowledge a specific alarm. */
  async acknowledgeAlarm(alarmId: string): Promise<void> {
    await this.http.post(`/api/v1/alarms/${alarmId}/acknowledge`);
  }

  /** Acknowledge all active alarms. */
  async acknowledgeAllAlarms(): Promise<void> {
    await this.http.post('/api/v1/alarms');
  }

  /** Shelf an alarm (temporarily suppress). */
  async shelfAlarm(alarmId: string, durationMinutes: number): Promise<void> {
    await this.http.post(`/api/v1/alarms/${alarmId}/shelf`, { duration_minutes: durationMinutes });
  }

  /** Unshelf an alarm. */
  async unshelfAlarm(alarmId: string): Promise<void> {
    await this.http.post(`/api/v1/alarms/${alarmId}/unshelf`);
  }

  /** Subscribe to alarms via SSE. */
  subscribeAlarms(onAlarm: (alarm: Alarm) => void): EventSource {
    const url = `${this.http.defaults.baseURL}/api/v1/stream`;
    const es = new EventSource(url);
    es.addEventListener('alarm', (event) => {
      onAlarm(JSON.parse(event.data));
    });
    return es;
  }

  // ── Devices ───────────────────────────────────────────────

  /** List Modbus devices. */
  async listModbusDevices(): Promise<Device[]> {
    const res = await this.http.get('/api/v1/modbus-devices');
    return res.data;
  }

  /** List OPC-UA devices. */
  async listOpcuaDevices(): Promise<Device[]> {
    const res = await this.http.get('/api/v1/opcua/devices');
    return res.data;
  }

  /** List S7 devices. */
  async listS7Devices(): Promise<Device[]> {
    const res = await this.http.get('/api/v1/s7/devices');
    return res.data;
  }

  /** List EtherNet/IP devices. */
  async listEnipDevices(): Promise<Device[]> {
    const res = await this.http.get('/api/v1/enip/devices');
    return res.data;
  }

  /** List DNP3 devices. */
  async listDnp3Devices(): Promise<Device[]> {
    const res = await this.http.get('/api/v1/dnp3/devices');
    return res.data;
  }

  /** List BACnet devices. */
  async listBacnetDevices(): Promise<Device[]> {
    const res = await this.http.get('/api/v1/bacnet/devices');
    return res.data;
  }

  // ── OEE ───────────────────────────────────────────────────

  /** Get OEE metrics. */
  async getOeeMetrics(params?: { device_id?: string; from?: string; to?: string }): Promise<OeeMetrics> {
    const res = await this.http.get('/api/v1/oee/metrics', { params });
    return res.data;
  }

  // ── Screens ───────────────────────────────────────────────

  /** List all screens. */
  async listScreens(): Promise<Screen[]> {
    const res = await this.http.get('/api/v1/screens');
    return res.data;
  }

  // ── System ────────────────────────────────────────────────

  /** Health check. */
  async health(): Promise<{ status: string }> {
    const res = await this.http.get('/api/v1/health');
    return res.data;
  }

  /** Get system info. */
  async getSystemInfo(): Promise<Record<string, unknown>> {
    const res = await this.http.get('/api/v1/system');
    return res.data;
  }

  /** Get version. */
  async getVersion(): Promise<{ version: string }> {
    const res = await this.http.get('/api/v1/version');
    return res.data;
  }
}

export default VoltrusClient;
