import { useState, useEffect } from 'react';
import { SensorConfig } from './types';
import { SOURCE_TYPES } from './types';
import { inputStyle, monoInputStyle, labelStyle, labelTextStyle, tabStyle, parseConfig, normalizeKeyEvent } from './styles';
import { SourceConfigFields } from './SourceConfigFields';

export function EditPanel({ sensor, activeScreen, onSave, onDelete, onClose, shortcuts }: { sensor: SensorConfig; activeScreen: string; onSave: (s: SensorConfig, shortcut?: string | null) => void; onDelete: (id: number) => void; onClose: () => void; shortcuts: Record<number, string> }) {
  const [form, setForm] = useState(sensor);
  const [tab, setTab] = useState<'general' | 'source'>('general');
  const [srcConfig, setSrcConfig] = useState<Record<string, any>>(parseConfig(sensor.source_config));
  const [recording, setRecording] = useState(false);
  const [capturedShortcut, setCapturedShortcut] = useState<string | null>(shortcuts[sensor.id] ?? null);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === 'Escape') {
        setCapturedShortcut(null);
        setRecording(false);
        return;
      }
      if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;
      setCapturedShortcut(normalizeKeyEvent(e));
      setRecording(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [recording]);

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 480, maxHeight: '80vh', overflow: 'auto' }} onClick={e => e.stopPropagation()}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 16px', borderBottom: '1px solid var(--border)' }}>
          <div style={{ fontSize: 13, fontWeight: 600 }}>Edit: {sensor.name}</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <div style={{ fontSize: 10, padding: '2px 8px', borderRadius: 4, background: 'rgba(34,197,94,.1)', color: 'var(--cyan)' }}>{form.source_type}</div>
            <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18, lineHeight: 1 }}>&times;</button>
          </div>
        </div>
        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
          {(['general', 'source'] as const).map(t => (<button key={t} style={tabStyle(tab === t)} onClick={() => setTab(t)}>{t}</button>))}
        </div>
        <div style={{ padding: 16 }}>
          {tab === 'general' && (<div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
            {(['name', 'key', 'unit'] as const).map(f => (<label key={f} style={labelStyle}><span style={labelTextStyle}>{f}</span><input value={form[f]} onChange={e => setForm({ ...form, [f]: e.target.value })} style={inputStyle} /></label>))}
            {(['min_val', 'max_val', 'warn_val', 'crit_val', 'base_val', 'decimals'] as const).map(f => (<label key={f} style={labelStyle}><span style={labelTextStyle}>{f.replace('_', ' ')}</span><input type="number" step="any" value={form[f]} onChange={e => setForm({ ...form, [f]: parseFloat(e.target.value) || 0 })} style={monoInputStyle} /></label>))}
            <label style={labelStyle}><span style={labelTextStyle}>color</span><input type="color" value={form.color} onChange={e => setForm({ ...form, color: e.target.value })} style={{ height: 32, padding: 2, background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6 }} /></label>
            <div style={{ gridColumn: '1 / -1', display: 'flex', alignItems: 'center', gap: 8, marginTop: 4 }}>
              <span style={{ ...labelTextStyle, minWidth: 55 }}>Shortcut</span>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, flex: 1 }}>
                {capturedShortcut ? (
                  <kbd style={{
                    fontFamily: "'JetBrains Mono',monospace", fontSize: 12, fontWeight: 600,
                    padding: '4px 8px', borderRadius: 4,
                    background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--cyan)',
                  }}>{capturedShortcut}</kbd>
                ) : (
                  <span style={{ fontSize: 12, color: 'var(--muted)' }}>None</span>
                )}
                <button
                  onClick={() => setRecording(true)}
                  style={{
                    padding: '4px 10px', borderRadius: 4, fontSize: 11, cursor: 'pointer',
                    border: recording ? '1px solid var(--cyan)' : '1px solid var(--border)',
                    background: recording ? 'rgba(34,197,94,.1)' : 'var(--card)',
                    color: recording ? 'var(--cyan)' : 'var(--fg)',
                  }}
                >{recording ? 'Press a key...' : 'Record'}</button>
                {capturedShortcut && !recording && (
                  <button
                    onClick={() => setCapturedShortcut(null)}
                    style={{
                      padding: '4px 8px', borderRadius: 4, fontSize: 11, cursor: 'pointer',
                      border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)',
                    }}
                  >Clear</button>
                )}
              </div>
            </div>
          </div>)}
          {tab === 'source' && (<div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <label style={labelStyle}><span style={labelTextStyle}>Source Type</span><select value={form.source_type} onChange={e => { setForm({ ...form, source_type: e.target.value }); setSrcConfig({}); }} style={inputStyle}>{SOURCE_TYPES.map(t => <option key={t} value={t}>{t}</option>)}</select></label>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginTop: 4 }}><SourceConfigFields sourceType={form.source_type} config={srcConfig} onChange={setSrcConfig} /></div>
          </div>)}
        </div>
        <div style={{ display: 'flex', gap: 8, padding: '0 16px 16px' }}>
          <button onClick={() => { onSave({ ...form, source_config: JSON.stringify(srcConfig) }, capturedShortcut); }} style={{ padding: '6px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13 }}>Save</button>
          <button onClick={() => onDelete(sensor.id)} style={{ padding: '6px 16px', borderRadius: 6, border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 13 }}>Delete</button>
        </div>
      </div>
    </div>
  );
}
