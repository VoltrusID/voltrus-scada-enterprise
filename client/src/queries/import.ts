import { useMutation } from '@tanstack/react-query'
import { fetchJson } from './fetch'

export interface CsvImportResult {
  total_rows: number
  imported: number
  skipped: number
  errors: { row?: number; error: string; tag?: string }[]
  dry_run: boolean
  mode: string
  column_mapping: Record<string, number>
}

export interface CsvImportParams {
  file: File
  dryRun?: boolean
  mode?: 'merge' | 'replace'
  delimiter?: string
}

export function useImportCsv() {
  return useMutation({
    mutationFn: async ({ file, dryRun, mode, delimiter }: CsvImportParams) => {
      const sp = new URLSearchParams()
      if (dryRun) sp.set('dry_run', 'true')
      if (mode) sp.set('mode', mode)
      if (delimiter) sp.set('delimiter', delimiter)
      const qs = sp.toString()

      const formData = new FormData()
      formData.append('file', file)

      const res = await fetch(`/api/v1/import/csv${qs ? '?' + qs : ''}`, {
        method: 'POST',
        credentials: 'include',
        body: formData,
      })
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `Import failed: ${res.status}`)
      }
      return res.json() as Promise<CsvImportResult>
    },
  })
}

export interface ExcelImportParams {
  file: File
  dryRun?: boolean
  mode?: 'merge' | 'replace'
  sheet?: string
}

export interface ExcelImportResult extends CsvImportResult {
  sheet: string
  available_sheets: string[]
}

export function useImportExcel() {
  return useMutation({
    mutationFn: async ({ file, dryRun, mode, sheet }: ExcelImportParams) => {
      const sp = new URLSearchParams()
      if (dryRun) sp.set('dry_run', 'true')
      if (mode) sp.set('mode', mode)
      if (sheet) sp.set('sheet', sheet)
      const qs = sp.toString()

      const formData = new FormData()
      formData.append('file', file)

      const res = await fetch(`/api/v1/import/excel${qs ? '?' + qs : ''}`, {
        method: 'POST',
        credentials: 'include',
        body: formData,
      })
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `Import failed: ${res.status}`)
      }
      return res.json() as Promise<ExcelImportResult>
    },
  })
}
