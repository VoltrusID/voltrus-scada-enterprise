import { useState, useEffect } from 'react';
import { SensorConfig, SOURCE_TYPES } from './types';
import { inputStyle, monoInputStyle, labelStyle, labelTextStyle, btnStyle, tabStyle } from './styles';
import { EditPanel } from './EditPanel';

export function SensorManager({ sensors, onSave, onDelete, onAdd, onDuplicate, onClose, shortcuts }: {
  sensors: SensorConfig[]; onSave: (s: SensorConfig, shortcut?: string | null) => void; onDelete: (id: number) => void;
  onAdd: (s: Partial<SensorConfig>) => void; onDuplicate: (s: SensorConfig) => void; onClose: () => void; shortcuts: Record<number, string>;
}) {
  const [filter, setFilter] = useState('');
  const [editing, setEditing] = useState<SensorConfig | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [createForm, setCreateForm] = useState({ key: '', name: '', unit: '', min_val: 0, max_val: 100, warn_val: 80, crit_val: 90, base_val: 50, decimals: 1, source_type: 'random_walk', color: '#8b5cf6' });
  const filtered = sensors.filter(s => s.name.toLowerCase().includes(filter) || s.key.toLowerCase().includes(filter));

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

  const thStyle: React.CSSProperties = { textAlign: 'left', padding: '8px 12px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)' };
  const tdStyle: React.CSSProperties = { padding: '8px 12px', fontSize: 13, borderBottom: '1px solid var(--border)' };

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1200, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
            </button>
            <h1 style={{ fontSize: 20, fontWeight: 700 }}>Data Sources</h1>
            <span style={{ color: 'var(--muted)', fontSize: 13 }}>{sensors.length} sources</span>
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <input placeholder="Search..." value={filter} onChange={e => setFilter(e.target.value)} style={{ ...inputStyle, width: 200 }} />
            <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={() => setShowCreate(true)}>+ Create Source</button>
          </div>
        </div>
        <div style={{ flex: 1, overflow: 'auto', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={thStyle}>Name</th>
                <th style={thStyle}>Key</th>
                <th style={thStyle}>Type</th>
                <th style={thStyle}>Source</th>
                <th style={thStyle}>Chart</th>
                <th style={{ ...thStyle, width: 140 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(s => (
                <tr key={s.id}>
                  <td style={tdStyle}><div style={{ display: 'flex', alignItems: 'center', gap: 6 }}><div style={{ width: 8, height: 8, borderRadius: '50%', background: s.color }} />{s.name}</div></td>
                  <td style={{ ...tdStyle, fontFamily: "'JetBrains Mono',monospace", fontSize: 12, color: 'var(--muted)' }}>{s.key}</td>
                  <td style={tdStyle}>{s.unit || '—'}</td>
                  <td style={tdStyle}><span style={{ fontSize: 11, padding: '2px 6px', borderRadius: 4, background: 'rgba(34,197,94,.1)', color: 'var(--cyan)' }}>{s.source_type}</span></td>
                  <td style={tdStyle}><span style={{ fontSize: 11, padding: '2px 6px', borderRadius: 4, background: 'rgba(139,92,246,.1)', color: '#a78bfa' }}>{s.chart_type || 'sparkline'}</span></td>
                  <td style={tdStyle}>
                    <div style={{ display: 'flex', gap: 4 }}>
                      <button onClick={() => setEditing(s)} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 11 }}>Edit</button>
                      <button onClick={() => onDuplicate(s)} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 11 }}>Dup</button>
                      <button onClick={() => { if (confirm(`Delete ${s.name}?`)) onDelete(s.id); }} style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 11 }}>Del</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
      {editing && <EditPanel sensor={editing} activeScreen="all" onSave={(s, sc) => { onSave(s, sc); setEditing(null); }} onDelete={id => { onDelete(id); setEditing(null); }} onClose={() => setEditing(null)} shortcuts={shortcuts} />}
      {showCreate && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={() => setShowCreate(false)}>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 480, padding: 20 }} onClick={e => e.stopPropagation()}>
            <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Create New Data Source</div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              <label style={labelStyle}><span style={labelTextStyle}>Key</span><input value={createForm.key} onChange={e => setCreateForm({ ...createForm, key: e.target.value })} style={inputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Name</span><input value={createForm.name} onChange={e => setCreateForm({ ...createForm, name: e.target.value })} style={inputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Unit</span><input value={createForm.unit} onChange={e => setCreateForm({ ...createForm, unit: e.target.value })} style={inputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Color</span><input type="color" value={createForm.color} onChange={e => setCreateForm({ ...createForm, color: e.target.value })} style={{ height: 32, padding: 2, background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6 }} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Min</span><input type="number" step="any" value={createForm.min_val} onChange={e => setCreateForm({ ...createForm, min_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Max</span><input type="number" step="any" value={createForm.max_val} onChange={e => setCreateForm({ ...createForm, max_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Warn</span><input type="number" step="any" value={createForm.warn_val} onChange={e => setCreateForm({ ...createForm, warn_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Crit</span><input type="number" step="any" value={createForm.crit_val} onChange={e => setCreateForm({ ...createForm, crit_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Base Value</span><input type="number" step="any" value={createForm.base_val} onChange={e => setCreateForm({ ...createForm, base_val: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>
              <label style={labelStyle}><span style={labelTextStyle}>Source Type</span><select value={createForm.source_type} onChange={e => setCreateForm({ ...createForm, source_type: e.target.value })} style={inputStyle}>{SOURCE_TYPES.map(t => <option key={t} value={t}>{t}</option>)}</select></label>
            </div>
            <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
              <button onClick={() => { onAdd(createForm); setShowCreate(false); }} style={{ padding: '8px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>Create</button>
              <button onClick={() => setShowCreate(false)} style={{ padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
