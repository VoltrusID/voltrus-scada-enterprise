import { useQuery, useMutation, useQueryClient, keepPreviousData } from '@tanstack/react-query'
import type { SensorConfig, Screen, ScreenLayouts, DataPoint, Alarm, CanvasLayoutItem } from './types'
import type { BrandingSettings } from './BrandingDialog'

// ── Query key factory ──────────────────────────────────────────────

export const queryKeys = {
  sensors:       ['dataSources'] as const,
  screenSensors: (screenId: string) => ['screenDataSources', screenId] as const,
  screens:       ['screens'] as const,
  layouts:       (screenId: string) => ['layouts', screenId] as const,
  twinLayouts:   (screenId: string) => ['twinLayouts', screenId] as const,
  shortcuts:     ['shortcuts'] as const,
  branding:      ['branding'] as const,
  history:       (range: string, screenId: string) => ['history', range, screenId] as const,
  systemInfo:    ['systemInfo'] as const,
  alarms:        (status: string) => ['alarms', status] as const,
  canvasLayouts: (screenId: string) => ['canvasLayouts', screenId] as const,
  twinConnections: (screenId: string) => ['twinConnections', screenId] as const,
}

// ── Fetcher helper ─────────────────────────────────────────────────

async function fetchJson<T>(url: string): Promise<T> {
  const r = await fetch(url)
  if (!r.ok) throw new Error(`fetch ${url} failed: ${r.status}`)
  return r.json()
}

async function fetchOk(url: string, init?: RequestInit): Promise<Response> {
  const r = await fetch(url, init)
  if (!r.ok) {
    const body = await r.json().catch(() => ({}))
    throw new Error(body.error || `Request failed: ${r.status}`)
  }
  return r
}

// ── Query hooks ────────────────────────────────────────────────────

export function useSensors(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.sensors,
    queryFn: () => fetchJson<SensorConfig[]>('/api/data-sources'),
    enabled,
  })
}

export function useScreenSensors(screenId: string, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.screenSensors(screenId),
    queryFn: () => fetchJson<SensorConfig[]>(
      screenId === 'all' ? '/api/data-sources' : `/api/screens/${screenId}/data-sources`
    ),
    enabled,
    placeholderData: keepPreviousData,
  })
}

export function useScreens(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.screens,
    queryFn: () => fetchJson<Screen[]>('/api/screens'),
    enabled,
  })
}

export function useLayouts(screenId: string, enabled: boolean) {
  const actualId = screenId === 'all' ? '__overview' : screenId
  return useQuery({
    queryKey: queryKeys.layouts(screenId),
    queryFn: async () => {
      const data = await fetchJson<any>(`/api/screens/${actualId}/layouts`)
      return { lg: data.lg || [], md: data.md || [], sm: data.sm || [] } as ScreenLayouts
    },
    enabled,
    placeholderData: keepPreviousData,
  })
}

export function useTwinLayouts(screenId: string, enabled: boolean) {
  const actualId = screenId === 'all' ? '__overview' : screenId
  return useQuery({
    queryKey: queryKeys.twinLayouts(screenId),
    queryFn: () => fetchJson<any[]>(`/api/screens/${actualId}/twin-layouts`),
    enabled,
  })
}

export function useCanvasLayouts(screenId: string, enabled: boolean) {
  const actualId = screenId === 'all' ? '__overview' : screenId
  return useQuery({
    queryKey: queryKeys.canvasLayouts(screenId),
    queryFn: () => fetchJson<CanvasLayoutItem[]>(`/api/screens/${actualId}/canvas-layouts`),
    enabled,
  })
}

export function useShortcuts(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.shortcuts,
    queryFn: () => fetchJson<Record<number, string>>('/api/shortcuts'),
    enabled,
  })
}

export function useBranding() {
  return useQuery({
    queryKey: queryKeys.branding,
    queryFn: () => fetchJson<BrandingSettings>('/api/settings/branding'),
  })
}

export function useHistory(range: string, screenId: string, enabled: boolean) {
  const dp = screenId === 'all' ? '' : `&screen=${screenId}`
  return useQuery({
    queryKey: queryKeys.history(range, screenId),
    queryFn: async () => {
      const result = await fetchJson<{ data: Record<string, DataPoint[]>; sensors: any[] }>(`/api/history?range=${range}${dp}`)
      return result.data ?? {}
    },
    enabled,
    staleTime: 10_000,
  })
}

export function useSystemInfo(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.systemInfo,
    queryFn: () => fetchJson<any>('/api/system'),
    enabled,
    refetchInterval: 5_000,
    staleTime: 4_000,
  })
}

// ── Mutation hooks ─────────────────────────────────────────────────

export function useUpdateSensor() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (s: SensorConfig) => {
      await fetchOk(`/api/data-sources/${s.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(s),
      })
      return s
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.sensors })
      qc.invalidateQueries({ queryKey: ['screenDataSources'] })
    },
  })
}

export function useDeleteSensor() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/data-sources/${id}`, { method: 'DELETE' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.sensors })
      qc.invalidateQueries({ queryKey: ['screenDataSources'] })
    },
  })
}

export function useAddSensor() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (form: Partial<SensorConfig>) => {
      const res = await fetchOk('/api/data-sources', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(form),
      })
      return res.json() as Promise<{ id: number }>
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.sensors })
    },
  })
}

export function useSaveLayout() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, layouts }: { screenId: string; layouts: ScreenLayouts }) => {
      const actualId = screenId === 'all' ? '__overview' : screenId
      await fetchOk(`/api/screens/${actualId}/layouts`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(layouts),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.layouts(screenId) })
    },
  })
}

export function useSaveTwinLayout() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, items }: { screenId: string; items: any[] }) => {
      const actualId = screenId === 'all' ? '__overview' : screenId
      await fetchOk(`/api/screens/${actualId}/twin-layouts`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(items),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.twinLayouts(screenId) })
    },
  })
}

export function useTwinConnections(screenId: string, enabled: boolean) {
  const actualId = screenId === 'all' ? '__overview' : screenId
  return useQuery({
    queryKey: queryKeys.twinConnections(screenId),
    queryFn: () => fetchJson<{ from_id: string; to_id: string }[]>(`/api/screens/${actualId}/twin-connections`),
    enabled,
  })
}

export function useSaveTwinConnections() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, connections }: { screenId: string; connections: { from_id: string; to_id: string }[] }) => {
      const actualId = screenId === 'all' ? '__overview' : screenId
      await fetchOk(`/api/screens/${actualId}/twin-connections`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(connections),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.twinConnections(screenId) })
    },
  })
}

export function useSaveCanvasLayout() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, layouts }: { screenId: string; layouts: CanvasLayoutItem[] }) => {
      const actualId = screenId === 'all' ? '__overview' : screenId
      await fetchOk(`/api/screens/${actualId}/canvas-layouts`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(layouts),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.canvasLayouts(screenId) })
    },
  })
}

export function useSaveShortcuts() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (shortcuts: Record<number, string | null>) => {
      await fetchOk('/api/shortcuts', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(shortcuts),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.shortcuts })
    },
  })
}

export function useSaveBranding() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (settings: BrandingSettings) => {
      await fetchOk('/api/settings/branding', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings),
      })
      return settings
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.branding })
    },
  })
}

export function useCreateScreen() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (name: string) => {
      await fetchOk('/api/screens', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.screens })
    },
  })
}

export function useRenameScreen() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, name }: { id: string; name: string }) => {
      await fetchOk(`/api/screens/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.screens })
    },
  })
}

export function useDeleteScreen() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await fetchOk(`/api/screens/${id}`, { method: 'DELETE' })
      return id
    },
    onSuccess: (deletedId) => {
      qc.invalidateQueries({ queryKey: queryKeys.screens })
      qc.invalidateQueries({ queryKey: queryKeys.layouts(deletedId) })
      qc.invalidateQueries({ queryKey: ['screenDataSources'] })
    },
  })
}

export function useAddSensorToScreen() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, sensorId }: { screenId: string; sensorId: number }) => {
      await fetchOk(`/api/screens/${screenId}/data-sources`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ data_source_id: sensorId }),
      })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.screenSensors(screenId) })
      qc.invalidateQueries({ queryKey: queryKeys.layouts(screenId) })
    },
  })
}

export function useRemoveSensorFromScreen() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ screenId, sensorId }: { screenId: string; sensorId: number }) => {
      await fetchOk(`/api/screens/${screenId}/data-sources/${sensorId}`, { method: 'DELETE' })
    },
    onSuccess: (_, { screenId }) => {
      qc.invalidateQueries({ queryKey: queryKeys.screenSensors(screenId) })
      qc.invalidateQueries({ queryKey: queryKeys.layouts(screenId) })
    },
  })
}

// ── Alarm hooks ─────────────────────────────────────────────────────

export function useAlarms(status: string = 'all', enabled: boolean = true) {
  return useQuery({
    queryKey: queryKeys.alarms(status),
    queryFn: () => fetchJson<Alarm[]>(`/api/alarms?status=${status}&limit=50`),
    enabled,
    staleTime: 5_000,
  })
}

export function useAcknowledgeAlarm() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/alarms/${id}/acknowledge`, { method: 'POST' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['alarms'] })
    },
  })
}

export function useAcknowledgeAllAlarms() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      await fetchOk('/api/alarms', { method: 'POST' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['alarms'] })
    },
  })
}
