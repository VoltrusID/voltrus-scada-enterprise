import type { EnterpriseClientModule } from '@voltrus/core/edition/types'
import { FeatureGate } from './components/FeatureGate'
import { Banners } from './components/Banners'
import { BrandingDialogHost } from './components/BrandingDialogHost'
import { telegramChannel } from './channels/telegram'
import { whatsappChannel } from './channels/whatsapp'
import { BrandingPanel } from './panels/BrandingPanel'
import { CameraManager } from './panels/CameraManager'
import { CompliancePanel, ESignaturesTab } from './panels/CompliancePanel'
import { EnergyPanel } from './panels/EnergyPanel'
import { HubManager } from './panels/HubManager'
import { LicensePanel } from './panels/LicensePanel'
import { LoginProviders } from './panels/LoginProviders'
import { ReportManager } from './panels/ReportManager'
import { SecurityAuditPanel } from './panels/SecurityAuditPanel'
import { useBranding } from './queries/branding'
import { CameraScreenWidget, CameraWidgetSettings } from './widgets/CameraScreenWidget'
import { cameraWidgetDef } from './widgets/cameraWidgetModel'
import de from './locales/de.json'
import en from './locales/en.json'
import id from './locales/id.json'
import zh from './locales/zh.json'

const SIZE = 16
const svg = (children: React.ReactNode, size = SIZE) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">{children}</svg>
)

const HUB_PATHS = (
  <>
    <circle cx="12" cy="12" r="3" /><path d="M12 2v4m0 12v4M2 12h4m12 0h4" />
    <path d="M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83" />
  </>
)
const CAMERA_PATHS = (
  <>
    <path d="M23 7l-7 5 7 5V7z" /><rect x="1" y="5" width="15" height="14" rx="2" ry="2" />
  </>
)
const REPORT_PATHS = (
  <>
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" />
    <line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" />
  </>
)
const SECURITY_PATHS = (
  <>
    <rect x="3" y="11" width="18" height="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" />
  </>
)
const BRANDING_PATHS = (
  <>
    <circle cx="13.5" cy="6.5" r="1.5" /><circle cx="17.5" cy="10.5" r="1.5" /><circle cx="8.5" cy="7.5" r="1.5" /><circle cx="6.5" cy="12.5" r="1.5" />
    <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.9 0 1.7-.8 1.7-1.7 0-.4-.2-.8-.4-1.1-.3-.3-.4-.7-.4-1.1 0-.9.8-1.7 1.7-1.7H17c2.8 0 5-2.2 5-5C22 6 17.5 2 12 2z" />
  </>
)
const ENERGY_PATHS = <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
const COMPLIANCE_PATHS = (
  <>
    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" /><polyline points="9 12 11 14 15 10" />
  </>
)

const enterprise: EnterpriseClientModule = {
  dataManagerTabs: [
    {
      id: 'hub', label: 'Hub', group: 'Data', admin: true, icon: svg(HUB_PATHS),
      render: ({ onClose }) => (
        <FeatureGate flag="multi_site" tier="Integrator" title="Multi-Site Hub" icon={svg(HUB_PATHS, 28)}
          description="Connect and manage multiple Voltrus instances from a single dashboard.">
          <HubManager onClose={onClose} embedded />
        </FeatureGate>
      ),
    },
    {
      id: 'cameras', label: 'Cameras', group: 'Data', admin: true, icon: svg(CAMERA_PATHS),
      render: ({ onClose }) => (
        <FeatureGate flag="rtsp_video" tier="Enterprise" title="RTSP Video" icon={svg(CAMERA_PATHS, 28)}
          description="Connect and manage RTSP camera feeds with alarm-triggered snapshots.">
          <CameraManager onClose={onClose} embedded />
        </FeatureGate>
      ),
    },
    {
      id: 'reports', label: 'Reports', group: 'Insights', admin: true, icon: svg(REPORT_PATHS),
      render: ({ onClose }) => (
        <FeatureGate flag="pdf_reporting" tier="Integrator" title="Scheduled Reports" icon={svg(REPORT_PATHS, 28)}
          description="Schedule automated reports with sensor data and summaries delivered to your inbox.">
          <ReportManager onClose={onClose} embedded />
        </FeatureGate>
      ),
    },
    {
      // Voltrus Energy (PLN bill shadow). Core has no `energy` flag yet: like the
      // server, the tab follows the Integrator tier (pdf_reporting is on exactly there).
      id: 'energy', label: 'PLN Bill', group: 'Insights', icon: svg(ENERGY_PATHS),
      render: () => (
        <FeatureGate flag="pdf_reporting" tier="Integrator" title="Voltrus Energy" icon={svg(ENERGY_PATHS, 28)}
          description="Shadow your PLN bill every day: WBP premium, excess kVArh charge and power factor alerts.">
          <EnergyPanel />
        </FeatureGate>
      ),
    },
    {
      id: 'security', label: 'Security', group: 'Admin', admin: true, strictAdmin: true, icon: svg(SECURITY_PATHS),
      render: ({ onClose }) => (
        <FeatureGate flag="iec_62443" tier="Enterprise" title="IEC 62443 Security Audit" icon={svg(SECURITY_PATHS, 28)}
          description="Review security events and tune detection thresholds.">
          <SecurityAuditPanel onClose={onClose} embedded />
        </FeatureGate>
      ),
    },
    {
      id: 'compliance', label: 'Compliance', group: 'Admin', admin: true, strictAdmin: true, icon: svg(COMPLIANCE_PATHS),
      render: ({ onClose }) => (
        <FeatureGate flag="cfr_part_11" tier="Enterprise" title="21 CFR Part 11" icon={svg(COMPLIANCE_PATHS, 28)}
          description="Electronic signatures, record checksums and compliance configuration.">
          <CompliancePanel onClose={onClose} embedded />
        </FeatureGate>
      ),
    },
    {
      id: 'branding', label: 'Branding', group: 'Admin', admin: true, strictAdmin: true, icon: svg(BRANDING_PATHS),
      render: () => (
        <FeatureGate flag="white_label" tier="Integrator" title="White-label branding" icon={svg(BRANDING_PATHS, 28)}
          description="Your name, logo, colours for the light and dark theme, and font.">
          <BrandingPanel />
        </FeatureGate>
      ),
    },
    {
      // Every signed-in person signs here; the Compliance tab above is for administrators.
      id: 'e-signatures', label: 'E-Signatures', group: 'Insights', icon: svg(COMPLIANCE_PATHS),
      render: () => (
        <FeatureGate flag="cfr_part_11" tier="Enterprise" title="Electronic signatures" icon={svg(COMPLIANCE_PATHS, 28)}
          description="Sign records with your password, a meaning and a reason (21 CFR Part 11).">
          <div style={{ padding: 24 }}><ESignaturesTab /></div>
        </FeatureGate>
      ),
    },
  ],
  LicensePanel,
  LoginProviders,
  branding: { useBranding, Dialog: BrandingDialogHost },
  Banners,
  notificationChannels: [telegramChannel, whatsappChannel],
  // Live camera on dashboards and SCADA screens (Enterprise video).
  screenWidgets: [
    cameraWidgetDef({ render: CameraScreenWidget, configEditor: CameraWidgetSettings, icon: svg(CAMERA_PATHS) }),
  ],
  locales: { en, id, zh, de },
}

export default enterprise
