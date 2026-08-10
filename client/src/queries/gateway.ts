// Gateway proxy hooks — drive the standalone voltrus-gateway (Discover / Browse /
// Info) via the voltrus HTTP proxy at /api/v1/gateway/*. These do NOT manage
// devices; browsed tags are imported into the normal per-protocol endpoints.
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { queryKeys } from './keys'
import { fetchJson, fetchOk } from './fetch'

export interface GatewayInfo {
  endpoint: string
  reachable: boolean
  version?: string
  build_profile?: string
  uptime_ms?: number | string
  error?: string
}

export interface GatewayEndpoint {
  endpoint: string
  source: 'db' | 'env' | 'default' | string
}

export interface DiscoveredDevice {
  protocol: string
  address: string
  identity: string
}

export interface BrowsedTag {
  protocol: string
  name: string
  data_type?: string
  address?: number
  scale?: number
  register_area?: string
  node_id?: string
  element_count?: number
  [key: string]: unknown
}

export function useGatewayInfo() {
  return useQuery({
    queryKey: queryKeys.gatewayInfo,
    queryFn: () => fetchJson<GatewayInfo>('/api/v1/gateway/info'),
    staleTime: 0,
  })
}

export function useGatewayEndpoint() {
  return useQuery({
    queryKey: queryKeys.gatewayEndpoint,
    queryFn: () => fetchJson<GatewayEndpoint>('/api/v1/gateway/endpoint'),
  })
}

export function useSetGatewayEndpoint() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (endpoint: string) => {
      await fetchOk('/api/v1/gateway/endpoint', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ endpoint }),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.gatewayEndpoint })
      qc.invalidateQueries({ queryKey: queryKeys.gatewayInfo })
    },
  })
}

export interface DiscoverArgs {
  protocol: 'modbus' | 'enip'
  ip: string
  port?: number
  unit_id?: number
  slot?: number
  timeout_ms?: number
}

export function useDiscoverGateway() {
  return useMutation({
    mutationFn: async (args: DiscoverArgs) => {
      const r = await fetchOk('/api/v1/gateway/discover', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(args),
      })
      return (await r.json()) as { devices: DiscoveredDevice[] }
    },
  })
}

export interface BrowseArgs {
  protocol: 'modbus' | 'opcua'
  params: Record<string, unknown>
}

export function useBrowseGateway() {
  return useMutation({
    mutationFn: async (args: BrowseArgs) => {
      const r = await fetchOk('/api/v1/gateway/browse', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(args),
      })
      return (await r.json()) as { tags: BrowsedTag[] }
    },
  })
}
export interface ScanArgs {
  ip: string
  port?: number
  unit_id?: number
  area?: string
  reg_type?: string
  start?: number
  count?: number
}

export function useScanGateway() {
  return useMutation({
    mutationFn: async (args: ScanArgs) => {
      const r = await fetchOk('/api/v1/gateway/scan', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(args),
      })
      return (await r.json()) as { tags: BrowsedTag[] }
    },
  })
}
