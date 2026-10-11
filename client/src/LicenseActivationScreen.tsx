import { useState, useEffect, useCallback } from 'react'
import { fetchJson } from './queries'

interface LicenseStatus {
  status: string
  message?: string
  license_key?: string
  expires_at?: string | null
  days_remaining?: number | null
}

const STEP_NUM: React.CSSProperties = {
  width: 28,
  height: 28,
  borderRadius: '50%',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  fontSize: 13,
  fontWeight: 700,
  flexShrink: 0,
  background: 'rgba(34,197,94,.12)',
  color: '#22c55e',
  border: '1px solid rgba(34,197,94,.25)',
}

const CODE_BLOCK: React.CSSProperties = {
  display: 'block',
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 8,
  padding: '12px 14px',
  fontFamily: "'var(--font-mono)', monospace",
  fontSize: 12,
  color: 'var(--fg)',
  marginTop: 8,
  marginBottom: 4,
  wordBreak: 'break-all',
  lineHeight: 1.6,
}

const INLINE_CODE: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '2px 7px',
  fontFamily: "'var(--font-mono)', monospace",
  fontSize: 12,
  color: 'var(--fg)',
}

const LINK: React.CSSProperties = {
  color: '#22c55e',
  textDecoration: 'none',
  fontWeight: 600,
  borderBottom: '1px dashed rgba(34,197,94,.4)',
  transition: 'border-color 0.15s',
}

// Console checkouts (same Polar products as voltrus.id/voltrus-scada pricing). Keys arrive by email after purchase.
const CONSOLE_CHECKOUT = 'https://console.voltrus.id/checkout/p/'
const TIERS = [
  { name: 'Edge', price: '$49', note: 'Up to 15 devices', id: '0f36bf0f-ecfe-4318-a993-50cf1d5235a8' },
  { name: 'Starter', price: '$249', note: 'Up to 50 devices', id: 'fb085b89-6be9-4e08-b202-edb5d7de360a' },
  { name: 'Professional', price: '$499', note: 'Unlimited devices, OPC-UA, RBAC', id: '8b82c7d0-24ef-4246-a5b4-eb2c85935b19' },
  { name: 'Enterprise', price: '$999', note: 'Every feature: SSO, S7, EtherNet/IP', id: '79bdecb0-6a5c-484e-b421-e608ea5d6898' },
]

export function LicenseActivationScreen() {
  const [status, setStatus] = useState<LicenseStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  const checkLicense = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const data = await fetchJson<LicenseStatus>('/api/license/status')
      setStatus(data)
      if (data.status === 'Valid') {
        window.location.reload()
      }
    } catch {
      setError('Failed to check license status. Please try again.')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    checkLicense()
  }, [checkLicense])

  if (loading) {
    return (
      <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', color: 'var(--muted)', fontFamily: 'var(--font-ui)' }}>
        Checking license...
      </div>
    )
  }

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', fontFamily: 'var(--font-ui)', padding: '40px 20px' }}>
      <div style={{
        background: 'var(--card)',
        border: '1px solid var(--border)',
        borderRadius: 16,
        padding: 0,
        maxWidth: 600,
        width: '100%',
        boxShadow: '0 8px 40px rgba(0,0,0,0.25)',
        overflow: 'hidden',
      }}>
        {/* Header */}
        <div style={{
          padding: '32px 36px 24px',
          borderBottom: '1px solid var(--border)',
          background: 'linear-gradient(135deg, rgba(34,197,94,.06) 0%, rgba(34,197,94,.02) 100%)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 10 }}>
            <div style={{
              width: 44, height: 44, borderRadius: 12,
              background: 'rgba(34,197,94,.1)', border: '1px solid rgba(34,197,94,.2)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#22c55e" strokeWidth="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
            </div>
            <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, color: 'var(--fg)' }}>Activate Your License</h1>
          </div>
          <p style={{ margin: 0, fontSize: 14, color: 'var(--muted)', lineHeight: 1.6 }}>
            Your Voltrus installation requires a valid license. Follow the steps below to activate.
          </p>
        </div>

        {/* Steps */}
        <div style={{ padding: '28px 36px 20px', display: 'flex', flexDirection: 'column', gap: 20 }}>
          {/* Step 0: get a key */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>1</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 8 }}>Get a license key</div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: 8 }}>
                {TIERS.map(t => (
                  <a key={t.id} href={CONSOLE_CHECKOUT + t.id} target="_blank" rel="noopener noreferrer" style={{
                    display: 'flex', flexDirection: 'column', gap: 2, padding: '10px 12px', borderRadius: 8, textDecoration: 'none',
                    border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)',
                  }}
                    onMouseEnter={e => { e.currentTarget.style.borderColor = '#22c55e' }}
                    onMouseLeave={e => { e.currentTarget.style.borderColor = 'var(--border)' }}
                  >
                    <span style={{ display: 'flex', justifyContent: 'space-between', fontSize: 13, fontWeight: 700 }}>
                      {t.name}<span style={{ color: '#22c55e' }}>{t.price}</span>
                    </span>
                    <span style={{ fontSize: 11, color: 'var(--muted)' }}>{t.note} · lifetime</span>
                  </a>
                ))}
              </div>
              <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 8, lineHeight: 1.5 }}>
                Server online? Skip the file steps: <code style={INLINE_CODE}>./voltrus --online-activate YOUR_LICENSE_KEY</code>
              </div>
            </div>
          </div>

          {/* Step 1 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>2</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>Generate a license request on your server</div>
              <code style={CODE_BLOCK}>./voltrus --generate-request YOUR_LICENSE_KEY</code>
            </div>
          </div>

          {/* Step 2 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>3</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>
                Copy <code style={INLINE_CODE}>license-request.bin</code> to a PC with internet access
              </div>
            </div>
          </div>

          {/* Step 3 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>4</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>
                Upload and activate at{' '}
                <a href="https://voltrus.id/activate" target="_blank" rel="noopener noreferrer" style={LINK}>
                  voltrus.id/activate
                </a>
              </div>
            </div>
          </div>

          {/* Step 4 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>5</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>
                Copy <code style={INLINE_CODE}>license-activation.bin</code> back to your server
              </div>
            </div>
          </div>

          {/* Step 5 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>6</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>Import the activation file</div>
              <code style={CODE_BLOCK}>./voltrus --import-activation license-activation.bin</code>
            </div>
          </div>

          {/* Step 6 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>7</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)' }}>
                Refresh this page or click <strong>Check Again</strong> below
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div style={{
          padding: '20px 36px 28px',
          borderTop: '1px solid var(--border)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 12,
        }}>
          <a
            href="https://console.voltrus.id/checkout/p/8b82c7d0-24ef-4246-a5b4-eb2c85935b19"
            target="_blank"
            rel="noopener noreferrer"
            style={{
              fontSize: 13,
              fontWeight: 600,
              color: 'var(--muted)',
              textDecoration: 'none',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              transition: 'color 0.15s',
            }}
            onMouseEnter={e => { e.currentTarget.style.color = '#22c55e' }}
            onMouseLeave={e => { e.currentTarget.style.color = 'var(--muted)' }}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="10"/><path d="M16 8h-6a2 2 0 1 0 0 4h4a2 2 0 1 1 0 4H8"/><path d="M12 18V6"/></svg>
            Buy Professional
          </a>
          <button
            onClick={checkLicense}
            style={{
              background: '#22c55e',
              color: '#fff',
              border: 'none',
              borderRadius: 8,
              padding: '10px 24px',
              fontSize: 14,
              fontWeight: 600,
              cursor: 'pointer',
              fontFamily: 'var(--font-ui)',
              transition: 'opacity 0.15s',
            }}
            onMouseEnter={e => { e.currentTarget.style.opacity = '0.85' }}
            onMouseLeave={e => { e.currentTarget.style.opacity = '1' }}
          >
            Check Again
          </button>
        </div>

        {error && (
          <div style={{ color: 'var(--crit)', fontSize: 13, padding: '0 36px 20px' }}>{error}</div>
        )}
        {status?.message && status.status !== 'Valid' && (
          <div style={{ color: 'var(--crit)', fontSize: 13, padding: '0 36px 20px' }}>{status.message}</div>
        )}
      </div>
    </div>
  )
}
