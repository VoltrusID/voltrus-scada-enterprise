import type { SymbolDef } from './index'

export const arrowSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 80 40" fill="none" stroke="currentColor" stroke-width="2">
    <line x1="5" y1="20" x2="65" y2="20" />
    <polygon points="65,10 80,20 65,30" fill="currentColor" stroke="none" />
    <circle cx="5" cy="20" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="80" cy="20" r="2.5" fill="currentColor" stroke="none" />
  </svg>`,
}
