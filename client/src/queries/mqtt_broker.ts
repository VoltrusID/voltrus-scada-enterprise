import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { queryKeys } from './keys'
import { fetchJson, fetchOk } from './fetch'

export interface MqttBrokerStatus {
  status: string
  connected_clients: number
  clients: MqttClient[]
  config: {
    port: number
    max_connections: number
    auth_enabled: boolean
  } | null
}

export interface MqttClient {
  client_id: string
  connected_at: number
  subscriptions: string[]
}

export interface MqttConfigUpdate {
  port?: number
  max_connections?: number
  auth?: {
    enabled?: boolean
    users?: { username: string; password: string }[]
  }
}

export function useMqttBrokerStatus(enabled = true) {
  return useQuery({
    queryKey: queryKeys.mqttBroker,
    queryFn: () => fetchJson<MqttBrokerStatus>('/api/v1/mqtt-broker/status'),
    enabled,
    refetchInterval: 5000,
  })
}

export function useUpdateMqttConfig() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (config: MqttConfigUpdate) => {
      const r = await fetchOk('/api/v1/mqtt-broker/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.mqttBroker })
    },
  })
}

export function useStartMqttBroker() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const r = await fetchOk('/api/v1/mqtt-broker/start', { method: 'POST' })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.mqttBroker })
    },
  })
}

export function useStopMqttBroker() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const r = await fetchOk('/api/v1/mqtt-broker/stop', { method: 'POST' })
      return r.json()
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.mqttBroker })
    },
  })
}
