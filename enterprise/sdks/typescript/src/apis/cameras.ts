import type { HttpClient } from 'voltrus';
import type {
  Camera,
  CameraSnapshot,
  CreateCameraRequest,
  CreateCameraResponse,
  StreamInfo,
  TriggerSnapshotResponse,
  UpdateCameraRequest,
} from '../types';

/** RTSP cameras and alarm snapshots. Admin only, Enterprise tier. */
export class CamerasApi {
  constructor(private readonly http: HttpClient) {}

  async list(): Promise<Camera[]> {
    return this.http.get('/api/v1/cameras');
  }

  async create(input: CreateCameraRequest): Promise<CreateCameraResponse> {
    return this.http.post('/api/v1/cameras', input);
  }

  async update(id: number, input: UpdateCameraRequest): Promise<{ ok: boolean }> {
    return this.http.put(`/api/v1/cameras/${id}`, input);
  }

  async delete(id: number): Promise<{ ok: boolean }> {
    return this.http.delete(`/api/v1/cameras/${id}`);
  }

  async getStreamInfo(id: number): Promise<StreamInfo> {
    return this.http.get(`/api/v1/cameras/${id}/stream`);
  }

  /** Capture a snapshot now. The camera must be enabled. */
  async triggerSnapshot(id: number): Promise<TriggerSnapshotResponse> {
    return this.http.post(`/api/v1/cameras/${id}/snapshot`);
  }

  /** Latest 100 snapshots of a camera. */
  async listSnapshots(id: number): Promise<CameraSnapshot[]> {
    return this.http.get(`/api/v1/cameras/${id}/snapshots`);
  }

  async deleteSnapshot(cameraId: number, snapshotId: number): Promise<{ ok: boolean }> {
    return this.http.delete(`/api/v1/cameras/${cameraId}/snapshots/${snapshotId}`);
  }
}
