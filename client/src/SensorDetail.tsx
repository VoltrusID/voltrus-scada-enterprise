import { useEffect, useState, useRef, useMemo } from 'react';
import { SensorConfig } from './types';
import { stColor, btnStyle, safeMinMax, displayShortcut } from './styles';
import { TvDetailChart, OverlaySeries } from './TvDetailChart';
import { HistoryRangeBar } from './CustomRangePicker';
import { useDetailData, useSensorCurrentValue, useDashboardSelector } from './store/dashboardStore';

export function SensorDetail({ sensor, onBack, historyRange, onRangeChange, historyDetail, onDetailChange, historyAutoAgg, shortcuts, mode, onModeChange, allSensors }: {
  sensor: SensorConfig; onBack: () => void; historyRange: string; onRangeChange: (r: string) => void;
  historyDetail?: 'auto' | 'full'; onDetailChange?: (d: 'auto' | 'full') => void;
  historyAutoAgg?: string;
  shortcuts: Record<number, string>;
  mode: 'live' | 'history'; onModeChange: (m: 'live' | 'history') => void;
  allSensors?: SensorConfig[];
}) {
  const data = useDetailData(sensor.id, mode)
  const currentValue = useSensorCurrentValue(sensor.id)
  const allCurrentValues = useDashboardSelector(s => s.currentValues)

  const st = currentValue?.status ?? 'normal';
  const isOffline = st === 'offline';
  const curV = isOffline ? null : currentValue ? currentValue.value : sensor.base_val;

  const [overlayIds, setOverlayIds] = useState<number[]>([]);
  const [showPicker, setShowPicker] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);
  const [pointWidth, setPointWidth] = useState(0); // 0 = auto/fit

  const DENSITY_PRESETS = [
    { label: 'Fit', value: 0 },
    { label: '2px', value: 2 },
    { label: '4px', value: 4 },
    { label: '8px', value: 8 },
    { label: '16px', value: 16 },
  ];

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onBack(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onBack]);

  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      if (showPicker && pickerRef.current && !pickerRef.current.contains(e.target as Node)) setShowPicker(false);
    };
    document.addEventListener('mousedown', onClick);
    return () => document.removeEventListener('mousedown', onClick);
  }, [showPicker]);

  // Memoize stats computations
  const stats = useMemo(() => {
    const safeValues = data.map(d => d.value).filter(v => v != null && !Number.isNaN(v));
    const [minV, maxV] = safeMinMax(safeValues);
    const avgV = safeValues.length ? safeValues.reduce((a, b) => a + b, 0) / safeValues.length : 0;
    return { safeValues, minV, maxV, avgV, pointCount: data.length };
  }, [data]);

  // Memoize overlays
  const overlays: OverlaySeries[] = useMemo(() => {
    if (!allSensors || overlayIds.length === 0) return [];
    return overlayIds.map(id => {
      const s = allSensors.find(x => x.id === id);
      if (!s) return null;
      // For overlays, we fetch their data from store in live mode
      // History mode overlays would need additional fetching; keeping simple for now
      return { data: [], color: s.color, name: s.name, unit: s.unit };
    }).filter(Boolean) as OverlaySeries[];
  }, [allSensors, overlayIds]);

  const sparklineSensors = useMemo(() =>
    (allSensors ?? []).filter(s => s.chart_type === 'sparkline' && s.id !== sensor.id && !overlayIds.includes(s.id)),
    [allSensors, sensor.id, overlayIds]
  );

  const toggleOverlay = (id: number) => {
    setOverlayIds(prev => prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]);
    setShowPicker(false);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12, height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexShrink: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <button onClick={onBack} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back<kbd style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 9, padding: '1px 4px', marginLeft: 2, borderRadius: 3, background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)' }}>Esc</kbd>
            </button>
            <h1 style={{ fontSize: 20, fontWeight: 700 }}>{sensor.name}</h1>
            {shortcuts[sensor.id] && (
              <kbd style={{
                fontFamily: "'JetBrains Mono',monospace", fontSize: 11, fontWeight: 600,
                padding: '2px 8px', borderRadius: 4,
                background: 'var(--bg)', border: '1px solid var(--border)', color: 'var(--muted)',
              }}>{displayShortcut(shortcuts[sensor.id])}</kbd>
            )}
            <span style={{ fontSize: 12, color: 'var(--muted)' }}>({sensor.key})</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <div style={{ display: 'flex', border: '1px solid var(--border)', borderRadius: 6, overflow: 'hidden' }}>
              <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'live' ? 'var(--cyan)' : 'var(--fg)', background: mode === 'live' ? 'rgba(34,197,94,.1)' : 'var(--card)' }} onClick={() => onModeChange('live')}>Live</button>
              <button style={{ ...btnStyle, border: 'none', borderRadius: 0, color: mode === 'history' ? 'var(--warn)' : 'var(--fg)', background: mode === 'history' ? 'rgba(234,179,8,.1)' : 'var(--card)' }} onClick={() => onModeChange('history')}>History</button>
            </div>
            {mode === 'history' && <HistoryRangeBar range={historyRange} onChange={onRangeChange} detail={historyDetail} onDetailChange={onDetailChange} agg={historyAutoAgg} />}
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
          {[{ l: 'Min', v: stats.safeValues.length ? stats.minV.toFixed(sensor.decimals) : '—' }, { l: 'Max', v: stats.safeValues.length ? stats.maxV.toFixed(sensor.decimals) : '—' }, { l: 'Avg', v: stats.safeValues.length ? stats.avgV.toFixed(sensor.decimals) : '—' }, { l: 'Points', v: String(stats.pointCount) }].map((s, i) => (
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
        {/* Chart area with overlay controls */}
        <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, padding: 12, flex: 1, minHeight: 0, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          {/* Overlay bar */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 8, flexShrink: 0, flexWrap: 'wrap' }}>
            <span style={{ fontSize: 11, color: 'var(--muted)', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '.05em' }}>Density:</span>
            {DENSITY_PRESETS.map(p => (
              <button key={p.value} onClick={() => setPointWidth(p.value)} style={{ padding: '2px 8px', borderRadius: 4, fontSize: 10, fontWeight: 600, cursor: 'pointer', border: `1px solid ${pointWidth === p.value ? 'var(--cyan)' : 'var(--border)'}`, background: pointWidth === p.value ? 'rgba(34,197,94,.1)' : 'var(--card)', color: pointWidth === p.value ? 'var(--cyan)' : 'var(--muted)', fontFamily: "'JetBrains Mono',monospace" }}>{p.label}</button>
            ))}
            {pointWidth > 0 && <span style={{ fontSize: 10, color: 'var(--muted)' }}>~{pointWidth * data.length}px wide · scroll to explore</span>}
            <div style={{ width: 1, height: 16, background: 'var(--border)', margin: '0 4px' }} />
            <span style={{ fontSize: 11, color: 'var(--muted)', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '.05em' }}>Overlays:</span>
            {/* Active overlay chips */}
            {overlayIds.map(id => {
              const s = (allSensors ?? []).find(x => x.id === id);
              if (!s) return null;
              return (
                <div key={id} style={{ display: 'flex', alignItems: 'center', gap: 4, padding: '2px 8px', borderRadius: 4, background: s.color + '20', border: `1px solid ${s.color}40`, fontSize: 11, fontFamily: "'JetBrains Mono',monospace", color: s.color, cursor: 'pointer' }} onClick={() => setOverlayIds(prev => prev.filter(x => x !== id))}>
                  <div style={{ width: 8, height: 2, background: s.color, borderRadius: 1 }} />
                  {s.name}
                  <span style={{ opacity: 0.6, marginLeft: 2 }}>x</span>
                </div>
              );
            })}
            {/* Add series button */}
            <div ref={pickerRef} style={{ position: 'relative' }}>
              <button onClick={() => setShowPicker(!showPicker)} style={{ ...btnStyle, fontSize: 11, padding: '3px 10px', borderColor: sparklineSensors.length ? 'var(--cyan)' : 'var(--border)', color: sparklineSensors.length ? 'var(--cyan)' : 'var(--muted)', opacity: sparklineSensors.length ? 1 : 0.5 }}>
                + Add Series
              </button>
              {showPicker && sparklineSensors.length > 0 && (
                <div style={{ position: 'absolute', top: '100%', left: 0, marginTop: 4, background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, padding: 6, minWidth: 220, maxHeight: 240, overflowY: 'auto', zIndex: 10, boxShadow: '0 8px 24px rgba(0,0,0,.4)' }}>
                  <div style={{ fontSize: 10, color: 'var(--muted)', padding: '2px 6px 6px', textTransform: 'uppercase', letterSpacing: '.05em' }}>Sparkline sensors</div>
                  {sparklineSensors.map(s => {
                    const cv = allCurrentValues[String(s.id)]
                    const curVal = cv ? cv.value : null
                    return (
                      <button key={s.id} onClick={() => toggleOverlay(s.id)} style={{ display: 'flex', alignItems: 'center', gap: 8, width: '100%', padding: '6px 8px', border: 'none', background: 'none', borderRadius: 4, cursor: 'pointer', color: 'var(--fg)', fontSize: 12, textAlign: 'left' }}
                        onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,.05)')}
                        onMouseLeave={e => (e.currentTarget.style.background = 'none')}>
                        <div style={{ width: 10, height: 10, borderRadius: 2, background: s.color, flexShrink: 0 }} />
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div style={{ fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{s.name}</div>
                          <div style={{ fontSize: 10, color: 'var(--muted)' }}>{s.key} · {s.unit}</div>
                        </div>
                        {curVal != null && <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: s.color, fontWeight: 600 }}>{curVal.toFixed(s.decimals)}</div>}
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
          <div style={{ flex: 1, minHeight: 0 }}>
            <TvDetailChart data={data} color={sensor.color} warn={sensor.warn_val} crit={sensor.crit_val} unit={sensor.unit} overlays={overlays} pointWidth={pointWidth} />
          </div>
        </div>
    </div>
  );
}
