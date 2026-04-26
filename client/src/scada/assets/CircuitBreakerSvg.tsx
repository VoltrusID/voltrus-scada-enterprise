import React from 'react'
interface Props { statusColor?: string }
export const CircuitBreakerSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 60 70" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <rect x="10" y="5" width="40" height="60" rx="3" />
    <line x1="30" y1="20" x2="30" y2="35" />
    <circle cx="30" cy="17" r="4" fill={statusColor} stroke="none" opacity="0.3" />
    <rect x="18" y="42" width="24" height="10" rx="2" strokeDasharray="3 2" />
    <text x="30" y="50" textAnchor="middle" fontSize="7" fill={statusColor} stroke="none">ON</text>
    <circle cx="5" cy="35" r="3" fill="none" stroke={statusColor} />
    <line x1="8" y1="35" x2="10" y2="35" />
    <circle cx="55" cy="35" r="3" fill="none" stroke={statusColor} />
    <line x1="50" y1="35" x2="52" y2="35" />
    <path d="M 22 30 Q 30 22, 38 30" strokeDasharray="2 2" />
  </svg>
)
