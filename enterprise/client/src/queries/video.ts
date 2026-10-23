import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'

export interface Camera {
  id: number
  name: string
  url: string
  enabled: boolean
  snapshot_on_alarm: boolean
  retention_hours: number
  created_at: number
}

export interface CameraSnapshot {
  id: number
  camera_id: number
  timestamp: number
  alarm_source: string | null
  file_path: string
  file_size: number
}

export interface CreateCameraRequest {
  name: string
  url: string
  enabled?: boolean
  snapshot_on_alarm?: boolean
  retention_hours?: number
}

export interface UpdateCameraRequest {
  name?: string
  url?: string
  enabled?: boolean
  snapshot_on_alarm?: boolean
  retention_hours?: number
}

export interface StreamInfo {
  url: string
  protocol: string
}

export function useCameras() {
  return useQuery({
    queryKey: ['cameras'],
    queryFn: () => fetchJson<Camera[]>('/api/v1/cameras'),
    refetchInterval: 15_000,
  })
}

export function useCamera(id: number) {
  return useQuery({
    queryKey: ['cameras'],
    queryFn: () => fetchJson<Camera[]>('/api/v1/cameras'),
    select: (data) => data.find(c => c.id === id) ?? null,
    enabled: id > 0,
  })
}

export function useCreateCamera() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateCameraRequest) => {
      const r = await fetchOk('/api/v1/cameras', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['cameras'] })
    },
  })
}

export function useUpdateCamera() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, ...data }: UpdateCameraRequest & { id: number }) => {
      await fetchOk(`/api/v1/cameras/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['cameras'] })
    },
  })
}

export function useDeleteCamera() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/v1/cameras/${id}`, { method: 'DELETE' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['cameras'] })
    },
  })
}

export function useCameraSnapshots(cameraId: number) {
  return useQuery({
    queryKey: ['cameras', cameraId, 'snapshots'],
    queryFn: () => fetchJson<CameraSnapshot[]>(`/api/v1/cameras/${cameraId}/snapshots`),
    enabled: cameraId > 0,
    refetchInterval: 30_000,
  })
}

export function useCaptureSnapshot(cameraId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const r = await fetchOk(`/api/v1/cameras/${cameraId}/snapshot`, { method: 'POST' })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['cameras', cameraId, 'snapshots'] })
    },
  })
}

export function useDeleteSnapshot() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ cameraId, snapshotId }: { cameraId: number; snapshotId: number }) => {
      await fetchOk(`/api/v1/cameras/${cameraId}/snapshots/${snapshotId}`, { method: 'DELETE' })
    },
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({ queryKey: ['cameras', variables.cameraId, 'snapshots'] })
    },
  })
}

export function useStreamInfo(cameraId: number) {
  return useQuery({
    queryKey: ['cameras', cameraId, 'stream'],
    queryFn: () => fetchJson<StreamInfo>(`/api/v1/cameras/${cameraId}/stream`),
    enabled: cameraId > 0,
    refetchInterval: 60_000,
  })
}
