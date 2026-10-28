import type { HaStatus } from '../queries/ha.ts'

export type HaTone = 'ok' | 'info' | 'warn' | 'alert'

export interface HaBannerModel {
  tone: HaTone
  text: string
  /** Only a takeover notice can be dismissed. */
  dismissKey: string | null
}

/** A takeover stays announced for a day. */
const TAKEOVER_NOTICE_MS = 24 * 60 * 60 * 1000

function lag(ms: number | null): string {
  if (ms === null) return ''
  return ms < 1000 ? `synced, ${ms} ms behind` : `${(ms / 1000).toFixed(1)} s behind`
}

/** What the banner says for a status. Pure, so every state is testable. */
export function describeHa(s: HaStatus, now: number = Date.now()): HaBannerModel | null {
  if (!s.enabled) return null
  const peer = s.peer_id ?? 'peer'

  if (s.role === 'fenced') {
    return {
      tone: 'alert',
      text: `FENCED: ${s.node_id} stopped polling and writing until it can reach its witness or peer${s.witness?.detail ? ` (${s.witness.detail})` : ''}`,
      dismissKey: null,
    }
  }

  if (s.role === 'standby') {
    if (s.sync_state === 'full_sync') {
      const p = s.sync_progress
      return { tone: 'info', text: `STANDBY: ${s.node_id} is taking a full sync from ${peer}${p ? ` (${p.tables_done}/${p.tables_total} tables, ${p.rows} rows)` : ''}`, dismissKey: null }
    }
    if (s.sync_state === 'error') {
      return { tone: 'alert', text: `STANDBY: ${s.node_id} cannot apply the primary's changes: ${s.sync_error ?? 'unknown error'}`, dismissKey: null }
    }
    if (!s.can_take_over) {
      return { tone: 'warn', text: `STANDBY: ${s.node_id} has no complete copy yet and cannot take over`, dismissKey: null }
    }
    if (s.peer_status !== 'connected') {
      const why = s.promotion_blocked ? `, blocked: ${s.promotion_blocked}` : `, takes over after ${(s.failover_after_ms / 1000).toFixed(1)} s of silence`
      return { tone: 'warn', text: `STANDBY: ${s.node_id} lost the primary${why}`, dismissKey: null }
    }
    return { tone: 'info', text: `STANDBY: ${s.node_id} follows ${peer} (epoch ${s.epoch}, ${lag(s.sync_lag_ms)})`, dismissKey: null }
  }

  const tookOver = s.failover_ts !== null && now - Date.parse(s.failover_ts) < TAKEOVER_NOTICE_MS
  if (tookOver) {
    return {
      tone: 'alert',
      text: `TOOK OVER: ${s.node_id} is primary at epoch ${s.epoch} since ${s.failover_ts}${s.failover_reason ? ` (${s.failover_reason})` : ''}`,
      dismissKey: s.failover_ts,
    }
  }
  if (s.peer_id === null && s.peer_status === 'disconnected') {
    return { tone: 'info', text: `PRIMARY: ${s.node_id} (epoch ${s.epoch}), no standby has connected`, dismissKey: null }
  }
  if (s.peer_status === 'connected') {
    return { tone: 'ok', text: `PRIMARY: ${s.node_id} (epoch ${s.epoch}) | standby ${peer}, ${lag(s.sync_lag_ms)}`, dismissKey: null }
  }
  return { tone: 'warn', text: `PRIMARY: ${s.node_id} (epoch ${s.epoch}) | standby ${peer} is not connected, no failover protection`, dismissKey: null }
}
