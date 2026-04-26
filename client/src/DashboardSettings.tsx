import { inputStyle, monoInputStyle, labelStyle, labelTextStyle } from './styles';

export function DashboardSettings({ settings, onChange, onClose, alarmMuted, onToggleMute }: { settings: { gap: number; refreshInterval: number }; onChange: (s: any) => void; onClose: () => void; alarmMuted: boolean; onToggleMute: () => void }) {
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 12, padding: 20, minWidth: 340, maxWidth: 400 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Dashboard Settings</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <label style={labelStyle}><span style={labelTextStyle}>Grid Gap (px)</span><input type="range" min="4" max="24" value={settings.gap} onChange={e => onChange({ ...settings, gap: parseInt(e.target.value) })} style={{ width: '100%' }} /><span style={{ fontSize: 11, fontFamily: 'JetBrains Mono,monospace' }}>{settings.gap}px</span></label>
          <label style={labelStyle}><span style={labelTextStyle}>Refresh Interval (ms)</span><input type="number" step="100" min="100" value={settings.refreshInterval} onChange={e => onChange({ ...settings, refreshInterval: parseInt(e.target.value) || 1000 })} style={monoInputStyle} /></label>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Mute Alarms</span>
            <button onClick={onToggleMute} style={{
              padding: '6px 14px', borderRadius: 6, border: `1px solid ${alarmMuted ? 'var(--warn)' : 'var(--border)'}`,
              background: alarmMuted ? 'rgba(234,179,8,.1)' : 'transparent', color: alarmMuted ? 'var(--warn)' : 'var(--fg)',
              cursor: 'pointer', fontSize: 13, fontWeight: 500, display: 'flex', alignItems: 'center', gap: 6,
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
        <button onClick={onClose} style={{ marginTop: 16, padding: '6px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13, width: '100%' }}>Close</button>
      </div>
    </div>
  );
}
