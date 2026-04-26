import React from 'react'
interface Props { statusColor?: string }
export const MeterSvg: React.FC<Props> = ({ statusColor = '#333' }) => (
  <svg viewBox="0 0 60 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <circle cx="30" cy="30" r="25" />
    <text x="30" y="36" textAnchor="middle" fontSize="16" fill={statusColor} stroke="none" fontWeight="bold">M</text>
    <line x1="0" y1="30" x2="5" y2="30" />
    <circle cx="2" cy="30" r="2" fill={statusColor} stroke="none" />
    <line x1="55" y1="30" x2="60" y2="30" />
    <circle cx="58" cy="30" r="2" fill={statusColor} stroke="none" />
  </svg>
)
