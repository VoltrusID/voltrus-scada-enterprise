import { useState } from 'react'
import { useModbusDevices, useModbusDevice, useModbusPollerStatus, useCreateModbusDevice, useUpdateModbusDevice, useDeleteModbusDevice, useRestartModbusPoller } from './queries'
import { btnStyle } from './styles'
import { ModbusDeviceForm } from './ModbusDeviceForm'
import type { ModbusDevice, ModbusRegister } from './types'

export function ModbusDeviceManager({ onClose }: { onClose: () => void }) {
  const { data: devices = [], isLoading } = useModbusDevices()
  const { data: statuses = [] } = useModbusPollerStatus()
  const createMut = useCreateModbusDevice()
  const updateMut = useUpdateModbusDevice()
  const deleteMut = useDeleteModbusDevice()
  const restartMut = useRestartModbusPoller()

  const [showForm, setShowForm] = useState(false)
  const [editDevice, setEditDevice] = useState<(ModbusDevice & { registers?: ModbusRegister[] }) | undefined>(undefined)

  const statusMap = Object.fromEntries(statuses.map(s => [s.device_id, s.running]))

  const handleCreate = (data: any) => {
    createMut.mutate(data, { onSuccess: () => setShowForm(false) })
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
    // Fetch full device with registers
    const res = await fetch(`/api/modbus-devices/${d.id}`)
    if (res.ok) {
      const data = await res.json()
      setEditDevice({ ...d, registers: data.registers })
    }
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '10px 16px', borderBottom: '1px solid var(--border)' }}>
        <span style={{ fontSize: 14, fontWeight: 600 }}>Modbus Devices</span>
        <div style={{ display: 'flex', gap: 8 }}>
          <button onClick={() => setShowForm(true)} style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)' }}>+ Add Device</button>
          <button onClick={onClose} style={btnStyle}>Back</button>
        </div>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 16 }}>
        {isLoading && <div style={{ color: 'var(--muted)', fontSize: 13 }}>Loading...</div>}
        {!isLoading && devices.length === 0 && <div style={{ color: 'var(--muted)', fontSize: 13 }}>No Modbus devices configured.</div>}
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ borderBottom: '1px solid var(--border)', textAlign: 'left' }}>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>ID</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Name</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>IP</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Port</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Unit</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Interval</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Endian</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Registers</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Status</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Enabled</th>
              <th style={{ padding: '6px 8px', color: 'var(--muted)', fontSize: 11, textTransform: 'uppercase' }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {devices.map(d => (
              <tr key={d.id} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '8px', fontFamily: 'JetBrains Mono,monospace', fontSize: 12 }}>{d.id}</td>
                <td style={{ padding: '8px' }}>{d.name || d.id}</td>
                <td style={{ padding: '8px', fontFamily: 'JetBrains Mono,monospace', fontSize: 12 }}>{d.ip}</td>
                <td style={{ padding: '8px', fontFamily: 'JetBrains Mono,monospace', fontSize: 12 }}>{d.port}</td>
                <td style={{ padding: '8px' }}>{d.unit_id}</td>
                <td style={{ padding: '8px' }}>{d.interval_ms}ms</td>
                <td style={{ padding: '8px' }}>{d.endianness}</td>
                <td style={{ padding: '8px' }}>{d.register_count}</td>
                <td style={{ padding: '8px' }}>
                  <span style={{ fontSize: 10, padding: '2px 8px', borderRadius: 4, background: statusMap[d.id] ? 'rgba(34,197,94,.1)' : 'rgba(239,68,68,.1)', color: statusMap[d.id] ? 'var(--cyan)' : 'var(--crit)' }}>
                    {d.enabled ? (statusMap[d.id] ? 'polling' : 'stopped') : 'disabled'}
                  </span>
                </td>
                <td style={{ padding: '8px' }}>
                  <input type="checkbox" checked={d.enabled} onChange={() => updateMut.mutate({ id: d.id, enabled: !d.enabled })} style={{ cursor: 'pointer' }} />
                </td>
                <td style={{ padding: '8px' }}>
                  <div style={{ display: 'flex', gap: 4 }}>
                    <button onClick={() => handleEdit(d)} style={{ ...btnStyle, fontSize: 11, padding: '4px 8px' }}>Edit</button>
                    <button onClick={() => restartMut.mutate(d.id)} style={{ ...btnStyle, fontSize: 11, padding: '4px 8px' }}>Restart</button>
                    <button onClick={() => handleDelete(d.id)} style={{ ...btnStyle, fontSize: 11, padding: '4px 8px', borderColor: 'var(--crit)', color: 'var(--crit)' }}>Delete</button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {showForm && <ModbusDeviceForm onSave={handleCreate} onClose={() => setShowForm(false)} />}
      {editDevice && <ModbusDeviceForm device={editDevice} onSave={handleUpdate} onClose={() => setEditDevice(undefined)} />}
    </div>
  )
}
