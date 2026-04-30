import { useState, useEffect, useRef, useCallback } from 'react';
import { SensorConfig, SOURCE_TYPES, CHART_TYPES, SOURCE_CATEGORY } from './types';
import { inputStyle, monoInputStyle, btnStyle, parseConfig } from './styles';
import { SourceConfigFields } from './SourceConfigFields';
import { useUpdateSensor } from './queries';

const CELL: React.CSSProperties = {
  background: 'transparent',
  border: '1px solid transparent',
  color: 'var(--fg)',
  fontSize: 12,
  padding: '4px 6px',
  width: '100%',
  boxSizing: 'border-box',
  outline: 'none',
  borderRadius: 0,
  fontFamily: 'inherit',
};
const CELL_MONO: React.CSSProperties = { ...CELL, fontFamily: "'JetBrains Mono',monospace" };
const CELL_NUM: React.CSSProperties = { ...CELL_MONO, textAlign: 'right' };

const COL_WIDTHS = {
  color: 40, name: 140, key: 120, unit: 60,
  min_val: 64, max_val: 64, warn_val: 64, crit_val: 64, base_val: 64,
  decimals: 40, chart_type: 90, sort_order: 40,
  source_type: 90, source_config: 60, actions: 90,
} as const;

const thStyle: React.CSSProperties = {
  textAlign: 'left', padding: '6px 6px', fontSize: 10, color: 'var(--muted)',
  textTransform: 'uppercase', letterSpacing: '.05em', fontWeight: 600,
  borderBottom: '1px solid var(--border)', whiteSpace: 'nowrap', position: 'sticky', top: 0,
  background: 'var(--card)', zIndex: 1,
};
const tdStyle: React.CSSProperties = {
  padding: '0', fontSize: 12, borderBottom: '1px solid var(--border)',
};

export function SensorManager({ sensors, onSave, onDelete, onAdd, onDuplicate, onClose, shortcuts, embedded }: {
  sensors: SensorConfig[]; onSave: (s: SensorConfig, shortcut?: string | null) => void; onDelete: (id: number) => void;
  onAdd: (s: Partial<SensorConfig>) => void; onDuplicate: (s: SensorConfig) => void; onClose: () => void; shortcuts: Record<number, string>;
  embedded?: boolean;
}) {
  const [filter, setFilter] = useState('');
  const [newRow, setNewRow] = useState<Partial<SensorConfig> & { source_config_obj?: Record<string, any> } | null>(null);
  const [configPopover, setConfigPopover] = useState<{ id: number; srcType: string; config: Record<string, any>; rect: DOMRect } | null>(null);
  const updateMut = useUpdateSensor();
  const configRef = useRef<HTMLDivElement>(null);

  const filtered = sensors.filter(s => s.name.toLowerCase().includes(filter) || s.key.toLowerCase().includes(filter));

  useEffect(() => {
    if (embedded) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose, embedded]);

  // Close config popover on outside click
  useEffect(() => {
    if (!configPopover) return;
    const handler = (e: MouseEvent) => {
      if (configRef.current && !configRef.current.contains(e.target as Node)) {
        setConfigPopover(null);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [configPopover]);

  const updateField = useCallback((sensor: SensorConfig, field: string, value: any) => {
    onSave({ ...sensor, [field]: value });
  }, [onSave]);

  const startConfigEdit = (sensor: SensorConfig, btn: HTMLButtonElement) => {
    const rect = btn.getBoundingClientRect();
    setConfigPopover({
      id: sensor.id,
      srcType: sensor.source_type,
      config: { ...parseConfig(sensor.source_config), ...(sensor.source_type === 'modbus' && sensor.device_id ? { modbus_device_id: sensor.device_id } : {}) },
      rect,
    });
  };

  const saveConfig = () => {
    if (!configPopover) return;
    const sensor = sensors.find(s => s.id === configPopover.id);
    if (!sensor) return;
    const { modbus_device_id, ...rest } = configPopover.config;
    const update: any = { ...sensor, source_config: JSON.stringify(rest) };
    if (sensor.source_type === 'modbus' && modbus_device_id) update.device_id = modbus_device_id;
    onSave(update);
    setConfigPopover(null);
  };

  const handleAdd = () => {
    if (newRow) return;
    setNewRow({
      key: '', name: '', unit: '', color: '#8b5cf6',
      min_val: 0, max_val: 100, warn_val: 80, crit_val: 90, base_val: 50,
      decimals: 1, sort_order: 0, source_type: 'random_walk', chart_type: 'sparkline',
      source_config: '{}', source_config_obj: {},
    });
  };

  const commitNewRow = () => {
    if (!newRow) return;
    const { source_config_obj, ...rest } = newRow;
    onAdd({ ...rest, source_config: JSON.stringify(source_config_obj ?? {}) });
    setNewRow(null);
  };

  const updateNewField = (field: string, value: any) => {
    if (!newRow) return;
    setNewRow({ ...newRow, [field]: value });
  };

  const numField = (sensor: SensorConfig, field: keyof SensorConfig) => (
    <input
      key={`${sensor.id}-${field}-${sensor[field]}`}
      type="number" step="any"
      defaultValue={sensor[field] as number}
      onBlur={e => {
        const v = parseFloat(e.target.value);
        if (!isNaN(v) && v !== sensor[field]) updateField(sensor, field, v);
      }}
      onKeyDown={e => {
        if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
      }}
      style={CELL_NUM}
    />
  );

  const renderRow = (s: SensorConfig) => (
    <tr key={s.id} style={{ opacity: updateMut.isPending && updateMut.variables?.id === s.id ? 0.6 : 1 }}>
      <td style={tdStyle}>
        <input type="color" value={s.color} onChange={e => updateField(s, 'color', e.target.value)} style={{ width: 28, height: 24, padding: 1, border: 'none', background: 'transparent', cursor: 'pointer' }} />
      </td>
      <td style={tdStyle}>
        <input key={`name-${s.id}-${s.name}`} defaultValue={s.name} onBlur={e => { if (e.target.value !== s.name) updateField(s, 'name', e.target.value); }} onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} style={CELL} />
      </td>
      <td style={tdStyle}>
        <input key={`key-${s.id}-${s.key}`} defaultValue={s.key} onBlur={e => { if (e.target.value !== s.key) updateField(s, 'key', e.target.value); }} onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} style={CELL_MONO} />
      </td>
      <td style={tdStyle}>
        <input key={`unit-${s.id}-${s.unit}`} defaultValue={s.unit} onBlur={e => { if (e.target.value !== s.unit) updateField(s, 'unit', e.target.value); }} onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} style={CELL} />
      </td>
      <td style={tdStyle}>{numField(s, 'min_val')}</td>
      <td style={tdStyle}>{numField(s, 'max_val')}</td>
      <td style={tdStyle}>{numField(s, 'warn_val')}</td>
      <td style={tdStyle}>{numField(s, 'crit_val')}</td>
      <td style={tdStyle}>{numField(s, 'base_val')}</td>
      <td style={tdStyle}>{numField(s, 'decimals')}</td>
      <td style={tdStyle}>
        <select value={s.chart_type || 'sparkline'} onChange={e => updateField(s, 'chart_type', e.target.value)} style={{ ...CELL, cursor: 'pointer' }}>
          {CHART_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
        </select>
      </td>
      <td style={tdStyle}>{numField(s, 'sort_order')}</td>
      <td style={tdStyle}>
        <select value={s.source_type} onChange={e => {
          const newType = e.target.value;
          onSave({ ...s, source_type: newType, source_config: '{}' });
        }} style={{ ...CELL, cursor: 'pointer', color: SOURCE_CATEGORY[s.source_type]?.color ?? 'var(--fg)', fontSize: 11 }}>
          <optgroup label="Real">
            <option value="modbus">modbus</option>
          </optgroup>
          <optgroup label="Simulate">
            {SOURCE_TYPES.filter(t => !['modbus','api','none'].includes(t)).map(t => <option key={t} value={t}>{t}</option>)}
          </optgroup>
          <optgroup label="Other">
            <option value="api">api</option>
            <option value="none">none</option>
          </optgroup>
        </select>
      </td>
      <td style={tdStyle}>
        <button onClick={e => startConfigEdit(s, e.currentTarget)} style={{ padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 10 }}>cfg</button>
      </td>
      <td style={{ ...tdStyle, whiteSpace: 'nowrap' }}>
        <div style={{ display: 'flex', gap: 3 }}>
          <button onClick={() => onDuplicate(s)} style={{ padding: '2px 6px', borderRadius: 3, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 10 }}>dup</button>
          <button onClick={() => { if (confirm(`Delete ${s.name}?`)) onDelete(s.id); }} style={{ padding: '2px 6px', borderRadius: 3, border: '1px solid var(--crit)', background: 'transparent', color: 'var(--crit)', cursor: 'pointer', fontSize: 10 }}>del</button>
        </div>
      </td>
    </tr>
  );

  const renderNewRow = () => {
    if (!newRow) return null;
    return (
      <tr style={{ background: 'rgba(139,92,246,.05)' }}>
        <td style={tdStyle}>
          <input type="color" value={newRow.color || '#8b5cf6'} onChange={e => updateNewField('color', e.target.value)} style={{ width: 28, height: 24, padding: 1, border: 'none', background: 'transparent', cursor: 'pointer' }} />
        </td>
        <td style={tdStyle}><input value={newRow.name ?? ''} onChange={e => updateNewField('name', e.target.value)} placeholder="Name" style={CELL} /></td>
        <td style={tdStyle}><input value={newRow.key ?? ''} onChange={e => updateNewField('key', e.target.value)} placeholder="key" style={CELL_MONO} /></td>
        <td style={tdStyle}><input value={newRow.unit ?? ''} onChange={e => updateNewField('unit', e.target.value)} placeholder="V" style={CELL} /></td>
        <td style={tdStyle}><input type="number" step="any" value={newRow.min_val ?? 0} onChange={e => updateNewField('min_val', parseFloat(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}><input type="number" step="any" value={newRow.max_val ?? 100} onChange={e => updateNewField('max_val', parseFloat(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}><input type="number" step="any" value={newRow.warn_val ?? 80} onChange={e => updateNewField('warn_val', parseFloat(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}><input type="number" step="any" value={newRow.crit_val ?? 90} onChange={e => updateNewField('crit_val', parseFloat(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}><input type="number" step="any" value={newRow.base_val ?? 50} onChange={e => updateNewField('base_val', parseFloat(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}><input type="number" value={newRow.decimals ?? 1} onChange={e => updateNewField('decimals', parseInt(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}>
          <select value={newRow.chart_type ?? 'sparkline'} onChange={e => updateNewField('chart_type', e.target.value)} style={{ ...CELL, cursor: 'pointer' }}>
            {CHART_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
          </select>
        </td>
        <td style={tdStyle}><input type="number" value={newRow.sort_order ?? 0} onChange={e => updateNewField('sort_order', parseInt(e.target.value) || 0)} style={CELL_NUM} /></td>
        <td style={tdStyle}>
          <select value={newRow.source_type ?? 'random_walk'} onChange={e => updateNewField('source_type', e.target.value)} style={{ ...CELL, cursor: 'pointer', fontSize: 11 }}>
            <optgroup label="Real"><option value="modbus">modbus</option></optgroup>
            <optgroup label="Simulate">{SOURCE_TYPES.filter(t => !['modbus','api','none'].includes(t)).map(t => <option key={t} value={t}>{t}</option>)}</optgroup>
            <optgroup label="Other"><option value="api">api</option><option value="none">none</option></optgroup>
          </select>
        </td>
        <td style={tdStyle} />
        <td style={{ ...tdStyle, whiteSpace: 'nowrap' }}>
          <div style={{ display: 'flex', gap: 3 }}>
            <button onClick={commitNewRow} style={{ padding: '2px 8px', borderRadius: 3, border: '1px solid var(--green)', background: 'transparent', color: 'var(--green)', cursor: 'pointer', fontSize: 10 }}>ok</button>
            <button onClick={() => setNewRow(null)} style={{ padding: '2px 6px', borderRadius: 3, border: '1px solid var(--border)', background: 'transparent', color: 'var(--muted)', cursor: 'pointer', fontSize: 10 }}>x</button>
          </div>
        </td>
      </tr>
    );
  };

  const columns = (
    <colgroup>
      <col style={{ width: COL_WIDTHS.color }} />
      <col style={{ width: COL_WIDTHS.name }} />
      <col style={{ width: COL_WIDTHS.key }} />
      <col style={{ width: COL_WIDTHS.unit }} />
      <col style={{ width: COL_WIDTHS.min_val }} />
      <col style={{ width: COL_WIDTHS.max_val }} />
      <col style={{ width: COL_WIDTHS.warn_val }} />
      <col style={{ width: COL_WIDTHS.crit_val }} />
      <col style={{ width: COL_WIDTHS.base_val }} />
      <col style={{ width: COL_WIDTHS.decimals }} />
      <col style={{ width: COL_WIDTHS.chart_type }} />
      <col style={{ width: COL_WIDTHS.sort_order }} />
      <col style={{ width: COL_WIDTHS.source_type }} />
      <col style={{ width: COL_WIDTHS.source_config }} />
      <col style={{ width: COL_WIDTHS.actions }} />
    </colgroup>
  );

  const headerRow = (
    <tr>
      <th style={thStyle}></th>
      <th style={thStyle}>name</th>
      <th style={thStyle}>key</th>
      <th style={thStyle}>unit</th>
      <th style={thStyle}>min</th>
      <th style={thStyle}>max</th>
      <th style={thStyle}>warn</th>
      <th style={thStyle}>crit</th>
      <th style={thStyle}>base</th>
      <th style={thStyle}>dec</th>
      <th style={thStyle}>chart</th>
      <th style={thStyle}>sort</th>
      <th style={thStyle}>source</th>
      <th style={thStyle}>cfg</th>
      <th style={thStyle}></th>
    </tr>
  );

  const tableContent = (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
        <span style={{ color: 'var(--muted)', fontSize: 13 }}>{sensors.length} sources</span>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="Search..." value={filter} onChange={e => setFilter(e.target.value)} style={{ ...inputStyle, width: 200 }} />
          <button style={{ ...btnStyle, borderColor: 'var(--green)', color: 'var(--green)' }} onClick={handleAdd}>+ Create Source</button>
        </div>
      </div>
      <div style={{ flex: 1, overflow: 'auto', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10 }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', minWidth: 1100 }}>
          {columns}
          <thead>{headerRow}</thead>
          <tbody>
            {renderNewRow()}
            {filtered.map(s => renderRow(s))}
          </tbody>
        </table>
      </div>
    </>
  );

  const configPopoverEl = configPopover && (
    <div
      ref={configRef}
      style={{
        position: 'fixed',
        left: Math.min(configPopover.rect.right + 4, window.innerWidth - 380),
        top: Math.min(configPopover.rect.top, window.innerHeight - 300),
        background: 'var(--card)',
        border: '1px solid var(--border)',
        borderRadius: 8,
        padding: 12,
        width: 340,
        maxHeight: 280,
        overflow: 'auto',
        zIndex: 70,
        boxShadow: '0 8px 24px rgba(0,0,0,.4)',
      }}
      onClick={e => e.stopPropagation()}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
        <span style={{ fontSize: 12, fontWeight: 600 }}>Source Config</span>
        <button onClick={saveConfig} style={{ padding: '3px 10px', borderRadius: 4, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 11 }}>Save</button>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
        <SourceConfigFields
          sourceType={configPopover.srcType}
          config={configPopover.config}
          onChange={c => setConfigPopover({ ...configPopover, config: c })}
        />
      </div>
    </div>
  );

  if (embedded) {
    return (<>{tableContent}{configPopoverEl}</>);
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1400, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16, flexShrink: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
            </button>
            <h1 style={{ fontSize: 20, fontWeight: 700 }}>Data Sources</h1>
          </div>
        </div>
        {tableContent}
      </div>
      {configPopoverEl}
    </div>
  );
}
