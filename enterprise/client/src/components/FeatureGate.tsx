import type { ReactNode } from 'react'
import { useLicense } from '@voltrus/core/LicenseContext'
import { hasFeature, type FeatureFlag } from '@voltrus/core/edition/license'
import { FeatureBadge } from './FeatureBadge'

interface FeatureGateProps {
  flag: FeatureFlag
  /** Lowest tier that includes the feature, shown in the badge. */
  tier: 'Integrator' | 'Enterprise'
  title: string
  description: string
  icon: ReactNode
  children: ReactNode
}

/** Renders `children` when the licence unlocks `flag`, an upgrade notice otherwise. */
export function FeatureGate({ flag, tier, title, description, icon, children }: FeatureGateProps) {
  const license = useLicense()
  if (hasFeature(license, flag)) return <>{children}</>

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 16 }}>
      <div style={{
        width: 64, height: 64, borderRadius: 16, background: 'rgba(34,197,94,.08)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        {icon}
      </div>
      <h2 style={{ fontSize: 18, fontWeight: 600 }}>{title}</h2>
      <p style={{ color: 'var(--muted)', fontSize: 13, maxWidth: 400, textAlign: 'center' }}>
        {description} Requires the {tier} licence.
      </p>
      <FeatureBadge tier={tier} />
    </div>
  )
}
