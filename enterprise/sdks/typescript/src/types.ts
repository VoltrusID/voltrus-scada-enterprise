// Request and response shapes of the Enterprise endpoints, taken from the
// handlers in enterprise/src. Timestamps are unix seconds unless noted.

// ── Licence ────────────────────────────────────────────────────

export type LicenseTier = 'community' | 'integrator' | 'enterprise';

export interface FeatureFlags {
  white_label: boolean;
  theme_full: boolean;
  pdf_reporting: boolean;
  sms_notifications: boolean;
  multi_site: boolean;
  sso: boolean;
  sso_saml: boolean;
  cfr_part_11: boolean;
  iec_62443: boolean;
  ha: boolean;
  rtsp_video: boolean;
}

export type LicenseStatusName =
  | 'Community'
  | 'Valid'
  | 'Unactivated'
  | 'Expired'
  | 'InvalidFingerprint'
  | 'InvalidSignature'
  | 'Corrupted';

/**
 * Body of `GET /api/license/status` and of the `status` field returned by
 * activate and refresh. Which extra fields are present depends on `status`.
 */
export interface LicenseStatus {
  status: LicenseStatusName;
  tier: LicenseTier;
  feature_flags: FeatureFlags;
  /** Valid only. */
  license_key?: string;
  /** Valid and Expired. Null or absent for lifetime keys (RFC 3339). */
  expires_at?: string | null;
  /** Valid only. Null for lifetime keys. */
  days_remaining?: number | null;
  /** Valid only. True while a lapsed monthly key is inside its 7 day grace period. */
  grace?: boolean;
  /** Valid only. True for a retired entitlement that keeps logo-only branding. */
  legacy_white_label?: boolean;
  /** Every status except Community and Valid. */
  message?: string;
  /** InvalidFingerprint only. */
  score?: number;
  /** InvalidFingerprint only. */
  required?: number;
}

/**
 * Activate online with `license_key`, or offline with `activation_base64`
 * (the signed activation file, base64 encoded). Send exactly one.
 */
export type ActivateLicenseRequest =
  | { license_key: string; activation_base64?: undefined }
  | { activation_base64: string; license_key?: undefined };

export interface LicenseActionResponse {
  ok: boolean;
  status: LicenseStatus;
}

export interface LicenseRequestFile {
  /** Upload this at activate.voltrus.id to get an activation file. */
  request_base64: string;
}

// ── Compliance (21 CFR Part 11) ────────────────────────────────

export interface ComplianceConfig {
  auto_logout_minutes: number;
  password_min_length: number;
  password_require_uppercase: boolean;
  password_require_number: boolean;
  password_require_special: boolean;
  password_rotation_days: number;
  authority_to_change: boolean;
}

export type UpdateComplianceConfigRequest = Partial<ComplianceConfig>;

export interface CreateSignatureRequest {
  resource_type: string;
  resource_id: string;
  /** Required, must not be blank. */
  reason: string;
  /** Required, must not be blank. */
  meaning: string;
}

export interface CreateSignatureResponse {
  ok: boolean;
  id: number;
  signature_hash: string;
  created_at: number;
}

export interface ElectronicSignature {
  id: number;
  user_id: string;
  action: string;
  resource_type: string;
  resource_id: string;
  reason: string;
  meaning: string;
  signature_hash: string;
  created_at: number;
}

export interface ListSignaturesQuery {
  resource_type?: string;
  resource_id?: string;
}

// ── Security audit (IEC 62443) ─────────────────────────────────

export interface SecurityEvent {
  id: number;
  event_type: string;
  severity: string;
  source_ip?: string | null;
  user_agent?: string | null;
  username?: string | null;
  details?: string | null;
  created_at: number;
}

export interface ListSecurityEventsQuery {
  event_type?: string;
  severity?: string;
  /** From, unix seconds. */
  from?: number;
  /** To, unix seconds. */
  to?: number;
  /** Default 100, capped at 1000 by the server. */
  limit?: number;
  offset?: number;
}

export interface SecurityEventsResponse {
  events: SecurityEvent[];
  count: number;
}

export interface ExportSecurityEventsQuery {
  from?: number;
  to?: number;
}

export interface SecurityThresholds {
  failed_login_threshold: number;
  failed_login_window_secs: number;
  suspicious_401_threshold: number;
  suspicious_401_window_secs: number;
  rate_limit_threshold: number;
  rate_limit_window_secs: number;
}

// ── Multi-site hub ─────────────────────────────────────────────

export interface HubSite {
  id: number;
  name: string;
  url: string;
  enabled: boolean;
  last_seen?: number | null;
  created_at: number;
}

export interface CreateHubSiteRequest {
  name: string;
  /** Must start with http:// or https://. */
  url: string;
  /** API key of the remote site. Never returned by the server. */
  api_key: string;
  enabled?: boolean;
}

export interface UpdateHubSiteRequest {
  name?: string;
  url?: string;
  api_key?: string;
  enabled?: boolean;
}

export interface HubSiteStatus {
  id: number;
  name: string;
  url: string;
  enabled: boolean;
  running: boolean;
  last_seen?: number | null;
}

export interface HubStatus {
  sites: HubSiteStatus[];
  total: number;
}

// ── Cameras (RTSP video) ───────────────────────────────────────

export interface Camera {
  id: number;
  name: string;
  url: string;
  enabled: boolean;
  snapshot_on_alarm: boolean;
  retention_hours: number;
  created_at: number;
  has_snapshots: boolean;
}

export interface CreateCameraRequest {
  name: string;
  /** Must start with rtsp://. */
  url: string;
  /** Default true. */
  enabled?: boolean;
  /** Default true. */
  snapshot_on_alarm?: boolean;
  /** Default set by the server. */
  retention_hours?: number;
}

export interface UpdateCameraRequest {
  name?: string;
  url?: string;
  enabled?: boolean;
  snapshot_on_alarm?: boolean;
  retention_hours?: number;
}

export interface CreateCameraResponse {
  ok: boolean;
  id: number;
}

export interface CameraSnapshot {
  id: number;
  camera_id: number;
  timestamp: number;
  alarm_source?: string | null;
  file_path: string;
  file_size: number;
}

export interface TriggerSnapshotResponse {
  ok: boolean;
  snapshot_id: number;
  camera_id: number;
  timestamp: number;
  file_size: number;
}

export interface StreamInfo {
  camera_id: number;
  name: string;
  stream_url: string;
  enabled: boolean;
  protocol: string;
}

// ── Scheduled reports ──────────────────────────────────────────

export type ReportSchedule = 'daily' | 'weekly' | 'monthly';

export interface Report {
  id: number;
  name: string;
  schedule: ReportSchedule;
  enabled: boolean;
  /** The string "all" or a JSON array of device ids as text. */
  device_ids: string;
  /** JSON array of email addresses as text, for example '["ops@example.com"]'. */
  email_recipients: string;
  last_generated?: number | null;
  created_at: number;
}

export interface CreateReportRequest {
  name: string;
  schedule: ReportSchedule;
  /** The string "all" (default) or a JSON array of device ids as text. */
  device_ids?: string;
  /** JSON array of email addresses as text. At least one address is required. */
  email_recipients: string;
  /** Default true. */
  enabled?: boolean;
}

export interface UpdateReportRequest {
  name?: string;
  schedule?: ReportSchedule;
  device_ids?: string;
  email_recipients?: string;
  enabled?: boolean;
}

// ── Branding ───────────────────────────────────────────────────

export interface Branding {
  title: string;
  subtitle: string;
  /** Logo as stored by the server (data URL or URL), empty for the default. */
  logo: string;
  theme_level: 'none' | 'basic' | 'full';
}

export interface UpdateBrandingRequest {
  title?: string;
  subtitle?: string;
  logo?: string;
}

// ── Hot standby ────────────────────────────────────────────────

export type HaRole = 'primary' | 'standby' | 'fenced';

export interface HaFullSyncReport {
  at: string;
  tables: number;
  rows: number;
  duration_ms: number;
}

export interface HaStatus {
  enabled: boolean;
  role: HaRole;
  /** This node polls devices, writes to them and stores history. */
  active: boolean;
  node_id: string;
  /** The term. Grows with every new primary. */
  epoch: number;
  peer_id?: string | null;
  peer_role?: HaRole | null;
  peer_epoch?: number | null;
  /** `connected`: a replication session is up. `degraded`: the peer answers, no session. */
  peer_status: 'connected' | 'degraded' | 'disconnected' | string;
  sync_state: 'waiting' | 'unsynced' | 'streaming' | 'full_sync' | 'error' | string;
  sync_progress?: { tables_done: number; tables_total: number; rows: number } | null;
  /** Age of the oldest change the standby has not acknowledged. */
  sync_lag_ms?: number | null;
  pending_changes?: number | null;
  last_sync_seq?: number | null;
  last_sync_at?: string | null;
  sync_error?: string | null;
  /** When this node last took over the primary role. */
  failover_ts?: string | null;
  failover_reason?: string | null;
  uptime_as_role_secs: number;
  failover_after_ms: number;
  /** Standby: its copy is complete, so it may take over. */
  can_take_over: boolean;
  promotion_blocked?: string | null;
  witness?: { kind: 'arbiter' | 'tcp' | 'http' | string; reachable?: boolean | null; detail?: string | null } | null;
  last_full_sync?: HaFullSyncReport | null;
  link_auth_failures: number;
}

export interface HaSyncResponse {
  status: 'synced' | 'sync_in_progress';
  report?: HaFullSyncReport | null;
  progress?: { tables_done: number; tables_total: number; rows: number } | null;
}

export interface HaFailoverResponse {
  status: 'failover_complete';
  node_id: string;
  /** The role of the node that was asked, after the switch. */
  new_role: HaRole;
  epoch: number;
  duration_ms: number;
}

// ── SSO ────────────────────────────────────────────────────────

export interface SsoProvider {
  provider: 'oidc' | 'saml' | 'ldap' | string;
  enabled: boolean;
  label: string;
  /** OIDC only. Empty when the provider is disabled. */
  issuer?: string;
}
