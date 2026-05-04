import { createContext, useContext } from 'react';

export interface LicenseInfo {
  status: string;
  tier: string;
  feature_flags: Record<string, unknown>;
  days_remaining?: number | null;
  started_at?: number | null;
  expires_at?: number | null;
  license_key?: string;
}

export const LicenseContext = createContext<LicenseInfo | null>(null);

export function useLicense(): LicenseInfo | null {
  return useContext(LicenseContext);
}
