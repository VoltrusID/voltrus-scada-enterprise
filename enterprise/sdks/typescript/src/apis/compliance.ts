import type { HttpClient } from 'voltrus';
import type {
  ComplianceConfig,
  CreateSignatureRequest,
  CreateSignatureResponse,
  ElectronicSignature,
  ListSignaturesQuery,
  UpdateComplianceConfigRequest,
} from '../types';

/** 21 CFR Part 11: compliance settings and electronic signatures. Admin only, Enterprise tier. */
export class ComplianceApi {
  constructor(private readonly http: HttpClient) {}

  async getConfig(): Promise<ComplianceConfig> {
    return this.http.get('/api/v1/compliance/config');
  }

  /** Only the fields you send change. */
  async updateConfig(input: UpdateComplianceConfigRequest): Promise<{ ok: boolean }> {
    return this.http.put('/api/v1/compliance/config', input);
  }

  /** Sign a resource as the logged-in user. `reason` and `meaning` must not be blank. */
  async sign(input: CreateSignatureRequest): Promise<CreateSignatureResponse> {
    return this.http.post('/api/v1/compliance/sign', input);
  }

  async listSignatures(query: ListSignaturesQuery = {}): Promise<ElectronicSignature[]> {
    return this.http.get('/api/v1/compliance/signatures', query);
  }
}
