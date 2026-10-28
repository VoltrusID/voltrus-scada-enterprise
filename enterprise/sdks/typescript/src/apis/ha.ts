import type { HttpClient } from 'voltrus';
import type { HaFailoverResponse, HaStatus, HaSyncResponse } from '../types';

/** Hot standby. One Enterprise licence covers both nodes of a pair. */
export class HaApi {
  constructor(private readonly http: HttpClient) {}

  async getStatus(): Promise<HaStatus> {
    return this.http.get('/api/v1/ha/status');
  }

  /**
   * Run a full resynchronisation of the standby from a fresh snapshot of the
   * primary. Admin only. Resolves with the report when it finishes within
   * `waitSecs` (default 30), or `sync_in_progress` when it is still running.
   * 400 when HA is not enabled, 409 when there is no peer to sync with.
   */
  async forceSync(options: { waitSecs?: number } = {}): Promise<HaSyncResponse> {
    return this.http.post(`/api/v1/ha/force-sync${query(options)}`);
  }

  /**
   * Switch the roles of the pair. Admin only. On the primary it is a controlled
   * switchover; on a standby linked to its primary it asks the primary to hand
   * over; on a standby without a link it takes over (`force` skips the data and
   * reachability checks). Resolves once the roles have changed. 409 with the
   * reason when it cannot.
   */
  async forceFailover(
    options: { waitSecs?: number; force?: boolean } = {},
  ): Promise<HaFailoverResponse> {
    return this.http.post(`/api/v1/ha/force-failover${query(options)}`);
  }
}

function query(options: { waitSecs?: number; force?: boolean }): string {
  const parts: string[] = [];
  if (options.waitSecs !== undefined) parts.push(`wait_secs=${options.waitSecs}`);
  if (options.force) parts.push('force=true');
  return parts.length ? `?${parts.join('&')}` : '';
}
