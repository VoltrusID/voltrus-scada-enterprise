import type { SymbolDef } from './index'

export const voltageTransformerSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 50" fill="none" stroke="currentColor" stroke-width="2">
    <line x1="0" y1="25" x2="18" y2="25" />
    <line x1="42" y1="25" x2="60" y2="25" />
    <rect x="18" y="8" width="24" height="34" rx="3" />
    <circle cx="26" cy="25" r="9" />
    <circle cx="34" cy="25" r="9" />
    <text x="30" y="28" text-anchor="middle" font-size="7" font-weight="bold" fill="currentColor" stroke="none">PT</text>
    <circle cx="2" cy="25" r="2.5" fill="none" stroke="currentColor" />
    <circle cx="58" cy="25" r="2.5" fill="none" stroke="currentColor" />
  </svg>`,
}
