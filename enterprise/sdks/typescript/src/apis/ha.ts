import type { HttpClient } from 'voltrus';
import type { HaFailoverResponse, HaStatus, HaSyncResponse } from '../types';

/** Hot standby. One Enterprise licence covers both nodes of a pair. */
export class HaApi {
  constructor(private readonly http: HttpClient) {}

  async getStatus(): Promise<HaStatus> {
    return this.http.get('/api/v1/ha/status');
  }

  /** Ask the pair to sync now. 400 when HA is not enabled. */
  async forceSync(): Promise<HaSyncResponse> {
    return this.http.post('/api/v1/ha/force-sync');
  }

  /** Start a failover. 400 when HA is not enabled. */
  async forceFailover(): Promise<HaFailoverResponse> {
    return this.http.post('/api/v1/ha/force-failover');
  }
}
