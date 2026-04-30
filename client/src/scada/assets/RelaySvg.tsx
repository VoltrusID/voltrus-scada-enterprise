import React from 'react'
interface Props { statusColor?: string }
export const RelaySvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 60 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <circle cx="30" cy="30" r="22" />
    <text x="30" y="35" textAnchor="middle" fontSize="14" fontWeight="bold" fill={statusColor} stroke="none">K</text>
    <line x1="0" y1="30" x2="8" y2="30" />
    <circle cx="2" cy="30" r="2.5" fill="none" stroke={statusColor} />
    <line x1="52" y1="30" x2="60" y2="30" />
    <circle cx="58" cy="30" r="2.5" fill="none" stroke={statusColor} />
    <line x1="30" y1="0" x2="30" y2="8" />
    <circle cx="30" cy="2" r="2.5" fill="none" stroke={statusColor} />
    <line x1="30" y1="52" x2="30" y2="60" />
    <circle cx="30" cy="58" r="2.5" fill="none" stroke={statusColor} />
  </svg>
)
