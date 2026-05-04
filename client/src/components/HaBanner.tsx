import React, { useState, useEffect, useRef } from "react";
import { useHaStatus, type HaStatus } from "../queries/ha";

export const HaBanner: React.FC = () => {
  const { data: status } = useHaStatus();
  const [dismissed, setDismissed] = useState(false);
  const prevRole = useRef(status?.role);

  useEffect(() => {
    if (status && status.role !== prevRole.current) {
      setDismissed(false);
      prevRole.current = status.role;
    }
  }, [status?.role]);

  if (!status || !status.enabled) return null;

  const isFailover = status.role === "standby" || status.failover_ts !== null;

  if (isFailover && !dismissed) {
    return (
      <div
        style={{
          background: "#dc2626",
          color: "white",
          padding: "6px 16px",
          display: "flex",
          alignItems: "center",
          gap: "12px",
          fontSize: "13px",
          fontWeight: 500,
          cursor: "pointer",
        }}
        title={`Node: ${status.node_id} | Since: ${status.failover_ts}`}
      >
        <span>&#9650;</span>
        <span>
          STANDBY TOOK OVER — {status.node_id} | since {status.failover_ts || "unknown"}
        </span>
        <button
          onClick={(e) => { e.stopPropagation(); setDismissed(true); }}
          style={{
            background: "transparent",
            border: "1px solid rgba(255,255,255,0.5)",
            color: "white",
            padding: "2px 8px",
            borderRadius: "4px",
            cursor: "pointer",
            fontSize: "12px",
          }}
        >
          Dismiss
        </button>
      </div>
    );
  }

  const isPrimary = status.role === "primary";
  const peerOk = status.peer_status === "connected";

  let bg = "#6b7280";
  let indicator = "\u25CB";

  if (isPrimary && status.peer_status === "disconnected" && status.peer_id === null) {
    bg = "#6b7280";
    indicator = "\u25CB";
  } else if (isPrimary && peerOk) {
    bg = "#16a34a";
    indicator = "\u25CF";
  } else if (isPrimary && !peerOk) {
    bg = "#d97706";
    indicator = "\u25CF";
  }

  const lagText = status.sync_lag_ms !== null
    ? status.sync_lag_ms < 1000
      ? `synced ${status.sync_lag_ms}ms`
      : `lag ${(status.sync_lag_ms / 1000).toFixed(1)}s`
    : "";

  return (
    <div
      style={{
        background: bg,
        color: "white",
        padding: "4px 16px",
        display: "flex",
        alignItems: "center",
        gap: "8px",
        fontSize: "12px",
      }}
    >
      <span>{indicator}</span>
      <span>PRIMARY — {status.node_id}</span>
      {status.peer_id && (
        <span>| peer: {status.peer_id} ({lagText || status.peer_status})</span>
      )}
      {!status.peer_id && <span>| no peer configured</span>}
    </div>
  );
};
