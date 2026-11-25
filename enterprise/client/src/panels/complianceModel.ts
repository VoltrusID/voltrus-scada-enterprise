// Part 11 settings as the Compliance › Policy tab edits them. React-free so it
// can be tested under node.

export type SignedActionName = 'tag_write' | 'alarm_ack' | 'alarm_shelve' | 'recipe_load' | 'mode_switch' | 'design_publish'
export type TagScope = 'all' | 'requires_reason' | 'listed'
export type UnattendedOrigin = 'api_key' | 'mcp' | 'flow'

export interface ActionRule { enabled: boolean; meaning: string }

export interface UnattendedException {
  origin: UnattendedOrigin
  /** `api_key:<id>`, `flow:<id>`, a user name (MCP), or `*`. */
  principal: string
  note: string
}

export interface SignaturePolicy {
  actions: Record<SignedActionName, ActionRule>
  tag_scope: TagScope
  tag_ids: number[]
  unattended_exceptions: UnattendedException[]
}

export interface ComplianceConfig {
  auto_logout_minutes: number
  password_min_length: number
  password_require_uppercase: boolean
  password_require_number: boolean
  password_require_special: boolean
  /** 0: passwords never expire. */
  password_rotation_days: number
  authority_to_change: boolean
  /** Refuse the last N passwords (0: no history). */
  password_history: number
  /** Sign out this long after sign-in, active or not (0: no limit). */
  session_max_hours: number
  signatures: SignaturePolicy
  /** Saved at least once by an administrator: the server enforces it. Read-only. */
  enforced?: boolean
}

export const SIGNED_ACTIONS: { key: SignedActionName; label: string; hint: string }[] = [
  { key: 'tag_write', label: 'Tag writes (setpoints)', hint: 'Writing a value to a device from a screen, the API or a recipe' },
  { key: 'alarm_ack', label: 'Acknowledge alarms', hint: 'One alarm or all of them' },
  { key: 'alarm_shelve', label: 'Shelve and unshelve alarms', hint: '' },
  { key: 'recipe_load', label: 'Load recipes', hint: 'One signature covers every write of the load' },
  { key: 'mode_switch', label: 'Mode switches', hint: 'Simulation or live, go-live and rollback, start, stop and run a flow, change a write policy' },
  { key: 'design_publish', label: 'Publish screen designs', hint: 'Publishing a design, or requesting and approving a publish' },
]

export const MEANING_SUGGESTIONS = ['Performed', 'Reviewed', 'Approved', 'Verified', 'Authorized'] as const

export const DEFAULT_CONFIG: ComplianceConfig = {
  auto_logout_minutes: 15,
  password_min_length: 8,
  password_require_uppercase: true,
  password_require_number: true,
  password_require_special: true,
  password_rotation_days: 90,
  authority_to_change: false,
  password_history: 5,
  session_max_hours: 12,
  signatures: {
    actions: {
      tag_write: { enabled: false, meaning: 'Performed' },
      alarm_ack: { enabled: false, meaning: 'Reviewed' },
      alarm_shelve: { enabled: false, meaning: 'Reviewed' },
      recipe_load: { enabled: false, meaning: 'Approved' },
      mode_switch: { enabled: false, meaning: 'Approved' },
      design_publish: { enabled: false, meaning: 'Approved' },
    },
    tag_scope: 'all',
    tag_ids: [],
    unattended_exceptions: [],
  },
}

/** The server's config over the defaults, so a reply from an older build still fills the form. */
export function withDefaults(c: Partial<ComplianceConfig> | undefined): ComplianceConfig {
  const sig = c?.signatures
  return {
    ...DEFAULT_CONFIG,
    ...c,
    signatures: {
      ...DEFAULT_CONFIG.signatures,
      ...sig,
      actions: { ...DEFAULT_CONFIG.signatures.actions, ...sig?.actions },
      tag_ids: sig?.tag_ids ?? [],
      unattended_exceptions: sig?.unattended_exceptions ?? [],
    },
  }
}

/** "12, 15 40" -> [12, 15, 40]; null when something is not a tag id. */
export function parseTagIds(text: string): number[] | null {
  const parts = text.split(/[\s,;]+/).filter(Boolean)
  const ids = parts.map(p => (/^\d+$/.test(p) ? Number(p) : NaN))
  if (ids.some(n => !Number.isSafeInteger(n) || n <= 0)) return null
  return [...new Set(ids)].sort((a, b) => a - b)
}

const PREFIX: Partial<Record<UnattendedOrigin, string>> = { api_key: 'api_key:', flow: 'flow:' }

/** Why an exception would be refused, or null. Mirrors the server's check. */
export function exceptionProblem(e: UnattendedException): string | null {
  const p = e.principal.trim()
  if (!p || /\s/.test(p)) return 'Name one caller, or * for all of this kind.'
  const prefix = PREFIX[e.origin]
  if (prefix && p !== '*' && !p.startsWith(prefix)) return `A ${e.origin === 'flow' ? 'flow' : 'API key'} is written ${prefix}<id>, for example ${prefix}3.`
  return null
}

/** Why the form cannot be saved, or null. */
export function configProblem(c: ComplianceConfig): string | null {
  if (c.password_min_length < 8) return 'The minimum password length is at least 8.'
  if (c.auto_logout_minutes < 1) return 'Auto-logout is at least 1 minute.'
  for (const a of SIGNED_ACTIONS) {
    if (!c.signatures.actions[a.key].meaning.trim()) return `${a.label}: the meaning cannot be empty.`
  }
  if (c.signatures.actions.tag_write.enabled && c.signatures.tag_scope === 'listed' && c.signatures.tag_ids.length === 0) {
    return 'Choose at least one tag, or sign every tag write.'
  }
  for (const e of c.signatures.unattended_exceptions) {
    const p = exceptionProblem(e)
    if (p) return p
  }
  return null
}

/** What a person sees, in one line per enabled action, for the summary under the form. */
export function signatureSummary(c: ComplianceConfig): string[] {
  return SIGNED_ACTIONS.filter(a => c.signatures.actions[a.key].enabled).map(a => {
    const scope = a.key === 'tag_write'
      ? c.signatures.tag_scope === 'requires_reason' ? ' (tags whose write policy requires a reason)'
        : c.signatures.tag_scope === 'listed' ? ` (tags ${c.signatures.tag_ids.join(', ')})` : ''
      : ''
    return `${a.label}${scope}: password, reason, signed as "${c.signatures.actions[a.key].meaning}"`
  })
}
