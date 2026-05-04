import { useState, useEffect, useCallback } from 'react'

interface LicenseStatus {
  status: string
  message?: string
  license_key?: string
  expires_at?: string | null
  days_remaining?: number | null
}

export function LicenseActivationScreen() {
  const [status, setStatus] = useState<LicenseStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  const checkLicense = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const res = await fetch('/api/license/status')
      const data = await res.json()
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
      <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', color: 'var(--muted)', fontFamily: 'Inter, sans-serif' }}>
        Checking license...
      </div>
    )
  }

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)', fontFamily: 'Inter, sans-serif', padding: '40px 20px' }}>
      <div style={{
        background: 'var(--card)',
        border: '1px solid var(--border)',
        borderRadius: 12,
        padding: 40,
        maxWidth: 560,
        width: '100%',
        boxShadow: '0 4px 24px rgba(0,0,0,0.08)',
      }}>
        <h1 style={{ margin: '0 0 8px', fontSize: 22, color: 'var(--fg)' }}>Welcome to Voltrus Lite</h1>
        <p style={{ margin: '0 0 24px', fontSize: 14, color: 'var(--muted)', lineHeight: 1.5 }}>
          Your license is not activated. Follow the steps below to activate your installation.
        </p>

        <ol style={{ margin: '0 0 24px', paddingLeft: 20, color: 'var(--fg)', fontSize: 14, lineHeight: 1.7 }}>
          <li style={{ marginBottom: 8 }}>
            Run this command on your server terminal:
            <code style={{
              display: 'block',
              background: 'var(--bg)',
              border: '1px solid var(--border)',
              borderRadius: 6,
              padding: '10px 12px',
              fontFamily: "'JetBrains Mono', monospace",
              fontSize: 12,
              color: 'var(--fg)',
              margin: '6px 0',
              wordBreak: 'break-all',
            }}>./voltrus-lite --generate-request YOUR_LICENSE_KEY</code>
          </li>
          <li style={{ marginBottom: 8 }}>
            Copy the generated <code style={{ background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 4, padding: '2px 6px', fontFamily: "'JetBrains Mono', monospace", fontSize: 12 }}>license-request.bin</code> to a PC with internet access.
          </li>
          <li style={{ marginBottom: 8 }}>
            Upload it at <a href="https://voltrus.id/activate.html" target="_blank" rel="noopener noreferrer" style={{ color: 'var(--accent)' }}>voltrus.id/activate.html</a>.
          </li>
          <li style={{ marginBottom: 8 }}>
            Copy the returned <code style={{ background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 4, padding: '2px 6px', fontFamily: "'JetBrains Mono', monospace", fontSize: 12 }}>license-activation.bin</code> back to your server.
          </li>
          <li style={{ marginBottom: 8 }}>
            Run this command to activate:
            <code style={{
              display: 'block',
              background: 'var(--bg)',
              border: '1px solid var(--border)',
              borderRadius: 6,
              padding: '10px 12px',
              fontFamily: "'JetBrains Mono', monospace",
              fontSize: 12,
              color: 'var(--fg)',
              margin: '6px 0',
              wordBreak: 'break-all',
            }}>./voltrus-lite --import-activation license-activation.bin</code>
          </li>
          <li>Refresh this page or click "Check Again" below.</li>
        </ol>

        <button
          onClick={checkLicense}
          style={{
            background: 'var(--accent)',
            color: '#fff',
            border: 'none',
            borderRadius: 8,
            padding: '10px 20px',
            fontSize: 14,
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          Check Again
        </button>

        {error && (
          <div style={{ color: 'var(--crit)', fontSize: 13, marginTop: 12 }}>{error}</div>
        )}
        {status?.message && status.status !== 'Valid' && (
          <div style={{ color: 'var(--crit)', fontSize: 13, marginTop: 12 }}>{status.message}</div>
        )}
      </div>
    </div>
  )
}
