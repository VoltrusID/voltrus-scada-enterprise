import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'

import { normalizeReport, type Report } from './reportModel'
export type { Report } from './reportModel'

export interface CreateReportInput {
  name: string
  schedule: string
  device_ids?: string[]
  email_recipients?: string[]
  enabled?: boolean
}

export interface UpdateReportInput {
  name?: string
  schedule?: string
  device_ids?: string[]
  email_recipients?: string[]
  enabled?: boolean
}

export function useReports() {
  return useQuery({
    queryKey: ['reports'],
    queryFn: async (): Promise<Report[]> => {
      const rows = await fetchJson<unknown[]>('/api/v1/reports')
      return Array.isArray(rows) ? rows.map(normalizeReport) : []
    },
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

/** Download a preview PDF (does not move the schedule or send mail). */
export function useDownloadReportPdf() {
  return useMutation({
    mutationFn: async ({ id, name }: { id: number; name: string }) => {
      const r = await fetchOk(`/api/v1/reports/${id}/generate?format=pdf`, { method: 'POST' })
      const blob = await r.blob()
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${name.replace(/[^\w.-]+/g, '_') || 'report'}.pdf`
      a.click()
      setTimeout(() => URL.revokeObjectURL(url), 1000)
    },
  })
}
