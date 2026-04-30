import React from 'react'
interface Props { statusColor?: string }
export const GeneratorSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 70 70" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <circle cx="35" cy="35" r="28" />
    <text x="35" y="41" textAnchor="middle" fontSize="18" fontWeight="bold" fill={statusColor} stroke="none">G</text>
    <line x1="35" y1="0" x2="35" y2="7" />
    <line x1="35" y1="63" x2="35" y2="70" />
    <circle cx="35" cy="2" r="2.5" fill="none" stroke={statusColor} />
    <circle cx="35" cy="68" r="2.5" fill="none" stroke={statusColor} />
    <line x1="0" y1="35" x2="7" y2="35" />
    <circle cx="2" cy="35" r="2.5" fill="none" stroke={statusColor} />
    <line x1="63" y1="35" x2="70" y2="35" />
    <circle cx="68" cy="35" r="2.5" fill="none" stroke={statusColor} />
  </svg>
)
