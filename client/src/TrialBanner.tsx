import { type LicenseInfo } from './LicenseContext';

interface TrialBannerProps {
  licenseInfo: LicenseInfo;
  onActivate: () => void;
}

export function TrialBanner({ licenseInfo, onActivate }: TrialBannerProps) {
  if (licenseInfo.status === 'Trial') {
    const days = licenseInfo.days_remaining ?? 0;
    return (
      <div style={{
        background: '#b45309',
        color: '#ffffff',
        padding: '10px 20px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 12,
        fontFamily: 'Inter, sans-serif',
        fontSize: 13,
        fontWeight: 500,
        flexShrink: 0,
        zIndex: 40,
      }}>
        <span>Trial Mode &mdash; {days} day{days !== 1 ? 's' : ''} remaining</span>
        <button
          onClick={onActivate}
          style={{
            background: 'rgba(255,255,255,0.2)',
            border: '1px solid rgba(255,255,255,0.4)',
            borderRadius: 6,
            padding: '4px 14px',
            color: '#ffffff',
            fontSize: 12,
            fontWeight: 600,
            cursor: 'pointer',
            fontFamily: 'Inter, sans-serif',
          }}
        >
          Activate License
        </button>
      </div>
    );
  }

  if (licenseInfo.status === 'TrialExpired') {
    return (
      <div style={{
        background: '#dc2626',
        color: '#ffffff',
        padding: '10px 20px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 12,
        fontFamily: 'Inter, sans-serif',
        fontSize: 13,
        fontWeight: 500,
        flexShrink: 0,
        zIndex: 40,
      }}>
        <span>License Required &mdash; Your trial has expired. Data is preserved but features are limited.</span>
        <button
          onClick={onActivate}
          style={{
            background: 'rgba(255,255,255,0.2)',
            border: '1px solid rgba(255,255,255,0.4)',
            borderRadius: 6,
            padding: '4px 14px',
            color: '#ffffff',
            fontSize: 12,
            fontWeight: 600,
            cursor: 'pointer',
            fontFamily: 'Inter, sans-serif',
          }}
        >
          Activate your license to continue
        </button>
      </div>
    );
  }

  return null;
}
