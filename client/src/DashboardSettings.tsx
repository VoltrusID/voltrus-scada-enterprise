import { inputStyle, monoInputStyle, labelStyle, labelTextStyle } from './styles';

export function DashboardSettings({ settings, onChange, onClose }: { settings: { gap: number; refreshInterval: number }; onChange: (s: any) => void; onClose: () => void }) {
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 12, padding: 20, minWidth: 340, maxWidth: 400 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Dashboard Settings</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <label style={labelStyle}><span style={labelTextStyle}>Grid Gap (px)</span><input type="range" min="4" max="24" value={settings.gap} onChange={e => onChange({ ...settings, gap: parseInt(e.target.value) })} style={{ width: '100%' }} /><span style={{ fontSize: 11, fontFamily: 'JetBrains Mono,monospace' }}>{settings.gap}px</span></label>
          <label style={labelStyle}><span style={labelTextStyle}>Refresh Interval (ms)</span><input type="number" step="100" min="100" value={settings.refreshInterval} onChange={e => onChange({ ...settings, refreshInterval: parseInt(e.target.value) || 1000 })} style={monoInputStyle} /></label>
        </div>
        <button onClick={onClose} style={{ marginTop: 16, padding: '6px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13, width: '100%' }}>Close</button>
      </div>
    </div>
  );
}
