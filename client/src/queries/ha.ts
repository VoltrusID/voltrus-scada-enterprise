import { useQuery } from "@tanstack/react-query";

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
    queryFn: async () => {
      const res = await fetch("/api/v1/ha/status");
      if (!res.ok) throw new Error("Failed to fetch HA status");
      return res.json();
    },
    refetchInterval: 5000,
  });
}
