import type { BrandingSettings } from '@voltrus/core/edition/branding'
import { useLicense } from '@voltrus/core/LicenseContext'
import { hasFeature } from '@voltrus/core/edition/license'
import { BrandingDialog } from '../panels/BrandingDialog'
import { useSaveBranding } from '../queries/branding'
import { FeatureBadge } from './FeatureBadge'

/** Branding editor opened from the header logo; shows an upgrade notice without the white_label flag. */
export function BrandingDialogHost({ branding, onClose }: { branding: BrandingSettings; onClose: () => void }) {
  const license = useLicense()
  const save = useSaveBranding()

  if (hasFeature(license, 'white_label')) {
    return <BrandingDialog branding={branding} logoOnly={!hasFeature(license, 'theme_full')} onSave={settings => save.mutate(settings)} onClose={onClose} />
  }

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 420, padding: 20, display: 'flex', flexDirection: 'column', gap: 12, alignItems: 'flex-start' }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600 }}>White-label branding</div>
        <p style={{ margin: 0, fontSize: 13, color: 'var(--muted)' }}>
          Replace the Voltrus name and logo with your own. Requires the Integrator licence.
        </p>
        <FeatureBadge tier="Integrator" />
        <button onClick={onClose} style={{ alignSelf: 'flex-end', padding: '7px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Close</button>
      </div>
    </div>
  )
}
