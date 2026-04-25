import { inputStyle, monoInputStyle, labelStyle, labelTextStyle } from './styles';

export function SourceConfigFields({ sourceType, config, onChange }: { sourceType: string; config: Record<string, any>; onChange: (c: Record<string, any>) => void }) {
  const set = (k: string, v: any) => onChange({ ...config, [k]: v });
  switch (sourceType) {
    case 'random_walk': return (<>
      <label style={labelStyle}><span style={labelTextStyle}>Volatility</span><input type="range" min="0.01" max="0.5" step="0.01" value={config.volatility ?? 0.06} onChange={e => set('volatility', parseFloat(e.target.value))} style={{ width: '100%' }} /><span style={{ fontSize: 11, color: 'var(--fg)', fontFamily: 'JetBrains Mono,monospace' }}>{(config.volatility ?? 0.06).toFixed(2)}</span></label>
      <label style={labelStyle}><span style={labelTextStyle}>Reversion</span><input type="range" min="0.01" max="0.2" step="0.01" value={config.reversion ?? 0.03} onChange={e => set('reversion', parseFloat(e.target.value))} style={{ width: '100%' }} /><span style={{ fontSize: 11, color: 'var(--fg)', fontFamily: 'JetBrains Mono,monospace' }}>{(config.reversion ?? 0.03).toFixed(2)}</span></label></>);
    case 'sine': return (<>
      <label style={labelStyle}><span style={labelTextStyle}>Amplitude</span><input type="number" step="any" value={config.amplitude ?? 5} onChange={e => set('amplitude', parseFloat(e.target.value))} style={monoInputStyle} /></label>
      <label style={labelStyle}><span style={labelTextStyle}>Frequency (Hz)</span><input type="number" step="any" value={config.frequency ?? 0.5} onChange={e => set('frequency', parseFloat(e.target.value))} style={monoInputStyle} /></label>
      <label style={labelStyle}><span style={labelTextStyle}>Phase Offset</span><input type="number" step="any" value={config.offset ?? 0} onChange={e => set('offset', parseFloat(e.target.value))} style={monoInputStyle} /></label></>);
    case 'step': return (<>
      <label style={labelStyle}><span style={labelTextStyle}>Jump Probability</span><input type="number" step="0.01" min="0" max="1" value={config.jump_probability ?? 0.02} onChange={e => set('jump_probability', parseFloat(e.target.value))} style={monoInputStyle} /></label>
      <label style={labelStyle}><span style={labelTextStyle}>Settle Time (s)</span><input type="number" step="1" min="1" value={config.settle_time ?? 5} onChange={e => set('settle_time', parseFloat(e.target.value))} style={monoInputStyle} /></label></>);
    case 'sawtooth': return <label style={labelStyle}><span style={labelTextStyle}>Period (s)</span><input type="number" step="any" min="1" value={config.period ?? 10} onChange={e => set('period', parseFloat(e.target.value))} style={monoInputStyle} /></label>;
    case 'constant': return <div style={{ color: 'var(--muted)', fontSize: 12, padding: 8 }}>Always returns base value.</div>;
    case 'none': return <div style={{ color: 'var(--muted)', fontSize: 12, padding: 8 }}>No data source. Use for image cards or static displays.</div>;
    case 'noise': return <label style={labelStyle}><span style={labelTextStyle}>Noise Amplitude</span><input type="number" step="any" value={config.noise_amplitude ?? 3} onChange={e => set('noise_amplitude', parseFloat(e.target.value))} style={monoInputStyle} /></label>;
    case 'api': return (<>
      <label style={labelStyle}><span style={labelTextStyle}>URL</span><input type="text" value={config.url ?? ''} onChange={e => set('url', e.target.value)} style={inputStyle} placeholder="https://example.com/api/sensor" /></label>
      <label style={labelStyle}><span style={labelTextStyle}>JSON Path</span><input type="text" value={config.path ?? ''} onChange={e => set('path', e.target.value)} style={inputStyle} placeholder="data.value" /></label>
      <label style={labelStyle}><span style={labelTextStyle}>Method</span><select value={config.method ?? 'GET'} onChange={e => set('method', e.target.value)} style={inputStyle}><option value="GET">GET</option><option value="POST">POST</option></select></label>
      <label style={labelStyle}><span style={labelTextStyle}>Poll Interval (ms)</span><input type="number" step="1000" min="1000" value={config.poll_interval ?? 5000} onChange={e => set('poll_interval', parseInt(e.target.value))} style={monoInputStyle} /></label>
      <label style={labelStyle}><span style={labelTextStyle}>Headers (JSON)</span><textarea value={typeof config.headers === 'object' ? JSON.stringify(config.headers, null, 2) : config.headers ?? '{}'} onChange={e => { try { set('headers', JSON.parse(e.target.value)) } catch { set('headers', e.target.value) } }} style={{ ...inputStyle, minHeight: 60, fontFamily: 'JetBrains Mono,monospace', fontSize: 11 }} /></label></>);
    default: return null;
  }
}
