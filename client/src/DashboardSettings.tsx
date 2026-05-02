import { inputStyle, monoInputStyle, labelStyle, labelTextStyle, btnStyle } from './styles';
import { useAlarmCooldown, useUpdateAlarmCooldown } from './queries';

const aggOptions = [
  { value: 'sampling', label: 'Sampling', desc: 'Pick evenly-spaced points. Fastest, but skips data between intervals.' },
  { value: 'avg', label: 'Average', desc: 'Average all 1s values within each interval bucket. Smooth curve, hides peaks.' },
  { value: 'minmax', label: 'Min/Max/Avg', desc: 'Shows average line with min/max band. Best overview — no data is lost.' },
] as const;

export type HistoryAutoAgg = typeof aggOptions[number]['value'];

export function DashboardSettings({ settings, onChange, onClose, alarmMuted, onToggleMute }: { settings: { gap: number; refreshInterval: number; historyAutoAgg: HistoryAutoAgg }; onChange: (s: any) => void; onClose: () => void; alarmMuted: boolean; onToggleMute: () => void }) {
  const { data: cooldownSecs = 300 } = useAlarmCooldown()
  const updateCooldown = useUpdateAlarmCooldown()

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 12, padding: 20, minWidth: 340, maxWidth: 440 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Dashboard Settings</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <label style={labelStyle}><span style={labelTextStyle}>Grid Gap (px)</span><input type="range" min="4" max="24" value={settings.gap} onChange={e => onChange({ ...settings, gap: parseInt(e.target.value) })} style={{ width: '100%' }} /><span style={{ fontSize: 11, fontFamily: 'JetBrains Mono,monospace' }}>{settings.gap}px</span></label>
          <label style={labelStyle}><span style={labelTextStyle}>Refresh Interval (ms)</span><input type="number" step="100" min="100" value={settings.refreshInterval} onChange={e => onChange({ ...settings, refreshInterval: parseInt(e.target.value) || 1000 })} style={monoInputStyle} /></label>
          <div style={labelStyle}>
            <span style={labelTextStyle}>History Auto Mode</span>
            <div style={{ display: 'flex', gap: 4, marginTop: 4 }}>
              {aggOptions.map(opt => (
                <button key={opt.value} onClick={() => onChange({ ...settings, historyAutoAgg: opt.value })} style={{
                  ...btnStyle, flex: 1, padding: '8px 6px', fontSize: 11, fontWeight: settings.historyAutoAgg === opt.value ? 600 : 400,
                  borderColor: settings.historyAutoAgg === opt.value ? 'var(--warn)' : 'var(--border)',
                  color: settings.historyAutoAgg === opt.value ? 'var(--warn)' : 'var(--fg)',
                  background: settings.historyAutoAgg === opt.value ? 'rgba(234,179,8,.08)' : 'transparent',
                  minHeight: 44,
                }}>
                  {opt.label}
                </button>
              ))}
            </div>
            <div style={{ fontSize: 11, color: 'var(--fg-muted)', marginTop: 6, lineHeight: 1.4 }}>
              {aggOptions.find(o => o.value === settings.historyAutoAgg)?.desc}
            </div>
          </div>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Alarm Cooldown (seconds)</span>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 4 }}>
              <input type="number" step="30" min="0" max="86400" value={cooldownSecs} onChange={e => {
                const val = Math.max(0, Math.min(86400, parseInt(e.target.value) || 0))
                updateCooldown.mutate(val)
              }} style={monoInputStyle} />
              <span style={{ fontSize: 11, color: 'var(--fg-muted)' }}>0 = disabled</span>
            </div>
            <div style={{ fontSize: 11, color: 'var(--fg-muted)', marginTop: 4, lineHeight: 1.4 }}>
              After acknowledging an alarm, wait this long before triggering a new alarm for the same sensor if the value is still above threshold.
            </div>
          </label>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Mute Alarms</span>
            <button onClick={onToggleMute} style={{
              padding: '8px 14px', borderRadius: 6, border: `1px solid ${alarmMuted ? 'var(--warn)' : 'var(--border)'}`,
              background: alarmMuted ? 'rgba(234,179,8,.1)' : 'transparent', color: alarmMuted ? 'var(--warn)' : 'var(--fg)',
              cursor: 'pointer', fontSize: 13, fontWeight: 500, display: 'flex', alignItems: 'center', gap: 6,
              minHeight: 44,
            }}>
              {alarmMuted ? (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/><line x1="1" y1="1" x2="23" y2="23" stroke="currentColor" strokeWidth="2"/></svg>
              ) : (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              )}
              {alarmMuted ? 'Muted' : 'Active'}
            </button>
          </label>
        </div>
        <button onClick={onClose} style={{ marginTop: 16, padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13, width: '100%', minHeight: 44 }}>Close</button>
      </div>
    </div>
  );
}
