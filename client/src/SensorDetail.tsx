import { useEffect } from 'react';
import { SensorConfig, SensorData, DataPoint } from './types';
import { status, stColor, btnStyle, safeMinMax } from './styles';
import { DetailChart } from './DetailChart';
import { HistoryRangeBar } from './CustomRangePicker';

export function SensorDetail({ sensor, data, currentValue, onBack, historyRange, onRangeChange, shortcuts }: {
  sensor: SensorConfig; data: DataPoint[]; currentValue: SensorData | undefined;
  onBack: () => void; historyRange: string; onRangeChange: (r: string) => void;
  shortcuts: Record<number, string>;
}) {
  const isOffline = currentValue && (currentValue.value == null || Number.isNaN(currentValue.value));
  const st = isOffline ? 'offline' : currentValue ? status(currentValue.value, currentValue.warn, currentValue.crit) : 'normal';
  const safeValues = data.map(d => d.value).filter(v => v != null && !Number.isNaN(v));
  const [minV, maxV] = safeMinMax(safeValues);
  const avgV = safeValues.length ? safeValues.reduce((a, b) => a + b, 0) / safeValues.length : 0;
  const curV = isOffline ? null : currentValue ? currentValue.value : sensor.base_val;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onBack(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onBack]);

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 16 }}>
      <div style={{ maxWidth: 1600, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%', gap: 12 }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexShrink: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <button onClick={onBack} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
            </button>
            <h1 style={{ fontSize: 20, fontWeight: 700 }}>{sensor.name}</h1>
            {shortcuts[sensor.id] && (
              <kbd style={{
                fontFamily: "'JetBrains Mono',monospace", fontSize: 11, fontWeight: 600,
                padding: '2px 8px', borderRadius: 4,
                background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)',
              }}>{shortcuts[sensor.id]}</kbd>
            )}
            <span style={{ fontSize: 12, color: 'var(--muted)' }}>({sensor.key})</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <HistoryRangeBar range={historyRange} onChange={onRangeChange} />
            <a href={`/api/export/csv?sensor=${sensor.key}&range=${historyRange}`} download style={{ ...btnStyle, textDecoration: 'none' }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>Export CSV
            </a>
            <div style={{ padding: '4px 12px', borderRadius: 6, fontWeight: 600, fontSize: 12, textTransform: 'uppercase', background: st === 'critical' ? 'rgba(239,68,68,.15)' : st === 'warning' ? 'rgba(234,179,8,.15)' : st === 'offline' ? 'rgba(100,100,100,.15)' : 'rgba(34,197,94,.15)', color: stColor(st) }}>{st}</div>
            <span style={{ color: 'var(--muted)', fontSize: 12 }}>Source: {sensor.source_type}</span>
          </div>
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'auto repeat(4, 1fr) auto', gap: 8, flexShrink: 0 }}>
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, padding: '10px 16px', display: 'flex', alignItems: 'center', gap: 8 }}>
            <div style={{ fontSize: 12, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>Current</div>
            <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 22, fontWeight: 700, color: st === 'normal' ? sensor.color : stColor(st) }}>
              {curV != null ? curV.toFixed(sensor.decimals) : 'OFFLINE'}<span style={{ fontSize: 13, color: 'var(--muted)', fontWeight: 400, marginLeft: 4 }}>{sensor.unit}</span>
            </div>
          </div>
          {[{ l: 'Min', v: safeValues.length ? minV.toFixed(sensor.decimals) : '—' }, { l: 'Max', v: safeValues.length ? maxV.toFixed(sensor.decimals) : '—' }, { l: 'Avg', v: safeValues.length ? avgV.toFixed(sensor.decimals) : '—' }, { l: 'Points', v: String(data.length) }].map((s, i) => (
            <div key={i} style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, padding: '10px 14px' }}>
              <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.05em' }}>{s.l}</div>
              <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 16, fontWeight: 700, marginTop: 2 }}>{s.v}</div>
            </div>
          ))}
          <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, padding: '10px 14px' }}>
            {[{ l: 'Warn', v: sensor.warn_val, c: 'var(--warn)' }, { l: 'Crit', v: sensor.crit_val, c: 'var(--crit)' }, { l: 'Range', v: `${sensor.min_val}–${sensor.max_val}`, c: 'var(--fg)' }].map((t, i) => (
              <div key={i} style={{ display: 'flex', justifyContent: 'space-between', gap: 12, fontSize: 12 }}>
                <span style={{ color: 'var(--muted)' }}>{t.l}</span>
                <span style={{ fontFamily: "'JetBrains Mono',monospace", color: t.c }}>{t.v}{sensor.unit && i < 2 ? sensor.unit : ''}</span>
              </div>
            ))}
          </div>
        </div>
        <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 12, flex: 1, minHeight: 0, overflow: 'hidden' }}>
          <DetailChart data={data} color={sensor.color} warn={sensor.warn_val} crit={sensor.crit_val} unit={sensor.unit} />
        </div>
      </div>
    </div>
  );
}
