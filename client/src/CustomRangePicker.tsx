import { useState } from 'react';
import { inputStyle, labelStyle, labelTextStyle, btnStyle } from './styles';

function toLocalDatetime(ts: number) {
  const d = new Date(ts);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function CustomRangePicker({ onApply, onClose }: { onApply: (range: string) => void; onClose: () => void }) {
  const now = Date.now();
  const [start, setStart] = useState(toLocalDatetime(now - 30 * 60 * 1000));
  const [end, setEnd] = useState(toLocalDatetime(now));

  const apply = () => {
    const startMs = new Date(start).getTime();
    const endMs = new Date(end).getTime();
    if (isNaN(startMs) || isNaN(endMs) || startMs >= endMs) return;
    onApply(`${startMs}-${endMs}`);
    onClose();
  };

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 360, padding: 20 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Custom Time Range</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Start</span>
            <input type="datetime-local" value={start} onChange={e => setStart(e.target.value)} style={inputStyle} />
          </label>
          <label style={labelStyle}>
            <span style={labelTextStyle}>End</span>
            <input type="datetime-local" value={end} onChange={e => setEnd(e.target.value)} style={inputStyle} />
          </label>
        </div>
        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
          <button onClick={onClose} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Cancel</button>
          <button onClick={apply} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--warn)', background: 'transparent', color: 'var(--warn)', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>Apply</button>
        </div>
      </div>
    </div>
  );
}

function formatCustomRange(range: string) {
  const [s, e] = range.split('-').map(Number);
  if (isNaN(s) || isNaN(e)) return range;
  const fmt = (ts: number) => new Date(ts).toLocaleString('en-US', { hour12: false, month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  return `${fmt(s)} – ${fmt(e)}`;
}

export function HistoryRangeBar({ range, onChange }: { range: string; onChange: (r: string) => void }) {
  const [showCustom, setShowCustom] = useState(false);
  const presets = ['30m', '1h', '6h', '24h'] as const;
  const isCustom = !presets.includes(range as any);

  return (
    <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
      {presets.map(r => (
        <button key={r} style={{ ...btnStyle, padding: '6px 10px', fontSize: 12, borderColor: range === r ? 'var(--warn)' : 'var(--border)', color: range === r ? 'var(--warn)' : 'var(--fg)' }} onClick={() => onChange(r)}>{r}</button>
      ))}
      <button style={{ ...btnStyle, padding: '6px 10px', fontSize: 12, borderColor: isCustom ? 'var(--warn)' : 'var(--border)', color: isCustom ? 'var(--warn)' : 'var(--fg)' }} onClick={() => setShowCustom(true)}>
        Custom
      </button>
      {isCustom && (
        <span style={{ fontSize: 11, color: 'var(--warn)', fontFamily: "'JetBrains Mono',monospace", whiteSpace: 'nowrap' }}>{formatCustomRange(range)}</span>
      )}
      {showCustom && <CustomRangePicker onApply={onChange} onClose={() => setShowCustom(false)} />}
    </div>
  );
}
