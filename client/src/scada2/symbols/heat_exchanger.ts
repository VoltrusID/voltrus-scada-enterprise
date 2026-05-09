import type { SymbolDef } from './index'

export const heatExchangerSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 90 70" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="15" y="8" width="60" height="54" rx="3" />
    <path d="M 15 20 Q 30 28, 45 20 Q 60 12, 75 20" />
    <path d="M 15 35 Q 30 43, 45 35 Q 60 27, 75 35" />
    <path d="M 15 50 Q 30 58, 45 50 Q 60 42, 75 50" />
    <line x1="0" y1="35" x2="15" y2="35" />
    <circle cx="3" cy="35" r="3" fill="none" stroke="currentColor" />
    <line x1="75" y1="35" x2="90" y2="35" />
    <circle cx="87" cy="35" r="3" fill="none" stroke="currentColor" />
    <line x1="45" y1="0" x2="45" y2="8" />
    <circle cx="45" cy="3" r="3" fill="none" stroke="currentColor" />
    <line x1="45" y1="62" x2="45" y2="70" />
    <circle cx="45" cy="67" r="3" fill="none" stroke="currentColor" />
  </svg>`,
}
