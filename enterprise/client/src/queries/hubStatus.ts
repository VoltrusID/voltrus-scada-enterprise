// Pure parsing of the hub status answer, kept free of React so tests can import it.

export interface HubSiteStatus {
  site_id: number
  name: string
  /** The hub runs a link task for the site. */
  running: boolean
  /** The link is open and authenticated (or data is arriving). */
  connected: boolean
  last_seen: number | null
  last_message_at: number | null
  tags: number
  last_error: string | null
  /** Hub device that holds the site's tags. */
  device_id: string
}

/** `GET /api/v1/hub/status` answers `{ sites: [{ id, ... }], total }`. */
export function parseHubStatus(data: unknown): HubSiteStatus[] {
  const rows: any[] = Array.isArray(data) ? data : ((data as any)?.sites ?? [])
  return rows.map(r => ({
    site_id: r.site_id ?? r.id,
    name: r.name,
    running: !!r.running,
    connected: !!r.connected,
    last_seen: r.last_seen ?? null,
    last_message_at: r.last_message_at ?? null,
    tags: r.tags ?? 0,
    last_error: r.last_error ?? null,
    device_id: r.device_id ?? `${r.name}:default`,
  }))
}
