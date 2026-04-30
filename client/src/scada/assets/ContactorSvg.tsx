import React from 'react'
interface Props { statusColor?: string }
export const ContactorSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 60 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <rect x="10" y="5" width="40" height="50" rx="3" />
    <line x1="0" y1="30" x2="10" y2="30" />
    <line x1="50" y1="30" x2="60" y2="30" />
    <line x1="18" y1="22" x2="30" y2="16" strokeWidth="2.5" />
    <line x1="30" y1="16" x2="42" y2="22" strokeWidth="1" strokeDasharray="3 2" />
    <line x1="18" y1="22" x2="18" y2="30" />
    <line x1="42" y1="22" x2="42" y2="30" />
    <rect x="22" y="36" width="16" height="10" rx="1" strokeDasharray="2 2" />
    <circle cx="2" cy="30" r="2.5" fill="none" stroke={statusColor} />
    <circle cx="58" cy="30" r="2.5" fill="none" stroke={statusColor} />
  </svg>
)
