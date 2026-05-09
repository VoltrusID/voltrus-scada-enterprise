import type { SymbolDef } from './index'

export const valveSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 80 60" fill="none" stroke="currentColor" stroke-width="2">
    <line x1="0" y1="30" x2="25" y2="30" />
    <line x1="55" y1="30" x2="80" y2="30" />
    <path d="M 25 12 L 40 30 L 25 48 Z" fill="none" />
    <path d="M 55 12 L 40 30 L 55 48 Z" fill="none" />
    <line x1="40" y1="2" x2="40" y2="12" />
    <rect x="32" y="2" width="16" height="4" rx="1" />
    <circle cx="3" cy="30" r="3" fill="none" stroke="currentColor" />
    <circle cx="77" cy="30" r="3" fill="none" stroke="currentColor" />
  </svg>`,
}
