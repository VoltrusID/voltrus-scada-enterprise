import { useRef, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { fetchOk } from '@voltrus/core/queries/fetch'
import { licenseQueryKey } from '@voltrus/core/queries/license'
import { useLicense } from '@voltrus/core/LicenseContext'
import { normalizeLicense, type LicenseInfo } from '@voltrus/core/edition/license'
import { btnStyle, inputStyle } from '@voltrus/core/styles'

const CONSOLE_URL = 'https://console.voltrus.id'
const ACTIVATE_URL = 'https://activate.voltrus.id'

const TIER_LABEL: Record<LicenseInfo['tier'], string> = {
  community: 'Community',
  integrator: 'Integrator',
  enterprise: 'Enterprise',
}

const CARD: React.CSSProperties = {
  background: 'var(--card)',
  border: '1px solid var(--border)',
  borderRadius: 10,
  padding: 16,
}

const HEADING: React.CSSProperties = { margin: '0 0 4px', fontSize: 14, fontWeight: 600, color: 'var(--fg)' }
const HINT: React.CSSProperties = { margin: '0 0 12px', fontSize: 12, color: 'var(--muted)', lineHeight: 1.5 }

function maskKey(key: string): string {
  return key.length <= 8 ? key : `${key.slice(0, 4)}...${key.slice(-4)}`
}

function describeExpiry(license: LicenseInfo): string {
  if (license.tier === 'community') return 'Not applicable'
  if (!license.expires_at) return 'Lifetime'
  const date = new Date(license.expires_at).toLocaleDateString()
  if (license.grace) return `${date} (expired, grace period running: renew to keep enterprise features)`
  if (typeof license.days_remaining === 'number') return `${date} (${license.days_remaining} days left)`
  return date
}

function base64ToBlob(b64: string): Blob {
  const bin = atob(b64)
  const bytes = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
  return new Blob([bytes], { type: 'application/octet-stream' })
}

async function fileToBase64(file: File): Promise<string> {
  let bin = ''
  for (const byte of new Uint8Array(await file.arrayBuffer())) bin += String.fromCharCode(byte)
  return btoa(bin)
}

function postJson(path: string, body?: unknown): Promise<Response> {
  return fetchOk(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  })
}

/** License status and activation: online key, offline request/activation files, and renewal of monthly keys. */
export function LicensePanel() {
  const license = useLicense()
  const qc = useQueryClient()
  const [keyInput, setKeyInput] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [notice, setNotice] = useState('')
  const fileRef = useRef<HTMLInputElement>(null)

  const run = async (action: () => Promise<string | void>) => {
    setBusy(true)
    setError('')
    setNotice('')
    try {
      const message = await action()
      if (message) setNotice(message)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Request failed')
    } finally {
      setBusy(false)
    }
  }

  /** Activate/refresh endpoints answer `{ ok, status }`; adopt the new status everywhere. */
  const applyStatus = async (r: Response, message: string) => {
    const body = await r.json()
    qc.setQueryData(licenseQueryKey, normalizeLicense(body.status))
    return message
  }

  const activateOnline = () =>
    run(async () => applyStatus(await postJson('/api/license/activate', { license_key: keyInput.trim() }), 'License activated.'))

  const refresh = () =>
    run(async () => applyStatus(await postJson('/api/license/refresh'), 'License refreshed.'))

  const downloadRequest = () =>
    run(async () => {
      const r = await postJson('/api/license/request', { license_key: keyInput.trim() })
      const { request_base64 } = await r.json()
      const url = URL.createObjectURL(base64ToBlob(request_base64))
      const a = document.createElement('a')
      a.href = url
      a.download = 'license-request.bin'
      a.click()
      URL.revokeObjectURL(url)
      return `Saved license-request.bin. Upload it at ${ACTIVATE_URL}, then import the activation file you get back.`
    })

  const importActivation = (file: File) =>
    run(async () => applyStatus(await postJson('/api/license/activate', { activation_base64: await fileToBase64(file) }), 'Activation file imported.'))

  const hasKey = keyInput.trim().length > 0
  const isMonthly = license.tier !== 'community' && !!license.expires_at

  const rows: [string, string][] = [
    ['Edition', TIER_LABEL[license.tier]],
    ['Status', license.status],
    ['Key', license.license_key ? maskKey(license.license_key) : 'None'],
    ['Expires', describeExpiry(license)],
  ]

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16, maxWidth: 640 }}>
      <div style={CARD}>
        <h2 style={{ ...HEADING, fontSize: 16, marginBottom: 12 }}>License</h2>
        <dl style={{ margin: 0, display: 'grid', gridTemplateColumns: '110px 1fr', rowGap: 8, fontSize: 13 }}>
          {rows.map(([label, value]) => (
            <div key={label} style={{ display: 'contents' }}>
              <dt style={{ color: 'var(--muted)' }}>{label}</dt>
              <dd style={{ margin: 0, color: 'var(--fg)' }}>{value}</dd>
            </div>
          ))}
        </dl>
        {license.message && <p style={{ ...HINT, marginTop: 12, marginBottom: 0 }}>{license.message}</p>}
        {isMonthly && (
          <button style={{ ...btnStyle, marginTop: 12 }} onClick={refresh} disabled={busy}>
            Refresh from activation server
          </button>
        )}
      </div>

      <div style={CARD}>
        <h3 style={HEADING}>Activate with a license key</h3>
        <p style={HINT}>
          Voltrus Core works without a license. A key unlocks the Integrator or Enterprise features.
          Buy or manage keys at <a href={CONSOLE_URL} target="_blank" rel="noreferrer" style={{ color: 'var(--cyan)' }}>console.voltrus.id</a>.
        </p>
        <div style={{ display: 'flex', gap: 8 }}>
          <input
            value={keyInput}
            onChange={e => setKeyInput(e.target.value)}
            placeholder="VLT-..."
            aria-label="License key"
            style={{ ...inputStyle, flex: 1, fontFamily: 'var(--font-mono)' }}
          />
          <button style={{ ...btnStyle, borderColor: 'var(--ok)', color: 'var(--ok)' }} onClick={activateOnline} disabled={busy || !hasKey}>
            Activate online
          </button>
        </div>
      </div>

      <div style={CARD}>
        <h3 style={HEADING}>Offline activation</h3>
        <p style={HINT}>
          For installs without internet access: enter the key above and save a request file, upload it at{' '}
          <a href={ACTIVATE_URL} target="_blank" rel="noreferrer" style={{ color: 'var(--cyan)' }}>activate.voltrus.id</a> from a connected PC,
          then import the activation file it returns.
        </p>
        <div style={{ display: 'flex', gap: 8 }}>
          <button style={btnStyle} onClick={downloadRequest} disabled={busy || !hasKey}>Save request file</button>
          <button style={btnStyle} onClick={() => fileRef.current?.click()} disabled={busy}>Import activation file</button>
          <input
            ref={fileRef}
            type="file"
            style={{ display: 'none' }}
            onChange={e => {
              const file = e.target.files?.[0]
              e.target.value = ''
              if (file) void importActivation(file)
            }}
          />
        </div>
      </div>

      {error && <div role="alert" style={{ fontSize: 13, color: 'var(--crit)' }}>{error}</div>}
      {notice && <div role="status" style={{ fontSize: 13, color: 'var(--ok)' }}>{notice}</div>}
    </div>
  )
}
