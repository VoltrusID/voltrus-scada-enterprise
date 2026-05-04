import { useState, useEffect } from 'react'
import {
  useMqttBrokerStatus,
  useStartMqttBroker,
  useStopMqttBroker,
  useUpdateMqttConfig,
} from './queries'
import type { MqttConfigUpdate } from './queries'
import { inputStyle, btnStyle } from './styles'

export function MqttBrokerPanel({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const { data: status, isLoading } = useMqttBrokerStatus()
  const startMut = useStartMqttBroker()
  const stopMut = useStopMqttBroker()
  const updateMut = useUpdateMqttConfig()

  const [showConfig, setShowConfig] = useState(false)
  const [port, setPort] = useState<number>(1883)
  const [maxConnections, setMaxConnections] = useState<number>(100)
  const [authEnabled, setAuthEnabled] = useState(false)
  const [users, setUsers] = useState<{ username: string; password: string }[]>([])

  useEffect(() => {
    if (status?.config) {
      setPort(status.config.port)
      setMaxConnections(status.config.max_connections)
      setAuthEnabled(status.config.auth_enabled)
    }
  }, [status?.config])

  useEffect(() => {
    if (embedded) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose, embedded])

  const isRunning = status?.status === 'running'

  const handleToggle = () => {
    if (isRunning) {
      stopMut.mutate()
    } else {
      startMut.mutate()
    }
  }

  const handleSaveConfig = (e: React.FormEvent) => {
    e.preventDefault()
    const payload: MqttConfigUpdate = {
      port,
      max_connections: maxConnections,
      auth: {
        enabled: authEnabled,
        users: authEnabled ? users.filter(u => u.username.trim() !== '') : undefined,
      },
    }
    updateMut.mutate(payload, { onSuccess: () => setShowConfig(false) })
  }

  const addUser = () => {
    setUsers([...users, { username: '', password: '' }])
  }

  const removeUser = (index: number) => {
    setUsers(users.filter((_, i) => i !== index))
  }

  const updateUser = (index: number, field: 'username' | 'password', value: string) => {
    const updated = [...users]
    updated[index] = { ...updated[index], [field]: value }
    setUsers(updated)
  }

  const cardStyle: React.CSSProperties = {
    background: 'var(--card)',
    border: '1px solid var(--border)',
    borderRadius: 10,
    padding: 24,
  }

  const thStyle: React.CSSProperties = {
    textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)',
    textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
    borderBottom: '1px solid var(--border)',
  }
  const tdStyle: React.CSSProperties = {
    padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)',
  }

  const statusContent = (
    <>
      <div style={{ ...cardStyle, marginBottom: 16 }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div style={{
              width: 12, height: 12, borderRadius: '50%',
              background: isRunning ? 'var(--green)' : 'var(--crit)',
              boxShadow: `0 0 8px ${isRunning ? 'var(--green)' : 'var(--crit)'}`,
            }} />
            <div>
              <div style={{ fontSize: 16, fontWeight: 700 }}>MQTT Broker</div>
              <div style={{ fontSize: 12, color: 'var(--muted)' }}>
                {isRunning ? `Running — ${status?.connected_clients ?? 0} connected clients` : 'Stopped'}
              </div>
            </div>
          </div>
          <button
            onClick={handleToggle}
            disabled={startMut.isPending || stopMut.isPending}
            style={{
              ...btnStyle,
              borderColor: isRunning ? 'var(--crit)' : 'var(--green)',
              color: isRunning ? 'var(--crit)' : 'var(--green)',
            }}
          >
            {startMut.isPending || stopMut.isPending
              ? (isRunning ? 'Stopping...' : 'Starting...')
              : (isRunning ? 'Stop Broker' : 'Start Broker')
            }
          </button>
        </div>

        {isRunning && status?.clients && status.clients.length > 0 && (
          <div>
            <div style={{ fontSize: 12, fontWeight: 600, marginBottom: 8, color: 'var(--muted)', textTransform: 'uppercase' }}>Connected Clients</div>
            <div style={{ overflow: 'auto', background: 'var(--bg)', borderRadius: 8, border: '1px solid var(--border)' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr>
                    <th style={thStyle}>Client ID</th>
                    <th style={thStyle}>Connected At</th>
                    <th style={thStyle}>Subscriptions</th>
                  </tr>
                </thead>
                <tbody>
                  {status.clients.map(c => (
                    <tr key={c.client_id}>
                      <td style={{ ...tdStyle, fontFamily: 'JetBrains Mono,monospace', fontSize: 12 }}>{c.client_id}</td>
                      <td style={tdStyle}>{new Date(c.connected_at * 1000).toLocaleString()}</td>
                      <td style={tdStyle}>{c.subscriptions.join(', ') || '-'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>

      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
        <div style={{ fontSize: 14, fontWeight: 600 }}>Configuration</div>
        <button style={{ ...btnStyle, fontSize: 12 }} onClick={() => setShowConfig(s => !s)}>
          {showConfig ? 'Hide Config' : 'Edit Config'}
        </button>
      </div>

      {showConfig && (
        <form onSubmit={handleSaveConfig} style={cardStyle}>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, marginBottom: 16 }}>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Port</label>
              <input
                type="number"
                value={port}
                onChange={e => setPort(Number(e.target.value))}
                style={inputStyle}
                min={1}
                max={65535}
              />
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              <label style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase' }}>Max Connections</label>
              <input
                type="number"
                value={maxConnections}
                onChange={e => setMaxConnections(Number(e.target.value))}
                style={inputStyle}
                min={1}
              />
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, padding: 12, background: 'var(--bg)', borderRadius: 8 }}>
            <label style={{ fontSize: 13, fontWeight: 500 }}>Authentication Required</label>
            <button
              type="button"
              onClick={() => setAuthEnabled(!authEnabled)}
              style={{
                width: 40, height: 22, borderRadius: 11, border: 'none', cursor: 'pointer',
                background: authEnabled ? 'var(--green)' : 'var(--border)',
                position: 'relative', transition: 'background 0.2s',
              }}
            >
              <span style={{
                position: 'absolute', top: 2,
                left: authEnabled ? 20 : 2,
                width: 18, height: 18, borderRadius: '50%', background: '#fff',
                transition: 'left 0.2s',
              }} />
            </button>
          </div>

          {authEnabled && (
            <div style={{ marginBottom: 16 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
                <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--muted)', textTransform: 'uppercase' }}>Users</div>
                <button type="button" style={{ ...btnStyle, fontSize: 11, padding: '4px 10px', borderColor: 'var(--cyan)', color: 'var(--cyan)' }} onClick={addUser}>
                  + Add User
                </button>
              </div>
              {users.length === 0 ? (
                <div style={{ padding: 16, textAlign: 'center', color: 'var(--muted)', fontSize: 12, background: 'var(--bg)', borderRadius: 8 }}>
                  No users configured. Click "+ Add User" to add MQTT credentials.
                </div>
              ) : (
                <div style={{ overflow: 'auto', background: 'var(--bg)', borderRadius: 8, border: '1px solid var(--border)' }}>
                  <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                    <thead>
                      <tr>
                        <th style={thStyle}>Username</th>
                        <th style={thStyle}>Password</th>
                        <th style={{ ...thStyle, width: 80 }}></th>
                      </tr>
                    </thead>
                    <tbody>
                      {users.map((u, i) => (
                        <tr key={i}>
                          <td style={tdStyle}>
                            <input
                              value={u.username}
                              onChange={e => updateUser(i, 'username', e.target.value)}
                              style={{ ...inputStyle, fontSize: 12 }}
                              placeholder="username"
                            />
                          </td>
                          <td style={tdStyle}>
                            <input
                              type="password"
                              value={u.password}
                              onChange={e => updateUser(i, 'password', e.target.value)}
                              style={{ ...inputStyle, fontSize: 12 }}
                              placeholder="password"
                            />
                          </td>
                          <td style={tdStyle}>
                            <button
                              type="button"
                              onClick={() => removeUser(i)}
                              style={{ ...btnStyle, fontSize: 11, padding: '4px 8px', borderColor: 'var(--crit)', color: 'var(--crit)' }}
                            >
                              Remove
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          )}

          {updateMut.error && (
            <div style={{ color: 'var(--crit)', fontSize: 12, marginBottom: 12 }}>
              {updateMut.error.message}
            </div>
          )}

          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
            <button type="button" style={btnStyle} onClick={() => setShowConfig(false)}>Cancel</button>
            <button
              type="submit"
              style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }}
              disabled={updateMut.isPending}
            >
              {updateMut.isPending ? 'Saving...' : 'Save Configuration'}
            </button>
          </div>
        </form>
      )}
    </>
  )

  if (embedded) {
    return <>{statusContent}</>
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1200, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, flexShrink: 0 }}>
          <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>MQTT Broker</h1>
        </div>
        {isLoading ? (
          <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--muted)' }}>
            Loading broker status...
          </div>
        ) : (
          <div style={{ flex: 1, overflow: 'auto' }}>
            {statusContent}
          </div>
        )}
      </div>
    </div>
  )
}
