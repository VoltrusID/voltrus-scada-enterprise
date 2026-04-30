import React from 'react'
interface Props { statusColor?: string }
export const IsolatorSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 60 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <circle cx="30" cy="30" r="6" />
    <line x1="30" y1="24" x2="30" y2="5" />
    <line x1="30" y1="36" x2="30" y2="55" />
    <line x1="0" y1="30" x2="24" y2="30" />
    <line x1="36" y1="30" x2="60" y2="30" />
    <line x1="24" y1="30" x2="18" y2="12" strokeWidth="2.5" />
    <circle cx="2" cy="30" r="2.5" fill="none" stroke={statusColor} />
    <circle cx="58" cy="30" r="2.5" fill="none" stroke={statusColor} />
  </svg>
)
