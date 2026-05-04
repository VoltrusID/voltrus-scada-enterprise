import { useState } from 'react';

interface FeatureBadgeProps {
  tier: string;
}

export function FeatureBadge({ tier }: FeatureBadgeProps) {
  const [showTooltip, setShowTooltip] = useState(false);

  return (
    <span
      style={{ position: 'relative', display: 'inline-flex', alignItems: 'center' }}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <span style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 4,
        fontSize: 11,
        color: 'var(--muted)',
        background: 'rgba(255,255,255,0.06)',
        border: '1px solid var(--border)',
        borderRadius: 4,
        padding: '1px 6px',
        lineHeight: '18px',
        fontWeight: 500,
      }}>
        {'\uD83D\uDD12'} {tier}
      </span>
      {showTooltip && (
        <span style={{
          position: 'absolute',
          bottom: '100%',
          left: '50%',
          transform: 'translateX(-50%)',
          marginBottom: 6,
          background: 'var(--card)',
          border: '1px solid var(--border)',
          borderRadius: 6,
          padding: '6px 10px',
          fontSize: 11,
          color: 'var(--fg)',
          whiteSpace: 'nowrap',
          boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
          zIndex: 100,
          pointerEvents: 'none',
        }}>
          Available in {tier} tier and above. Upgrade at voltrus.id
        </span>
      )}
    </span>
  );
}
