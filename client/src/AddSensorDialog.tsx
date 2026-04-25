import { useState } from 'react';
import { SensorConfig } from './types';
import { inputStyle, monoInputStyle, labelStyle, labelTextStyle, tabStyle } from './styles';

export function AddSensorDialog({ screenId, sensors, screenSensorIds, onClose, onDone }: {
  screenId: string; sensors: SensorConfig[]; screenSensorIds: number[];
  onClose: () => void; onDone: () => void;
}) {
  const [tab, setTab] = useState<'existing' | 'new'>('existing');
  const available = sensors.filter(s => !screenSensorIds.includes(s.id));
  const [newForm, setNewForm] = useState({ key: '', name: '', unit: '', min_val: 0, max_val: 100, warn_val: 80, crit_val: 90, base_val: 50, decimals: 1, source_type: 'random_walk', color: '#8b5cf6' });

  const addExisting = async (sensorId: number) => {
    await fetch(`/api/screens/${screenId}/sensors`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sensor_id: sensorId }) });
    onDone();
  };

  const createNew = async () => {
    const res = await fetch('/api/sensors', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(newForm) });
    const { id } = await res.json();
    await fetch(`/api/screens/${screenId}/sensors`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sensor_id: id }) });
    onDone();
  };

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 480, maxHeight: '80vh', overflow: 'auto' }} onClick={e => e.stopPropagation()}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '12px 16px', borderBottom: '1px solid var(--border)' }}>
          <div style={{ fontSize: 13, fontWeight: 600 }}>Add Data Source</div>
          <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18 }}>&times;</button>
        </div>
        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
          <button style={tabStyle(tab === 'existing')} onClick={() => setTab('existing')}>Existing</button>
          <button style={tabStyle(tab === 'new')} onClick={() => setTab('new')}>Create New</button>
        </div>
        <div style={{ padding: 16 }}>
          {tab === 'existing' ? (
            available.length === 0 ? <div style={{ color: 'var(--muted)', fontSize: 13, padding: 12 }}>All sources already on this screen.</div> :
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              {available.map(s => (
                <button key={s.id} onClick={() => addExisting(s.id)} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 12px', background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6, cursor: 'pointer', color: 'var(--fg)', width: '100%' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <div style={{ width: 8, height: 8, borderRadius: '50%', background: s.color }} />
                    <span style={{ fontWeight: 600, fontSize: 13 }}>{s.name}</span>
                    <span style={{ color: 'var(--muted)', fontSize: 11 }}>({s.key})</span>
                  </div>
                  <span style={{ fontSize: 11, color: 'var(--cyan)' }}>+ Add</span>
                </button>
              ))}
            </div>
          ) : (
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              <label style={labelStyle}><span style={labelTextStyle}>Key</span><input value={newForm.key} onChange={e => setNewForm({ ...newForm, key: e.target.value })} style={inputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Name</span><input value={newForm.name} onChange={e => setNewForm({ ...newForm, name: e.target.value })} style={inputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Unit</span><input value={newForm.unit} onChange={e => setNewForm({ ...newForm, unit: e.target.value })} style={inputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Min</span><input type="number" step="any" value={newForm.min_val} onChange={e => setNewForm({ ...newForm, min_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Max</span><input type="number" step="any" value={newForm.max_val} onChange={e => setNewForm({ ...newForm, max_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Base Value</span><input type="number" step="any" value={newForm.base_val} onChange={e => setNewForm({ ...newForm, base_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <button onClick={createNew} style={{ gridColumn: 'span 2', marginTop: 8, padding: '8px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>Create & Add</button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
