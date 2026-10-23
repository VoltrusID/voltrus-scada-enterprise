import { useLicense } from '@voltrus/core/LicenseContext'
import { hasFeature } from '@voltrus/core/edition/license'
import { HaBanner } from './HaBanner'

/** Banners shown above the app header. HA status is only polled when the licence includes HA. */
export function Banners() {
  const license = useLicense()
  return hasFeature(license, 'ha') ? <HaBanner /> : null
}
