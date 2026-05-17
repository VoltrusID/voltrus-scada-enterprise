import { createContext, useContext } from 'react';

export interface LicenseInfo {
  status: string;
  tier: string;
  feature_flags: Record<string, unknown>;
  days_remaining?: number | null;
  polling_elapsed_secs?: number | null;
  polling_remaining_secs?: number | null;
  daily_budget_secs?: number | null;
  daily_used_secs?: number | null;
  polling_active?: boolean | null;
  started_at?: number | null;
  expires_at?: number | null;
  license_key?: string;
  message?: string;
}

export const LicenseContext = createContext<LicenseInfo | null>(null);

export function useLicense(): LicenseInfo | null {
  return useContext(LicenseContext);
}
