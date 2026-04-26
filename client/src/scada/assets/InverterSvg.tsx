import React from 'react'
interface Props { statusColor?: string }
export const InverterSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 80 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <rect x="5" y="5" width="70" height="50" rx="3" />
    <text x="15" y="22" fontSize="8" fill={statusColor} stroke="none" fontWeight="bold">DC</text>
    <line x1="28" y1="30" x2="52" y2="30" />
    <polygon points="52,26 60,30 52,34" fill={statusColor} stroke="none" />
    <text x="62" y="22" fontSize="8" fill={statusColor} stroke="none" fontWeight="bold">AC</text>
    <path d="M 30 38 Q 35 33, 40 38 Q 45 43, 50 38" />
    <circle cx="40" cy="48" r="3" fill={statusColor} stroke="none" opacity="0.7" />
    <circle cx="2" cy="20" r="3" fill="none" stroke={statusColor} />
    <circle cx="78" cy="20" r="3" fill="none" stroke={statusColor} />
  </svg>
)
