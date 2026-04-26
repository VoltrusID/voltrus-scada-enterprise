import React from 'react'
interface Props { statusColor?: string }
export const TransformerSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 90 80" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <path d="M 25 15 Q 10 15, 10 25 Q 10 35, 25 35 Q 10 35, 10 45 Q 10 55, 25 55 Q 10 55, 10 65 Q 10 75, 25 75" />
    <path d="M 65 15 Q 80 15, 80 25 Q 80 35, 65 35 Q 80 35, 80 45 Q 80 55, 65 55 Q 80 55, 80 65 Q 80 75, 65 75" />
    <line x1="38" y1="10" x2="38" y2="80" strokeWidth="3" />
    <line x1="52" y1="10" x2="52" y2="80" strokeWidth="3" />
    <circle cx="5" cy="15" r="3" fill="none" stroke={statusColor} />
    <circle cx="5" cy="75" r="3" fill="none" stroke={statusColor} />
    <circle cx="85" cy="15" r="3" fill="none" stroke={statusColor} />
    <circle cx="85" cy="75" r="3" fill="none" stroke={statusColor} />
    <text x="20" y="8" fontSize="7" fill={statusColor} stroke="none">P</text>
    <text x="65" y="8" fontSize="7" fill={statusColor} stroke="none">S</text>
  </svg>
)
