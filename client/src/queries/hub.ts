import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchJson, fetchOk } from './fetch'

export interface HubSite {
  id: number
  name: string
  url: string
  api_key: string
  enabled: boolean
  last_seen: number | null
  created_at: number
}

export interface HubSiteStatus {
  site_id: number
  name: string
  running: boolean
  last_seen: number | null
}

export function useHubSites() {
  return useQuery({
    queryKey: ['hub-sites'],
    queryFn: () => fetchJson<HubSite[]>('/api/v1/hub/sites'),
    refetchInterval: 15_000,
  })
}

export function useHubStatus() {
  return useQuery({
    queryKey: ['hub-status'],
    queryFn: () => fetchJson<HubSiteStatus[]>('/api/v1/hub/status'),
    refetchInterval: 10_000,
  })
}

export function useCreateHubSite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: { name: string; url: string; api_key: string; enabled?: boolean }) => {
      const r = await fetchOk('/api/v1/hub/sites', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['hub-sites'] })
      qc.invalidateQueries({ queryKey: ['hub-status'] })
    },
  })
}

export function useUpdateHubSite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, ...data }: Partial<HubSite> & { id: number }) => {
      await fetchOk(`/api/v1/hub/sites/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['hub-sites'] })
      qc.invalidateQueries({ queryKey: ['hub-status'] })
    },
  })
}

export function useDeleteHubSite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/v1/hub/sites/${id}`, { method: 'DELETE' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['hub-sites'] })
      qc.invalidateQueries({ queryKey: ['hub-status'] })
    },
  })
}

export function useStartHubSite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/v1/hub/sites/${id}/start`, { method: 'POST' })
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['hub-status'] }),
  })
}

export function useStopHubSite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/v1/hub/sites/${id}/stop`, { method: 'POST' })
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['hub-status'] }),
  })
}
