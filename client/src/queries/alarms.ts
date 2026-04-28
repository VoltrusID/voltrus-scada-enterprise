import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import type { Alarm } from '../types'
import { queryKeys } from './keys'
import { fetchJson, fetchOk } from './fetch'

export function useActiveAlarmCount(enabled: boolean) {
  return useQuery({
    queryKey: ['alarms', 'active', 'count'],
    queryFn: async () => {
      const r = await fetchJson<{ count: number }>('/api/alarms/active/count')
      return r
    },
    enabled,
    refetchInterval: 5_000,
    staleTime: 4_000,
  })
}

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

export function useAlarmCooldown() {
  return useQuery({
    queryKey: ['alarmCooldown'],
    queryFn: async () => {
      const r = await fetchJson<{ secs: number }>('/api/settings/alarm-cooldown')
      return r.secs
    },
    staleTime: 60_000,
  })
}

export function useUpdateAlarmCooldown() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (secs: number) => {
      await fetchOk('/api/settings/alarm-cooldown', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ secs }),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['alarmCooldown'] })
    },
  })
}
