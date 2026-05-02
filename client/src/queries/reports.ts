import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchJson, fetchOk } from './fetch'

export interface Report {
  id: number
  name: string
  schedule: string
  enabled: boolean
  device_ids: number[]
  email_recipients: string[]
  last_generated: string | null
  created_at: string
}

export interface CreateReportInput {
  name: string
  schedule: string
  device_ids?: number[]
  email_recipients?: string[]
  enabled?: boolean
}

export interface UpdateReportInput {
  name?: string
  schedule?: string
  device_ids?: number[]
  email_recipients?: string[]
  enabled?: boolean
}

export function useReports() {
  return useQuery({
    queryKey: ['reports'],
    queryFn: () => fetchJson<Report[]>('/api/v1/reports'),
  })
}

export function useCreateReport() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateReportInput) => {
      const r = await fetchOk('/api/v1/reports', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['reports'] })
    },
  })
}

export function useUpdateReport() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, ...data }: UpdateReportInput & { id: number }) => {
      await fetchOk(`/api/v1/reports/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['reports'] })
    },
  })
}

export function useDeleteReport() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/v1/reports/${id}`, { method: 'DELETE' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['reports'] })
    },
  })
}

export function useGenerateReport() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      const r = await fetchOk(`/api/v1/reports/${id}/generate`, { method: 'POST' })
      return r.text()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['reports'] })
    },
  })
}
