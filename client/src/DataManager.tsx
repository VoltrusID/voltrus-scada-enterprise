import { useState, useEffect, useRef } from 'react';
import { SensorManager } from './SensorManager';
import { ModbusDeviceManager } from './ModbusDeviceManager';
import { OpcUaDeviceManager } from './OpcUaDeviceManager';
import { UserManager } from './UserManager';
import { ApiKeyManager } from './ApiKeyManager';
import { NotificationSettingsPanel } from './NotificationSettingsPanel';
import { PushSettings } from './PushSettings';
import { btnStyle } from './styles';
import type { SensorConfig } from './types';

type Tab = 'sources' | 'modbus' | 'opcua' | 'users' | 'notifications' | 'api-keys';

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
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

  // Focus trap
  useEffect(() => {
    const modal = modalRef.current;
    if (!modal) return;

    const handleTabKey = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;
      const focusableSelector = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
      const focusableElements = modal.querySelectorAll<HTMLElement>(focusableSelector);
      if (focusableElements.length === 0) return;

      const firstEl = focusableElements[0];
      const lastEl = focusableElements[focusableElements.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === firstEl) {
          e.preventDefault();
          lastEl.focus();
        }
      } else {
        if (document.activeElement === lastEl) {
          e.preventDefault();
          firstEl.focus();
        }
      }
    };

    document.addEventListener('keydown', handleTabKey);
    return () => document.removeEventListener('keydown', handleTabKey);
  }, []);

  const tabBtn = (t: Tab) => ({
    ...btnStyle, border: 'none', borderRadius: 0, cursor: 'pointer',
    color: tab === t ? 'var(--cyan)' : 'var(--muted)',
    background: tab === t ? 'rgba(34,197,94,.1)' : 'transparent',
    borderBottom: tab === t ? '2px solid var(--cyan)' : '2px solid transparent',
    fontWeight: tab === t ? 600 : 400, minHeight: 44,
  });

  return (
    <div ref={modalRef} role="dialog" aria-label="Data Manager" aria-modal="true" style={{ position: 'fixed', inset: 0, background: 'var(--bg)', zIndex: 50, display: 'flex', flexDirection: 'column', padding: 24 }}>
      <div style={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 0, flexShrink: 0 }}>
          <button onClick={onClose} style={{ ...btnStyle, display: 'flex', alignItems: 'center', gap: 6, minHeight: 44 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>Back
          </button>
          <h1 style={{ fontSize: 20, fontWeight: 700 }}>Data</h1>
          <div style={{ display: 'flex', borderBottom: '1px solid var(--border)', marginLeft: 8 }}>
            <button style={tabBtn('sources')} onClick={() => setTab('sources')} aria-selected={tab === 'sources'}>Sources</button>
            <button style={tabBtn('modbus')} onClick={() => setTab('modbus')} aria-selected={tab === 'modbus'}>Modbus</button>
            {isAdmin && <button style={tabBtn('users')} onClick={() => setTab('users')} aria-selected={tab === 'users'}>Users</button>}
            {isAdmin && <button style={tabBtn('notifications')} onClick={() => setTab('notifications')} aria-selected={tab === 'notifications'}>Notifications</button>}
            {isAdmin && <button style={tabBtn('api-keys')} onClick={() => setTab('api-keys')} aria-selected={tab === 'api-keys'}>API Keys</button>}
            {isAdmin && <button style={tabBtn('opcua')} onClick={() => setTab('opcua')} aria-selected={tab === 'opcua'}>OPC-UA</button>}
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
          {tab === 'notifications' && isAdmin && (
            <>
              <PushSettings />
              <NotificationSettingsPanel onClose={onClose} embedded />
            </>
          )}
          {tab === 'api-keys' && isAdmin && (
            <ApiKeyManager onClose={onClose} embedded />
          )}
          {tab === 'opcua' && isAdmin && (
            <OpcUaDeviceManager onClose={onClose} embedded />
          )}
        </div>
      </div>
    </div>
  );
}
