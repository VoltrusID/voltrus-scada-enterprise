import React from 'react'
interface Props { statusColor?: string }
export const BusbarSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 120 30" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <line x1="5" y1="15" x2="115" y2="15" strokeWidth="5" />
    <circle cx="20" cy="15" r="3" fill={statusColor} stroke="none" opacity="0.5" />
    <circle cx="50" cy="15" r="3" fill={statusColor} stroke="none" opacity="0.5" />
    <circle cx="80" cy="15" r="3" fill={statusColor} stroke="none" opacity="0.5" />
    <circle cx="20" cy="5" r="2.5" fill="none" stroke={statusColor} />
    <line x1="20" y1="7" x2="20" y2="12" />
    <circle cx="50" cy="5" r="2.5" fill="none" stroke={statusColor} />
    <line x1="50" y1="7" x2="50" y2="12" />
    <circle cx="80" cy="5" r="2.5" fill="none" stroke={statusColor} />
    <line x1="80" y1="7" x2="80" y2="12" />
    <circle cx="20" cy="25" r="2.5" fill="none" stroke={statusColor} />
    <line x1="20" y1="18" x2="20" y2="23" />
    <circle cx="50" cy="25" r="2.5" fill="none" stroke={statusColor} />
    <line x1="50" y1="18" x2="50" y2="23" />
    <circle cx="80" cy="25" r="2.5" fill="none" stroke={statusColor} />
    <line x1="80" y1="18" x2="80" y2="23" />
  </svg>
)
