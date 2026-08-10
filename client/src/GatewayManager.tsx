// Gateway manager — Discover / Browse / Import against a standalone voltrus-gateway.
// Reuses the per-protocol device endpoints on import: a browsed tag becomes a
// normal Modbus register or OPC-UA node, so the existing poller + cache + WS
// pipeline serves it. No gateway-specific device type.
import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { btnStyle, inputStyle } from './styles'
import {
  useGatewayInfo,
  useGatewayEndpoint,
  useSetGatewayEndpoint,
  useDiscoverGateway,
  useBrowseGateway,
  type BrowsedTag,
} from './queries'
import { queryKeys } from './queries/keys'
import { fetchOk } from './queries/fetch'

type Protocol = 'modbus' | 'opcua'

const cardStyle: React.CSSProperties = {
  background: 'var(--card)',
  border: '1px solid var(--border)',
  borderRadius: 10,
  padding: 16,
  marginBottom: 12,
}
const labelStyle: React.CSSProperties = {
  fontSize: 11,
  color: 'var(--muted)',
  textTransform: 'uppercase',
  marginBottom: 4,
  display: 'block',
}
const sectionTitle: React.CSSProperties = {
  fontSize: 13,
  fontWeight: 600,
  marginBottom: 10,
  paddingBottom: 6,
  borderBottom: '1px solid var(--border)',
}
const thStyle: React.CSSProperties = {
  textAlign: 'left',
  padding: '8px 12px',
  fontSize: 11,
  color: 'var(--muted)',
  textTransform: 'uppercase',
  borderBottom: '1px solid var(--border)',
}
const tdStyle: React.CSSProperties = {
  padding: '8px 12px',
  fontSize: 13,
  borderBottom: '1px solid var(--border)',
  fontFamily: "'JetBrains Mono', monospace",
}

export function GatewayManager({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const infoQ = useGatewayInfo()
  const endpointQ = useGatewayEndpoint()
  const setEndpointMut = useSetGatewayEndpoint()
  const discoverMut = useDiscoverGateway()
  const browseMut = useBrowseGateway()

  const [endpointInput, setEndpointInput] = useState('')
  const [protocol, setProtocol] = useState<Protocol>('modbus')
  // modbus connect params
  const [mIp, setMIp] = useState('127.0.0.1')
  const [mPort, setMPort] = useState(502)
  const [mUnit, setMUnit] = useState(1)
  // opcua connect params
  const [oUrl, setOUrl] = useState('opc.tcp://127.0.0.1:4840')
  const [oSecPolicy, setOSecPolicy] = useState('None')
  const [oSecMode, setOSecMode] = useState('None')
  const [oUser, setOUser] = useState('')
  const [oPass, setOPass] = useState('')
  // results
  const [tags, setTags] = useState<BrowsedTag[]>([])
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [deviceName, setDeviceName] = useState('')
  const [importing, setImporting] = useState(false)
  const [importMsg, setImportMsg] = useState<string | null>(null)

  // Seed endpoint input once the persisted value loads.
  if (!endpointInput && endpointQ.data?.endpoint) {
    setEndpointInput(endpointQ.data.endpoint)
  }

  const reachable = infoQ.data?.reachable === true
  const info = infoQ.data

  const connectParams = (): Record<string, unknown> => {
    if (protocol === 'modbus') {
      return { ip: mIp, port: mPort, unit_id: mUnit, endianness: 'big' }
    }
    return {
      endpoint_url: oUrl,
      security_policy: oSecPolicy,
      security_mode: oSecMode,
      username: oUser,
      password: oPass,
    }
  }

  const onDiscover = () => {
    discoverMut.mutate(
      { protocol: 'modbus', ip: mIp, port: mPort, unit_id: mUnit },
      {
        onSuccess: (r) => setImportMsg(`${r.devices.length} device(s) found on ${mIp}.`),
        onError: (e: unknown) => setImportMsg(`discover failed: ${String(e)}`),
      },
    )
  }

  const onBrowse = () => {
    setSelected(new Set())
    browseMut.mutate(
      { protocol, params: connectParams() },
      {
        onSuccess: (r) => {
          setTags(r.tags)
          setImportMsg(r.tags.length ? `${r.tags.length} tags browsed.` : 'No tags returned.')
        },
        onError: (e: unknown) => {
          setTags([])
          setImportMsg(`browse failed: ${String(e)}`)
        },
      },
    )
  }

  const toggle = (name: string) => {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(name)) next.delete(name)
      else next.add(name)
      return next
    })
  }

  const onImport = async () => {
    const chosen = tags.filter((t) => selected.has(t.name))
    if (chosen.length === 0) return
    setImporting(true)
    setImportMsg(null)
    try {
      const name = deviceName.trim() || `${protocol}-${Date.now()}`
      if (protocol === 'modbus') {
        const res = await fetchOk('/api/v1/modbus-devices', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            name,
            ip: mIp,
            port: mPort,
            unit_id: mUnit,
            interval_ms: 1000,
            endianness: 'big',
            enabled: true,
          }),
        })
        const dev = (await res.json()) as { id: unknown }
        const id = String(dev.id)
        const registers = chosen.map((t) => ({
          name: t.name,
          address: Number(t.address ?? 0),
          register_type: t.data_type || 'float32',
          scale: Number(t.scale ?? 1),
        }))
        await fetchOk(`/api/v1/modbus-devices/${id}/registers`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(registers),
        })
        qc.invalidateQueries({ queryKey: queryKeys.modbusDevices })
      } else {
        const res = await fetchOk('/api/v1/opcua/devices', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            name,
            endpoint_url: oUrl,
            security_policy: oSecPolicy,
            security_mode: oSecMode,
            username: oUser,
            password: oPass,
          }),
        })
        const dev = (await res.json()) as { id: unknown }
        const id = String(dev.id)
        for (const t of chosen) {
          await fetchOk(`/api/v1/opcua/devices/${id}/nodes`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              node_id: t.node_id || t.name,
              name: t.name,
              data_type: t.data_type || 'Double',
            }),
          })
        }
        qc.invalidateQueries({ queryKey: ['opcua-devices'] })
      }
      setImportMsg(`Imported ${chosen.length} ${protocol} tag(s) as "${name}".`)
      navigate({ to: '/data', search: (prev) => ({ ...prev, tab: protocol }) })
    } catch (e) {
      setImportMsg(`import failed: ${String(e)}`)
    } finally {
      setImporting(false)
    }
  }

  const tableContent = (
    <>
      {/* Endpoint bar */}
      <div style={cardStyle}>
        <div style={sectionTitle}>Gateway Endpoint</div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          <input
            value={endpointInput}
            onChange={(e) => setEndpointInput(e.target.value)}
            placeholder="http://127.0.0.1:50061"
            style={{ ...inputStyle, flex: 1, minWidth: 240, fontFamily: "'JetBrains Mono', monospace" }}
          />
          <button
            style={btnStyle}
            onClick={() => setEndpointMut.mutate(endpointInput.trim())}
            disabled={setEndpointMut.isPending || !endpointInput.trim()}
          >
            {setEndpointMut.isPending ? 'Saving…' : 'Save'}
          </button>
          <button style={btnStyle} onClick={() => infoQ.refetch()} disabled={infoQ.isFetching}>
            {infoQ.isFetching ? 'Testing…' : 'Test'}
          </button>
          <span
            style={{
              width: 9,
              height: 9,
              borderRadius: '50%',
              background: reachable ? 'var(--ok)' : 'var(--crit)',
              display: 'inline-block',
            }}
            title={reachable ? 'reachable' : info?.error || 'unreachable'}
          />
          <span style={{ fontSize: 12, color: 'var(--muted)' }}>
            {reachable ? `${info?.version ?? 'gateway'} · up ${String(info?.uptime_ms ?? '?')}ms` : info?.error || 'not reachable'}
          </span>
        </div>
      </div>

      {/* Protocol selector */}
      <div style={cardStyle}>
        <div style={sectionTitle}>Connect</div>
        <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
          {(['modbus', 'opcua'] as Protocol[]).map((p) => (
            <button
              key={p}
              onClick={() => setProtocol(p)}
              style={{
                ...btnStyle,
                borderColor: protocol === p ? 'var(--cyan)' : 'var(--border)',
                color: protocol === p ? 'var(--cyan)' : 'var(--muted)',
                fontWeight: protocol === p ? 600 : 400,
              }}
            >
              {p === 'modbus' ? 'Modbus' : 'OPC-UA'}
            </button>
          ))}
        </div>

        {protocol === 'modbus' ? (
          <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr 1fr', gap: 8 }}>
            <div>
              <label style={labelStyle}>IP</label>
              <input value={mIp} onChange={(e) => setMIp(e.target.value)} style={{ ...inputStyle, fontFamily: "'JetBrains Mono', monospace" }} />
            </div>
            <div>
              <label style={labelStyle}>Port</label>
              <input type="number" value={mPort} onChange={(e) => setMPort(Number(e.target.value) || 502)} style={inputStyle} />
            </div>
            <div>
              <label style={labelStyle}>Unit ID</label>
              <input type="number" value={mUnit} onChange={(e) => setMUnit(Number(e.target.value) || 1)} style={inputStyle} />
            </div>
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
            <div style={{ gridColumn: '1 / span 2' }}>
              <label style={labelStyle}>Endpoint URL</label>
              <input value={oUrl} onChange={(e) => setOUrl(e.target.value)} style={{ ...inputStyle, fontFamily: "'JetBrains Mono', monospace" }} />
            </div>
            <div>
              <label style={labelStyle}>Security Policy</label>
              <input value={oSecPolicy} onChange={(e) => setOSecPolicy(e.target.value)} style={inputStyle} />
            </div>
            <div>
              <label style={labelStyle}>Security Mode</label>
              <input value={oSecMode} onChange={(e) => setOSecMode(e.target.value)} style={inputStyle} />
            </div>
            <div>
              <label style={labelStyle}>Username</label>
              <input value={oUser} onChange={(e) => setOUser(e.target.value)} style={inputStyle} />
            </div>
            <div>
              <label style={labelStyle}>Password</label>
              <input type="password" value={oPass} onChange={(e) => setOPass(e.target.value)} style={inputStyle} />
            </div>
          </div>
        )}

        <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
          {protocol === 'modbus' && (
            <button style={btnStyle} onClick={onDiscover} disabled={discoverMut.isPending}>
              {discoverMut.isPending ? 'Scanning…' : 'Discover'}
            </button>
          )}
          <button style={btnStyle} onClick={onBrowse} disabled={browseMut.isPending}>
            {browseMut.isPending ? 'Browsing…' : 'Browse Tags'}
          </button>
        </div>

        {discoverMut.data && protocol === 'modbus' && (
          <div style={{ marginTop: 10 }}>
            <div style={labelStyle}>Discovered devices</div>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={thStyle}>Address</th>
                  <th style={thStyle}>Identity</th>
                </tr>
              </thead>
              <tbody>
                {discoverMut.data.devices.map((d, i) => (
                  <tr key={i}>
                    <td style={tdStyle}>{d.address}</td>
                    <td style={tdStyle}>{d.identity}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Browsed tags */}
      {tags.length > 0 && (
        <div style={cardStyle}>
          <div style={{ ...sectionTitle, display: 'flex', justifyContent: 'space-between' }}>
            <span>Browsed Tags ({selected.size}/{tags.length} selected)</span>
            <button style={{ ...btnStyle, padding: '2px 8px', fontSize: 11 }} onClick={() => setSelected(new Set(tags.map((t) => t.name)))}>
              All
            </button>
          </div>
          <div style={{ maxHeight: 280, overflow: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={thStyle}></th>
                  <th style={thStyle}>Name</th>
                  <th style={thStyle}>Data Type</th>
                  {protocol === 'modbus' ? (
                    <>
                      <th style={thStyle}>Address</th>
                      <th style={thStyle}>Area</th>
                    </>
                  ) : (
                    <th style={thStyle}>Node ID</th>
                  )}
                </tr>
              </thead>
              <tbody>
                {tags.map((t) => (
                  <tr key={t.name}>
                    <td style={tdStyle}>
                      <input type="checkbox" checked={selected.has(t.name)} onChange={() => toggle(t.name)} />
                    </td>
                    <td style={tdStyle}>{t.name}</td>
                    <td style={tdStyle}>{t.data_type ?? '—'}</td>
                    {protocol === 'modbus' ? (
                      <>
                        <td style={tdStyle}>{t.address ?? '—'}</td>
                        <td style={tdStyle}>{t.register_area ?? '—'}</td>
                      </>
                    ) : (
                      <td style={tdStyle}>{t.node_id ?? '—'}</td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginTop: 12 }}>
            <input
              value={deviceName}
              onChange={(e) => setDeviceName(e.target.value)}
              placeholder={`device name (default: ${protocol}-<ts>)`}
              style={{ ...inputStyle, flex: 1 }}
            />
            <button
              style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }}
              onClick={onImport}
              disabled={importing || selected.size === 0}
            >
              {importing ? 'Importing…' : `Import ${selected.size || ''} → ${protocol}`}
            </button>
          </div>
        </div>
      )}

      {importMsg && (
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 4 }}>{importMsg}</div>
      )}
    </>
  )

  if (embedded) return <>{tableContent}</>
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1000, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
          <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>
            Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>Gateway</h1>
        </div>
        <div style={{ flex: 1, overflow: 'auto' }}>{tableContent}</div>
      </div>
    </div>
  )
}
