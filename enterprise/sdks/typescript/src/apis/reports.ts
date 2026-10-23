import type { HttpClient } from 'voltrus';
import type { CreateReportRequest, Report, UpdateReportRequest } from '../types';

/** Scheduled reports. Integrator tier and above; a Core-only server returns 404. */
export class ReportsApi {
  constructor(private readonly http: HttpClient) {}

  async list(): Promise<Report[]> {
    return this.http.get('/api/v1/reports');
  }

  async create(input: CreateReportRequest): Promise<Report> {
    return this.http.post('/api/v1/reports', input);
  }

  async update(id: number, input: UpdateReportRequest): Promise<Report> {
    return this.http.put(`/api/v1/reports/${id}`, input);
  }

  async delete(id: number): Promise<void> {
    await this.http.delete(`/api/v1/reports/${id}`);
  }

  /** Generate a report now instead of waiting for its schedule. Returns the HTML document. */
  async generate(id: number): Promise<string> {
    const res = await this.http.postRaw(`/api/v1/reports/${id}/generate`);
    return typeof res.data === 'string' ? res.data : JSON.stringify(res.data);
  }
}
