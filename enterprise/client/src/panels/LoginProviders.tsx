import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchJson } from '@voltrus/core/queries/fetch'
import type { LoginProvidersProps } from '@voltrus/core/edition/types'
import { providerLoginUrl } from './loginUrl'

interface AuthProvider {
  provider: string
  enabled: boolean
  label: string
}

const BUTTON: React.CSSProperties = {
  padding: '11px 20px',
  borderRadius: 8,
  border: '1px solid var(--border)',
  background: 'var(--bg)',
  color: 'var(--fg)',
  fontSize: 14,
  fontWeight: 600,
  cursor: 'pointer',
  fontFamily: 'Inter, sans-serif',
  width: '100%',
}

const INPUT: React.CSSProperties = {
  width: '100%',
  padding: '10px 12px',
  borderRadius: 8,
  border: '1px solid var(--border)',
  background: 'var(--bg)',
  color: 'var(--fg)',
  fontSize: 14,
  boxSizing: 'border-box',
}

/** Providers other than local, as reported by GET /api/v1/auth/providers. Never throws. */
async function fetchExternalProviders(): Promise<AuthProvider[]> {
  try {
    const body = await fetchJson<{ providers?: AuthProvider[] }>('/api/v1/auth/providers')
    return (body.providers ?? []).filter(p => p.provider !== 'local' && p.enabled)
  } catch {
    return []
  }
}

function LdapForm({ label, onLoggedIn }: { label: string } & LoginProvidersProps) {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setBusy(true)
    try {
      // Raw fetch: the refresh wrapper would turn this endpoint's own 401 into "Session expired".
      const r = await fetch('/api/v1/auth/ldap/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
        credentials: 'include',
      })
      const body = await r.json().catch(() => ({}))
      if (!r.ok) throw new Error(body.error || `Login failed: ${r.status}`)
      onLoggedIn({ username: body.username ?? username, role: body.role, must_change_password: body.must_change_password })
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Login failed')
    } finally {
      setBusy(false)
    }
  }

  return (
    <form onSubmit={submit} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.04em' }}>{label}</div>
      <input value={username} onChange={e => setUsername(e.target.value)} placeholder="Directory username" autoComplete="username" aria-label={`${label} username`} style={INPUT} />
      <input type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="Directory password" autoComplete="current-password" aria-label={`${label} password`} style={INPUT} />
      {error && <div role="alert" style={{ fontSize: 13, color: 'var(--crit)' }}>{error}</div>}
      <button type="submit" disabled={busy || !username || !password} style={BUTTON}>
        {busy ? 'Signing in...' : `Sign in with ${label}`}
      </button>
    </form>
  )
}

/** Extra sign-in options under the password form: directory login for LDAP, redirect buttons for OIDC and SAML. */
export function LoginProviders({ onLoggedIn }: LoginProvidersProps) {
  const { data: providers = [] } = useQuery({
    queryKey: ['authProviders'],
    queryFn: fetchExternalProviders,
    staleTime: 60_000,
    retry: false,
  })

  if (providers.length === 0) return null

  return (
    <div style={{ padding: '0 36px 28px', display: 'flex', flexDirection: 'column', gap: 12, borderTop: '1px solid var(--border)', paddingTop: 20 }}>
      {providers.map(p =>
        p.provider === 'ldap' ? (
          <LdapForm key={p.provider} label={p.label} onLoggedIn={onLoggedIn} />
        ) : (
          <button
            key={p.provider}
            type="button"
            style={BUTTON}
            onClick={() => { window.location.href = providerLoginUrl(p.provider, window.location.pathname + window.location.search) }}
          >
            Sign in with {p.label}
          </button>
        ),
      )}
    </div>
  )
}
