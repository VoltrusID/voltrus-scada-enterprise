import React from 'react'
interface Props { statusColor?: string }
export const BatterySvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 70 80" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <rect x="10" y="10" width="50" height="65" rx="3" />
    <rect x="25" y="5" width="20" height="5" rx="1" fill={statusColor} stroke="none" opacity="0.3" />
    <rect x="16" y="50" width="38" height="8" rx="1" fill={statusColor} stroke="none" opacity="0.2" />
    <rect x="16" y="38" width="38" height="8" rx="1" fill={statusColor} stroke="none" opacity="0.3" />
    <rect x="16" y="26" width="38" height="8" rx="1" fill={statusColor} stroke="none" opacity="0.4" />
    <text x="25" y="3" textAnchor="middle" fontSize="6" fill={statusColor} stroke="none" fontWeight="bold">+</text>
    <text x="45" y="3" textAnchor="middle" fontSize="6" fill={statusColor} stroke="none" fontWeight="bold">-</text>
    <circle cx="25" cy="2" r="2" fill="none" stroke={statusColor} />
    <circle cx="45" cy="2" r="2" fill="none" stroke={statusColor} />
  </svg>
)
