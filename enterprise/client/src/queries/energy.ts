import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { fetchJson, fetchOk } from '@voltrus/core/queries/fetch'
import type {
  DayRow, Meter, MeterRole, MonthReport, PlnFigures, Site, SiteDetail, SiteSummary, TariffValues,
} from '../energy/model'

const KEY = ['energy'] as const

const json = (method: string, body: unknown): RequestInit => ({
  method,
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(body),
})

export function useEnergySites() {
  return useQuery({
    queryKey: [...KEY, 'sites'],
    queryFn: () => fetchJson<SiteSummary[]>('/api/v1/energy/sites'),
  })
}

export function useEnergySite(id: number | null) {
  return useQuery({
    queryKey: [...KEY, 'site', id],
    queryFn: () => fetchJson<SiteDetail>(`/api/v1/energy/sites/${id}`),
    enabled: id != null,
  })
}

export function useEnergyBill(id: number | null, month: string) {
  return useQuery({
    queryKey: [...KEY, 'bill', id, month],
    queryFn: () => fetchJson<MonthReport>(`/api/v1/energy/sites/${id}/bill?month=${encodeURIComponent(month)}`),
    enabled: id != null && month !== '',
    refetchInterval: 5 * 60_000,
  })
}

export function useEnergyDaily(id: number | null, month: string) {
  return useQuery({
    queryKey: [...KEY, 'daily', id, month],
    queryFn: () => fetchJson<{ month: string; days: DayRow[] }>(`/api/v1/energy/sites/${id}/daily?month=${encodeURIComponent(month)}`),
    enabled: id != null && month !== '',
    refetchInterval: 5 * 60_000,
  })
}

/** Every mutation refreshes all energy queries: they are cheap and interlinked. */
function useEnergyMutation<T, R = unknown>(fn: (input: T) => Promise<R>) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: fn,
    onSuccess: () => qc.invalidateQueries({ queryKey: KEY }),
  })
}

export interface SiteInput {
  name: string
  timezone: string
  wbp_windows: Site['wbp_windows']
  contracted_kva: number | null
  alerts: Site['alerts']
  tariff?: TariffInput
}

export interface TariffInput extends TariffValues {
  valid_from: string
  note?: string
}

export interface MeterInput {
  name: string
  device_id: string
  kwh_metric: string
  kvarh_metric: string | null
  kvar_metric: string | null
  role: MeterRole
  multiplier: number
  rollover: number | null
  max_gap_secs: number
  enabled: boolean
}

export const useCreateSite = () =>
  useEnergyMutation(async (input: SiteInput) => (await fetchOk('/api/v1/energy/sites', json('POST', input))).json() as Promise<SiteDetail>)

export const useUpdateSite = () =>
  useEnergyMutation(({ id, ...input }: SiteInput & { id: number }) => fetchOk(`/api/v1/energy/sites/${id}`, json('PUT', input)))

export const useDeleteSite = () =>
  useEnergyMutation((id: number) => fetchOk(`/api/v1/energy/sites/${id}`, { method: 'DELETE' }))

export const useSetTariff = () =>
  useEnergyMutation(({ siteId, ...input }: TariffInput & { siteId: number }) =>
    fetchOk(`/api/v1/energy/sites/${siteId}/tariffs`, json('POST', input)))

export const useDeleteTariff = () =>
  useEnergyMutation((id: number) => fetchOk(`/api/v1/energy/tariffs/${id}`, { method: 'DELETE' }))

export const useSaveMeter = () =>
  useEnergyMutation(({ siteId, id, ...input }: MeterInput & { siteId: number; id?: number }) =>
    id != null
      ? fetchOk(`/api/v1/energy/meters/${id}`, json('PUT', input))
      : fetchOk(`/api/v1/energy/sites/${siteId}/meters`, json('POST', input)))

export const useDeleteMeter = () =>
  useEnergyMutation((m: Pick<Meter, 'id'>) => fetchOk(`/api/v1/energy/meters/${m.id}`, { method: 'DELETE' }))

export const useReconcile = () =>
  useEnergyMutation(({ siteId, month, figures }: { siteId: number; month: string; figures: PlnFigures }) =>
    fetchOk(`/api/v1/energy/sites/${siteId}/reconcile/${month}`, json('PUT', figures)))

export const useDeleteReconcile = () =>
  useEnergyMutation(({ siteId, month }: { siteId: number; month: string }) =>
    fetchOk(`/api/v1/energy/sites/${siteId}/reconcile/${month}`, { method: 'DELETE' }))

export const useRecompute = () =>
  useEnergyMutation(({ siteId, month }: { siteId: number; month: string }) =>
    fetchOk(`/api/v1/energy/sites/${siteId}/recompute?month=${encodeURIComponent(month)}`, { method: 'POST' }))
