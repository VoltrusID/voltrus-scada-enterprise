import React from 'react'
interface Props { statusColor?: string }
export const GatewaySvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 80 70" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <rect x="5" y="15" width="70" height="50" rx="3" />
    <rect x="15" y="22" width="50" height="20" rx="1" strokeDasharray="3 2" />
    <line x1="40" y1="15" x2="40" y2="5" />
    <path d="M 32 8 Q 36 3, 40 8" />
    <path d="M 40 8 Q 44 3, 48 8" />
    <rect x="30" y="60" width="20" height="6" rx="1" />
    <line x1="34" y1="62" x2="34" y2="65" />
    <line x1="38" y1="62" x2="38" y2="65" />
    <line x1="42" y1="62" x2="42" y2="65" />
    <line x1="46" y1="62" x2="46" y2="65" />
    <circle cx="15" cy="55" r="2.5" fill={statusColor} stroke="none" opacity="0.7" />
    <circle cx="22" cy="55" r="2.5" fill={statusColor} stroke="none" opacity="0.4" />
  </svg>
)
