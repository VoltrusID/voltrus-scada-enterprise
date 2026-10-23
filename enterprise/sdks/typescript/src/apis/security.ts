import type { HttpClient } from 'voltrus';
import type {
  ExportSecurityEventsQuery,
  ListSecurityEventsQuery,
  SecurityEventsResponse,
  SecurityThresholds,
} from '../types';

/** IEC 62443 security audit. Admin only, Enterprise tier. */
export class SecurityApi {
  constructor(private readonly http: HttpClient) {}

  async listEvents(query: ListSecurityEventsQuery = {}): Promise<SecurityEventsResponse> {
    return this.http.get('/api/v1/security/events', query);
  }

  /** Security events as CSV text. */
  async exportCsv(query: ExportSecurityEventsQuery = {}): Promise<string> {
    const res = await this.http.getRaw('/api/v1/security/events/export', {
      params: query,
      responseType: 'text',
    });
    return res.data as string;
  }

  async getThresholds(): Promise<SecurityThresholds> {
    return this.http.get('/api/v1/security/thresholds');
  }

  /** Replaces all six values. Each threshold must be greater than 0. */
  async updateThresholds(input: SecurityThresholds): Promise<{ ok: boolean }> {
    return this.http.put('/api/v1/security/thresholds', input);
  }
}
