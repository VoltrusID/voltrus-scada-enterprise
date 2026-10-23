import type { HttpClient } from 'voltrus';
import type {
  CreateHubSiteRequest,
  HubSite,
  HubStatus,
  UpdateHubSiteRequest,
} from '../types';

/** Multi-site hub: remote Voltrus sites this server pulls data from. Integrator tier and above. */
export class HubApi {
  constructor(private readonly http: HttpClient) {}

  async listSites(): Promise<HubSite[]> {
    return this.http.get('/api/v1/hub/sites');
  }

  async createSite(input: CreateHubSiteRequest): Promise<HubSite> {
    return this.http.post('/api/v1/hub/sites', input);
  }

  async updateSite(id: number, input: UpdateHubSiteRequest): Promise<void> {
    await this.http.put(`/api/v1/hub/sites/${id}`, input);
  }

  async deleteSite(id: number): Promise<void> {
    await this.http.delete(`/api/v1/hub/sites/${id}`);
  }

  /** Start the connection to a site. The site must be enabled. */
  async startSite(id: number): Promise<{ ok: boolean }> {
    return this.http.post(`/api/v1/hub/sites/${id}/start`);
  }

  async stopSite(id: number): Promise<{ ok: boolean }> {
    return this.http.post(`/api/v1/hub/sites/${id}/stop`);
  }

  /** Every site with its connection state. */
  async getStatus(): Promise<HubStatus> {
    return this.http.get('/api/v1/hub/status');
  }
}
