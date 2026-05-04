import { useState, useEffect, useRef } from 'react';
import { SensorManager } from './SensorManager';
import { ModbusDeviceManager } from './ModbusDeviceManager';
import { OpcUaDeviceManager } from './OpcUaDeviceManager';
import { UserManager } from './UserManager';
import { ApiKeyManager } from './ApiKeyManager';
import { NotificationSettingsPanel } from './NotificationSettingsPanel';
import { PushSettings } from './PushSettings';
import { HubManager } from './HubManager';
import { ReportManager } from './ReportManager';
import { OeePanel } from './OeePanel';
import { AnalyticsPanel } from './AnalyticsPanel';
import { RecipeManager } from './RecipeManager';
import { CameraManager } from './CameraManager';
import { S7DeviceManager } from './S7DeviceManager';
import { EnipDeviceManager } from './EnipDeviceManager';
import { Dnp3DeviceManager } from './Dnp3DeviceManager';
import { BacnetDeviceManager } from './BacnetDeviceManager';
import { SecurityAuditPanel } from './SecurityAuditPanel';
import { CompliancePanel } from './CompliancePanel';
import { RoleManager } from './RoleManager';
import { btnStyle } from './styles';
import { useLicense } from './LicenseContext';
import { FeatureBadge } from './components/FeatureBadge';
import type { SensorConfig } from './types';

type Tab = 'sources' | 'modbus' | 'opcua' | 's7' | 'enip' | 'dnp3' | 'bacnet' | 'hub' | 'users' | 'notifications' | 'api-keys' | 'reports' | 'oee' | 'analytics' | 'recipes' | 'cameras' | 'security' | 'compliance' | 'roles';

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
  const license = useLicense();
  const isOpcUaDisabled = license?.feature_flags?.opcua_enabled === false;
  const isHubDisabled = license?.feature_flags?.multi_site === false;
  const isReportsDisabled = license?.feature_flags?.pdf_reporting === false;
  const isOeeDisabled = license?.feature_flags?.oee === false;
  const isAnalyticsDisabled = license?.feature_flags?.analytics === false;
  const isRecipesDisabled = license?.feature_flags?.recipes === false;
  const isCamerasDisabled = license?.feature_flags?.rtsp_video === false;
  const isS7Disabled = license?.feature_flags?.s7_protocol === false;
  const isEnipDisabled = license?.feature_flags?.ethernet_ip === false;
  const isDnp3Disabled = license?.feature_flags?.dnp3 === false;
  const isBacnetDisabled = license?.feature_flags?.bacnet === false;
  const isComplianceDisabled = license?.feature_flags?.cfr_part_11 === false;
  const isRbacDisabled = license?.feature_flags?.rbac === false;

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
            {isAdmin && <button style={tabBtn('opcua')} onClick={() => setTab('opcua')} aria-selected={tab === 'opcua'}>OPC-UA{isOpcUaDisabled && <FeatureBadge tier="Professional" />}</button>}
            {isAdmin && <button style={tabBtn('s7')} onClick={() => setTab('s7')} aria-selected={tab === 's7'}>S7{isS7Disabled && <FeatureBadge tier="Enterprise" />}</button>}
            {isAdmin && <button style={tabBtn('enip')} onClick={() => setTab('enip')} aria-selected={tab === 'enip'}>ENIP{isEnipDisabled && <FeatureBadge tier="Enterprise" />}</button>}
            {isAdmin && <button style={tabBtn('dnp3')} onClick={() => setTab('dnp3')} aria-selected={tab === 'dnp3'}>DNP3{isDnp3Disabled && <FeatureBadge tier="Enterprise" />}</button>}
            {isAdmin && <button style={tabBtn('bacnet')} onClick={() => setTab('bacnet')} aria-selected={tab === 'bacnet'}>BACnet{isBacnetDisabled && <FeatureBadge tier="Enterprise" />}</button>}
            {isAdmin && <button style={tabBtn('users')} onClick={() => setTab('users')} aria-selected={tab === 'users'}>Users</button>}
            {isAdmin && <button style={tabBtn('roles')} onClick={() => setTab('roles')} aria-selected={tab === 'roles'}>Roles{isRbacDisabled && <FeatureBadge tier="Enterprise" />}</button>}
            {isAdmin && <button style={tabBtn('notifications')} onClick={() => setTab('notifications')} aria-selected={tab === 'notifications'}>Notifications</button>}
            {isAdmin && <button style={tabBtn('api-keys')} onClick={() => setTab('api-keys')} aria-selected={tab === 'api-keys'}>API Keys</button>}
            {isAdmin && <button style={tabBtn('hub')} onClick={() => setTab('hub')} aria-selected={tab === 'hub'}>Hub{isHubDisabled && <FeatureBadge tier="Professional" />}</button>}
            {isAdmin && <button style={tabBtn('reports')} onClick={() => setTab('reports')} aria-selected={tab === 'reports'}>Reports{isReportsDisabled && <FeatureBadge tier="Professional" />}</button>}
            <button style={tabBtn('oee')} onClick={() => setTab('oee')} aria-selected={tab === 'oee'}>OEE{isOeeDisabled && <FeatureBadge tier="Starter" />}</button>
            <button style={tabBtn('analytics')} onClick={() => setTab('analytics')} aria-selected={tab === 'analytics'}>Analytics{isAnalyticsDisabled && <FeatureBadge tier="Starter" />}</button>
            {isAdmin && <button style={tabBtn('recipes')} onClick={() => setTab('recipes')} aria-selected={tab === 'recipes'}>Recipes{isRecipesDisabled && <FeatureBadge tier="Professional" />}</button>}
            {isAdmin && <button style={tabBtn('cameras')} onClick={() => setTab('cameras')} aria-selected={tab === 'cameras'}>Cameras{isCamerasDisabled && <FeatureBadge tier="Enterprise" />}</button>}
            {isAdmin && <button style={tabBtn('security')} onClick={() => setTab('security')} aria-selected={tab === 'security'}>Security</button>}
            {isAdmin && <button style={tabBtn('compliance')} onClick={() => setTab('compliance')} aria-selected={tab === 'compliance'}>Compliance{isComplianceDisabled && <FeatureBadge tier="Enterprise" />}</button>}
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
          {tab === 'hub' && isAdmin && (
            <HubManager onClose={onClose} embedded />
          )}
          {tab === 'reports' && isAdmin && (
            <ReportManager onClose={onClose} embedded />
          )}
          {tab === 'oee' && (
            <OeePanel />
          )}
          {tab === 'analytics' && (
            <AnalyticsPanel />
          )}
          {tab === 'recipes' && isAdmin && (
            <RecipeManager />
          )}
          {tab === 'cameras' && isAdmin && (
            <CameraManager onClose={onClose} embedded />
          )}
          {tab === 's7' && isAdmin && (
            <S7DeviceManager onClose={onClose} embedded />
          )}
          {tab === 'enip' && isAdmin && (
            <EnipDeviceManager onClose={onClose} embedded />
          )}
          {tab === 'dnp3' && isAdmin && (
            <Dnp3DeviceManager onClose={onClose} embedded />
          )}
          {tab === 'bacnet' && isAdmin && (
            <BacnetDeviceManager onClose={onClose} embedded />
          )}
          {tab === 'security' && isAdmin && (
            <SecurityAuditPanel onClose={onClose} embedded />
          )}
          {tab === 'compliance' && isAdmin && (
            <CompliancePanel onClose={onClose} embedded />
          )}
          {tab === 'roles' && isAdmin && (
            <RoleManager onClose={onClose} embedded />
          )}
        </div>
      </div>
    </div>
  );
}
