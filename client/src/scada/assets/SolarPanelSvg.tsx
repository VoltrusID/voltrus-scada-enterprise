import React from 'react'
interface Props { statusColor?: string }
export const SolarPanelSvg: React.FC<Props> = ({ statusColor = '#ccc' }) => (
  <svg viewBox="0 0 100 60" width="100%" height="100%" fill="none" stroke={statusColor} strokeWidth="2">
    <rect x="5" y="5" width="90" height="40" rx="2" />
    <line x1="5" y1="25" x2="95" y2="25" />
    <line x1="35" y1="5" x2="35" y2="45" />
    <line x1="65" y1="5" x2="65" y2="45" />
    <line x1="25" y1="45" x2="20" y2="56" />
    <line x1="75" y1="45" x2="80" y2="56" />
    <circle cx="50" cy="15" r="4" strokeDasharray="2 2" />
    <circle cx="30" cy="57" r="3" fill={statusColor} stroke="none" />
    <text x="30" y="59" textAnchor="middle" fill="#fff" fontSize="5" stroke="none">+</text>
    <circle cx="70" cy="57" r="3" fill={statusColor} stroke="none" />
    <text x="70" y="59" textAnchor="middle" fill="#fff" fontSize="5" stroke="none">-</text>
  </svg>
)
