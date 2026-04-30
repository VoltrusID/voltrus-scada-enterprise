import React from 'react'
interface Props { statusColor?: string }
export const EarthSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 40 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <line x1="20" y1="0" x2="20" y2="25" />
    <line x1="8" y1="25" x2="32" y2="25" />
    <line x1="12" y1="32" x2="28" y2="32" />
    <line x1="16" y1="39" x2="24" y2="39" />
    <line x1="19" y1="46" x2="21" y2="46" />
    <circle cx="20" cy="2" r="2.5" fill="none" stroke={statusColor} />
  </svg>
)
