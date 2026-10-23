import { useQuery } from "@tanstack/react-query";
import { fetchJson } from "@voltrus/core/queries/fetch";

export interface HaStatus {
  enabled: boolean;
  role: "primary" | "standby";
  node_id: string;
  peer_id: string | null;
  peer_status: "connected" | "degraded" | "disconnected";
  sync_lag_ms: number | null;
  last_sync_seq: number | null;
  failover_ts: string | null;
  uptime_as_role_secs: number;
}

export function useHaStatus() {
  return useQuery<HaStatus>({
    queryKey: ["ha-status"],
    queryFn: () => fetchJson<HaStatus>("/api/v1/ha/status"),
    refetchInterval: 5000,
  });
}
