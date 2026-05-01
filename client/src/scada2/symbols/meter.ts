import type { SymbolDef } from './index'

export const meterSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="30" cy="30" r="25" />
    <text x="30" y="36" text-anchor="middle" font-size="16" fill="currentColor" stroke="none" font-weight="bold">M</text>
    <line x1="0" y1="30" x2="5" y2="30" />
    <circle cx="2" cy="30" r="2" fill="currentColor" stroke="none" />
    <line x1="55" y1="30" x2="60" y2="30" />
    <circle cx="58" cy="30" r="2" fill="currentColor" stroke="none" />
  </svg>`,
}
