import { useState, useEffect } from 'react'
import { useModbusDevices, useModbusPollerStatus, useCreateModbusDevice, useUpdateModbusDevice, useDeleteModbusDevice, useRestartModbusPoller } from './queries'
import { inputStyle, btnStyle } from './styles'
import { ModbusDeviceForm } from './ModbusDeviceForm'
import type { ModbusDevice, ModbusRegister } from './types'

export function ModbusDeviceManager({ onClose, embedded }: { onClose: () => void; embedded?: boolean }) {
  const { data: devices = [], isLoading } = useModbusDevices()
  const { data: statuses = [] } = useModbusPollerStatus()
  const createMut = useCreateModbusDevice()
  const updateMut = useUpdateModbusDevice()
  const deleteMut = useDeleteModbusDevice()
  const restartMut = useRestartModbusPoller()

  const [filter, setFilter] = useState('')
  const [showForm, setShowForm] = useState(false)
  const [editDevice, setEditDevice] = useState<(ModbusDevice & { registers?: ModbusRegister[] }) | undefined>(undefined)
  const [dupDevice, setDupDevice] = useState<(ModbusDevice & { registers?: ModbusRegister[] }) | undefined>(undefined)

  const statusMap = Object.fromEntries(statuses.map(s => [s.device_id, s.status || (s.running ? 'online' : 'offline')]))
  const filtered = devices.filter(d => (d.name || d.id).toLowerCase().includes(filter) || d.ip.toLowerCase().includes(filter))

  useEffect(() => {
    if (embedded) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose, embedded]);

  const thStyle: React.CSSProperties = { textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' };
  const tdStyle: React.CSSProperties = { padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)' };

  const handleDup = async (d: ModbusDevice) => {
    const res = await fetch(`/api/v1/modbus-devices/${d.id}`)
    if (res.ok) {
      const data = await res.json()
      setDupDevice({ ...d, registers: data.registers })
      setShowForm(true)
    }
  }

  const handleCreate = (data: any) => {
    createMut.mutate(data, { onSuccess: () => { setShowForm(false); setDupDevice(undefined) } })
  }

  const handleUpdate = (data: any) => {
    if (!editDevice) return
    updateMut.mutate({ id: editDevice.id, ...data }, {
      onSuccess: () => { setEditDevice(undefined) }
    })
  }

  const handleDelete = (id: string) => {
    if (!confirm('Delete this device and stop its poller?')) return
    deleteMut.mutate(id)
  }

  const handleEdit = async (d: ModbusDevice) => {
    const res = await fetch(`/api/v1/modbus-devices/${d.id}`)
    if (res.ok) {
      const data = await res.json()
      setEditDevice({ ...d, registers: data.registers })
    }
  }

  const tableContent = (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
        <span style={{ color: 'var(--muted)', fontSize: 13 }}>{devices.length} devices</span>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="Search..." value={filter} onChange={e => setFilter(e.target.value)} style={{ ...inputStyle, width: 200 }} />
          <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={() => setShowForm(true)}>+ Add Device</button>
        </div>
      </div>
      <div style={{ flex: 1, overflow: 'auto', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr>
              <th style={thStyle}>Name</th>
              <th style={thStyle}>IP</th>
              <th style={thStyle}>Port</th>
              <th style={thStyle}>Unit</th>
              <th style={thStyle}>Interval</th>
              <th style={thStyle}>Endian</th>
              <th style={thStyle}>Registers</th>
              <th style={thStyle}>Status</th>
              <th style={thStyle}>Enabled</th>
              <th style={{ ...thStyle, width: 180 }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(d => (
              <tr key={d.id}>
                <td style={tdStyle}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <div style={{ width: 8, height: 8, borderRadius: '50%', background: statusMap[d.id] === 'online' ? 'var(--cyan)' : 'var(--crit)' }} />
                    {d.name || d.id}
                  </div>
                </td>
                <td style={{ ...tdStyle, fontFamily: "'JetBrains Mono',monospace", fontSize: 12 }}>{d.ip}</td>
                <td style={{ ...tdStyle, fontFamily: "'JetBrains Mono',monospace", fontSize: 12 }}>{d.port}</td>
                <td style={tdStyle}>{d.unit_id}</td>
                <td style={tdStyle}>{d.interval_ms}ms</td>
                <td style={tdStyle}><span style={{ fontSize: 11, padding: '2px 6px', borderRadius: 4, background: 'rgba(139,92,246,.1)', color: '#a78bfa' }}>{d.endianness}</span></td>
                <td style={tdStyle}>{d.register_count}</td>
                <td style={tdStyle}>
                  <span style={{ fontSize: 11, padding: '2px 6px', borderRadius: 4, background: statusMap[d.id] === 'online' ? 'rgba(34,197,94,.1)' : 'rgba(239,68,68,.1)', color: statusMap[d.id] === 'online' ? 'var(--cyan)' : 'var(--crit)' }}>
                    {d.enabled ? (statusMap[d.id] === 'online' ? 'online' : 'offline') : 'disabled'}
                  </span>
                </td>
                <td style={tdStyle}>
                  <input type="checkbox" checked={d.enabled} onChange={() => updateMut.mutate({ id: d.id, enabled: !d.enabled })} style={{ cursor: 'pointer' }} />
                </td>
                <td style={tdStyle}>
                  <div style={{ display: 'flex', gap: 4 }}>
                    <button onClick={() => handleEdit(d)} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 11 }}>Edit</button>
                    <button onClick={() => handleDup(d)} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 11 }}>Dup</button>
                    <button onClick={() => restartMut.mutate(d.id)} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 11 }}>Restart</button>
                    <button onClick={() => handleDelete(d.id)} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 11 }}>Del</button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );

  const modals = (<>
    {showForm && <ModbusDeviceForm dupDevice={dupDevice} onSave={handleCreate} onClose={() => { setShowForm(false); setDupDevice(undefined) }} />}
    {editDevice && <ModbusDeviceForm device={editDevice} onSave={handleUpdate} onClose={() => setEditDevice(undefined)} />}
  </>);

  if (embedded) {
    return (<>{tableContent}{modals}</>);
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1200, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
            </button>
            <h1 style={{ fontSize: 20, fontWeight: 700 }}>Modbus Devices</h1>
          </div>
        </div>
        {tableContent}
      </div>
      {modals}
    </div>
  )
}
