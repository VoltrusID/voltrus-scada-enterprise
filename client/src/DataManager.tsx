import { useState, useEffect } from 'react';
import { SensorManager } from './SensorManager';
import { ModbusDeviceManager } from './ModbusDeviceManager';
import { UserManager } from './UserManager';
import { btnStyle } from './styles';
import type { SensorConfig } from './types';

type Tab = 'sources' | 'modbus' | 'users';

export function DataManager({ sensors, onSave, onDelete, onAdd, onDuplicate, onClose, shortcuts, isAdmin }: {
  sensors: SensorConfig[];
  onSave: (s: SensorConfig, shortcut?: string | null) => void;
  onDelete: (id: number) => void;
  onAdd: (s: Partial<SensorConfig>) => void;
  onDuplicate: (s: SensorConfig) => void;
  onClose: () => void;
  shortcuts: Record<number, string>;
  isAdmin?: boolean;
}) {
  const [tab, setTab] = useState<Tab>('sources');

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

  const tabBtn = (t: Tab) => ({
    ...btnStyle, border: 'none', borderRadius: 0, cursor: 'pointer',
    color: tab === t ? 'var(--cyan)' : 'var(--muted)',
    background: tab === t ? 'rgba(34,197,94,.1)' : 'transparent',
    borderBottom: tab === t ? '2px solid var(--cyan)' : '2px solid transparent',
    fontWeight: tab === t ? 600 : 400,
  });

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ maxWidth: 1200, width: '100%', margin: '0 auto', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 0, flexShrink: 0 }}>
          <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>Data</h1>
          <div style={{ display: 'flex', borderBottom: '1px solid var(--border)', marginLeft: 8 }}>
            <button style={tabBtn('sources')} onClick={() => setTab('sources')}>Sources</button>
            <button style={tabBtn('modbus')} onClick={() => setTab('modbus')}>Modbus</button>
            {isAdmin && <button style={tabBtn('users')} onClick={() => setTab('users')}>Users</button>}
          </div>
        </div>
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0, marginTop: 16 }}>
          {tab === 'sources' && (
            <SensorManager
              sensors={sensors} onSave={onSave} onDelete={onDelete}
              onAdd={onAdd} onDuplicate={onDuplicate} onClose={onClose}
              shortcuts={shortcuts} embedded
            />
          )}
          {tab === 'modbus' && (
            <ModbusDeviceManager onClose={onClose} embedded />
          )}
          {tab === 'users' && isAdmin && (
            <UserManager onClose={onClose} embedded />
          )}
        </div>
      </div>
    </div>
  );
}
