import { type LicenseInfo } from './LicenseContext';

interface EngineeringBannerProps {
  licenseInfo: LicenseInfo;
  onActivate: () => void;
}

function formatRemaining(secs: number): string {
  const totalMinutes = Math.floor(secs / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return `${hours}h ${minutes}m`;
}

const activateButtonStyle: React.CSSProperties = {
  background: 'rgba(255,255,255,0.2)',
  border: '1px solid rgba(255,255,255,0.4)',
  borderRadius: 6,
  padding: '4px 14px',
  color: '#ffffff',
  fontSize: 12,
  fontWeight: 600,
  cursor: 'pointer',
  fontFamily: 'Inter, sans-serif',
};

const pollingButtonStyle: React.CSSProperties = {
  background: 'rgba(255,255,255,0.2)',
  border: '1px solid rgba(255,255,255,0.4)',
  borderRadius: 6,
  padding: '4px 14px',
  color: '#ffffff',
  fontSize: 12,
  fontWeight: 600,
  cursor: 'pointer',
  fontFamily: 'Inter, sans-serif',
};

export function EngineeringBanner({ licenseInfo, onActivate }: EngineeringBannerProps) {
  if (licenseInfo.status === 'Engineering') {
    const dailyBudget = licenseInfo.daily_budget_secs ?? 14400;
    const dailyUsed = licenseInfo.daily_used_secs ?? 0;
    const remainingSecs = dailyBudget - dailyUsed;
    const remainingText = formatRemaining(Math.max(0, remainingSecs));
    const budgetHours = Math.floor(dailyBudget / 3600);
    const isActive = licenseInfo.polling_active === true;
    const isLow = remainingSecs > 0 && remainingSecs < 30 * 60;

    const handleStart = async () => {
      try {
        const res = await fetch('/api/v1/engineering/polling/start', { method: 'POST', credentials: 'include' });
        if (!res.ok) console.error('Failed to start polling');
        // Reload to refresh license state
        window.location.reload();
      } catch (e) {
        console.error('Failed to start polling', e);
      }
    };

    const handleStop = async () => {
      try {
        const res = await fetch('/api/v1/engineering/polling/stop', { method: 'POST', credentials: 'include' });
        if (!res.ok) console.error('Failed to stop polling');
        window.location.reload();
      } catch (e) {
        console.error('Failed to stop polling', e);
      }
    };

    return (
      <div style={{
        background: isLow ? '#b45309' : '#1e3a5f',
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
        <span>
          Engineering Mode — {remainingText} of {budgetHours}h daily budget remaining
          {!isActive && ' (polling paused)'}
        </span>
        {isActive ? (
          <button onClick={handleStop} style={pollingButtonStyle}>
            Stop Polling
          </button>
        ) : (
          <button onClick={handleStart} style={pollingButtonStyle}>
            Start Polling
          </button>
        )}
        <button onClick={onActivate} style={activateButtonStyle}>
          Activate License
        </button>
      </div>
    );
  }

  if (licenseInfo.status === 'EngineeringPollingStopped') {
    const dailyBudget = licenseInfo.daily_budget_secs ?? 14400;
    const budgetHours = Math.floor(dailyBudget / 3600);

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
        <span>
          Daily polling budget exhausted ({budgetHours}h/day). Resumes tomorrow at midnight UTC.
          Config remains fully editable.
        </span>
        <button onClick={onActivate} style={activateButtonStyle}>
          Activate License
        </button>
      </div>
    );
  }

  return null;
}
