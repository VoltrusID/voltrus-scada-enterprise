import { useState } from 'react'
import { inputStyle, monoInputStyle, labelStyle, labelTextStyle } from './styles'
import type { ModbusDevice, ModbusRegister } from './types'

interface RegisterRow {
  name: string
  address: number
  register_type: string
  scale: number
}

interface Props {
  device?: ModbusDevice & { registers?: ModbusRegister[] }
  onSave: (data: { id: string; name: string; ip: string; port: number; unit_id: number; interval_ms: number; endianness: string; registers: RegisterRow[] }) => void
  onClose: () => void
}

const REG_TYPES = ['float32', 'int32', 'uint16', 'int16']
const ENCODINGS = ['big', 'little', 'swapped']

export function ModbusDeviceForm({ device, onSave, onClose }: Props) {
  const isEdit = !!device
  const [id, setId] = useState(device?.id ?? '')
  const [name, setName] = useState(device?.name ?? '')
  const [ip, setIp] = useState(device?.ip ?? '')
  const [port, setPort] = useState(device?.port ?? 502)
  const [unitId, setUnitId] = useState(device?.unit_id ?? 1)
  const [intervalMs, setIntervalMs] = useState(device?.interval_ms ?? 1000)
  const [endianness, setEndianness] = useState(device?.endianness ?? 'big')
  const [registers, setRegisters] = useState<RegisterRow[]>(
    device?.registers?.map(r => ({ name: r.name, address: r.address, register_type: r.register_type, scale: r.scale })) ?? []
  )

  const addRegister = () => setRegisters([...registers, { name: '', address: 0, register_type: 'float32', scale: 1 }])
  const removeRegister = (i: number) => setRegisters(registers.filter((_, j) => j !== i))
  const updateRegister = (i: number, field: keyof RegisterRow, value: any) => {
    const next = [...registers]
    next[i] = { ...next[i], [field]: value }
    setRegisters(next)
  }

  const handleSave = () => {
    if (!id || !ip) return
    onSave({ id, name: name || id, ip, port, unit_id: unitId, interval_ms: intervalMs, endianness, registers })
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 520, maxHeight: '80vh', overflow: 'auto' }} onClick={e => e.stopPropagation()}>
        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border)', fontSize: 13, fontWeight: 600 }}>
          {isEdit ? `Edit: ${device!.name}` : 'Add Modbus Device'}
        </div>
        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 8 }}>
          {!isEdit && (
            <label style={labelStyle}>
              <span style={labelTextStyle}>Device ID</span>
              <input value={id} onChange={e => setId(e.target.value)} placeholder="plts_1" style={inputStyle} />
            </label>
          )}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
            <label style={labelStyle}><span style={labelTextStyle}>Name</span><input value={name} onChange={e => setName(e.target.value)} placeholder="PLTS 1" style={inputStyle} /></label>
            <label style={labelStyle}><span style={labelTextStyle}>IP Address</span><input value={ip} onChange={e => setIp(e.target.value)} placeholder="192.168.1.10" style={monoInputStyle} /></label>
            <label style={labelStyle}><span style={labelTextStyle}>Port</span><input type="number" value={port} onChange={e => setPort(parseInt(e.target.value) || 502)} style={monoInputStyle} /></label>
            <label style={labelStyle}><span style={labelTextStyle}>Unit ID</span><input type="number" value={unitId} onChange={e => setUnitId(parseInt(e.target.value) || 1)} style={monoInputStyle} /></label>
            <label style={labelStyle}><span style={labelTextStyle}>Interval (ms)</span><input type="number" value={intervalMs} onChange={e => setIntervalMs(parseInt(e.target.value) || 1000)} style={monoInputStyle} /></label>
            <label style={labelStyle}><span style={labelTextStyle}>Endianness</span><select value={endianness} onChange={e => setEndianness(e.target.value)} style={inputStyle}>{ENCODINGS.map(e => <option key={e} value={e}>{e}</option>)}</select></label>
          </div>

          <div style={{ marginTop: 8, borderTop: '1px solid var(--border)', paddingTop: 8 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
              <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--fg)' }}>Registers</span>
              <button onClick={addRegister} style={{ padding: '3px 10px', borderRadius: 4, fontSize: 11, cursor: 'pointer', border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--cyan)' }}>+ Add</button>
            </div>
            {registers.length === 0 && <div style={{ color: 'var(--muted)', fontSize: 12, padding: 8 }}>No registers configured.</div>}
            {registers.map((reg, i) => (
              <div key={i} style={{ display: 'grid', gridTemplateColumns: '1fr 70px 80px 60px 28px', gap: 4, marginBottom: 4, alignItems: 'end' }}>
                <input value={reg.name} onChange={e => updateRegister(i, 'name', e.target.value)} placeholder="voltage" style={{ ...monoInputStyle, fontSize: 12 }} />
                <input type="number" value={reg.address} onChange={e => updateRegister(i, 'address', parseInt(e.target.value) || 0)} placeholder="40001" style={{ ...monoInputStyle, fontSize: 12 }} />
                <select value={reg.register_type} onChange={e => updateRegister(i, 'register_type', e.target.value)} style={{ ...inputStyle, fontSize: 11 }}>{REG_TYPES.map(t => <option key={t} value={t}>{t}</option>)}</select>
                <input type="number" step="0.01" value={reg.scale} onChange={e => updateRegister(i, 'scale', parseFloat(e.target.value) || 1)} style={{ ...monoInputStyle, fontSize: 12 }} />
                <button onClick={() => removeRegister(i)} style={{ background: 'none', border: 'none', color: 'var(--crit)', cursor: 'pointer', fontSize: 14 }}>&times;</button>
              </div>
            ))}
          </div>
        </div>
        <div style={{ display: 'flex', gap: 8, padding: '0 16px 16px' }}>
          <button onClick={handleSave} style={{ padding: '6px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13 }}>{isEdit ? 'Update' : 'Create'}</button>
          <button onClick={onClose} style={{ padding: '6px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--muted)', cursor: 'pointer', fontSize: 13 }}>Cancel</button>
        </div>
      </div>
    </div>
  )
}
