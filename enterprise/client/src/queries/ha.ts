import { useQuery } from "@tanstack/react-query";
import { fetchJson } from "@voltrus/core/queries/fetch";

export type HaRole = "primary" | "standby" | "fenced";

export interface HaStatus {
  enabled: boolean;
  role: HaRole;
  /** This node polls devices, writes to them and stores history. */
  active: boolean;
  node_id: string;
  epoch: number;
  peer_id: string | null;
  peer_role: HaRole | null;
  peer_epoch: number | null;
  peer_status: "connected" | "degraded" | "disconnected";
  sync_state: "waiting" | "unsynced" | "streaming" | "full_sync" | "error";
  sync_progress: { tables_done: number; tables_total: number; rows: number } | null;
  sync_lag_ms: number | null;
  pending_changes: number | null;
  last_sync_seq: number | null;
  last_sync_at: string | null;
  sync_error: string | null;
  failover_ts: string | null;
  failover_reason: string | null;
  uptime_as_role_secs: number;
  failover_after_ms: number;
  can_take_over: boolean;
  promotion_blocked: string | null;
  witness: { kind: string; reachable: boolean | null; detail: string | null } | null;
  link_auth_failures: number;
}

export function useHaStatus() {
  return useQuery<HaStatus>({
    queryKey: ["ha-status"],
    queryFn: () => fetchJson<HaStatus>("/api/v1/ha/status"),
    refetchInterval: 2000,
  });
}
