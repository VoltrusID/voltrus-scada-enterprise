import React, { useState } from "react";
import { useHaStatus } from "../queries/ha";
import { describeHa, type HaTone } from "./haBannerModel";

const TONE: Record<HaTone, string> = {
  ok: "#16a34a",
  info: "#475569",
  warn: "#d97706",
  alert: "#dc2626",
};

export const HaBanner: React.FC = () => {
  const { data: status } = useHaStatus();
  const [dismissed, setDismissed] = useState<string | null>(null);

  const model = status ? describeHa(status) : null;
  if (!model) return null;
  if (model.dismissKey !== null && model.dismissKey === dismissed) return null;

  return (
    <div
      style={{
        background: TONE[model.tone],
        color: "white",
        padding: "5px 16px",
        display: "flex",
        alignItems: "center",
        gap: "12px",
        fontSize: "12px",
        fontWeight: model.tone === "alert" ? 600 : 500,
      }}
      title={status ? `Node ${status.node_id}, epoch ${status.epoch}, ${status.uptime_as_role_secs} s in this role` : undefined}
    >
      <span>{model.text}</span>
      {model.dismissKey !== null && (
        <button
          onClick={() => setDismissed(model.dismissKey)}
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
      )}
    </div>
  );
};
