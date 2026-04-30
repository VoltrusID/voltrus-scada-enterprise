import React from 'react'
interface Props { statusColor?: string }
export const CapacitorSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 60 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <line x1="30" y1="0" x2="30" y2="22" />
    <line x1="15" y1="22" x2="45" y2="22" strokeWidth="3" />
    <line x1="15" y1="30" x2="45" y2="30" strokeWidth="3" />
    <line x1="15" y1="38" x2="45" y2="38" strokeWidth="3" />
    <line x1="30" y1="38" x2="30" y2="60" />
    <circle cx="30" cy="2" r="2.5" fill="none" stroke={statusColor} />
    <circle cx="30" cy="58" r="2.5" fill="none" stroke={statusColor} />
  </svg>
)
