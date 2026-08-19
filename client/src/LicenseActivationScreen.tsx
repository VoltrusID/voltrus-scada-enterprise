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
          {/* Step 1 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>1</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>Generate a license request on your server</div>
              <code style={CODE_BLOCK}>./voltrus --generate-request YOUR_LICENSE_KEY</code>
            </div>
          </div>

          {/* Step 2 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>2</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>
                Copy <code style={INLINE_CODE}>license-request.bin</code> to a PC with internet access
              </div>
            </div>
          </div>

          {/* Step 3 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>3</div>
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
            <div style={STEP_NUM}>4</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>
                Copy <code style={INLINE_CODE}>license-activation.bin</code> back to your server
              </div>
            </div>
          </div>

          {/* Step 5 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>5</div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--fg)', marginBottom: 2 }}>Import the activation file</div>
              <code style={CODE_BLOCK}>./voltrus --import-activation license-activation.bin</code>
            </div>
          </div>

          {/* Step 6 */}
          <div style={{ display: 'flex', gap: 14 }}>
            <div style={STEP_NUM}>6</div>
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
            href="https://voltrus.id/#pricing"
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
            Purchase a License
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
