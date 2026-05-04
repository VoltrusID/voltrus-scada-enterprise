import type { SymbolDef } from './index'

export const loadSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <line x1="30" y1="0" x2="30" y2="12" />
    <path d="M 30 12 L 22 20 L 38 28 L 22 36 L 38 44 L 30 48" stroke-width="2" />
    <line x1="30" y1="48" x2="30" y2="60" />
    <circle cx="30" cy="2" r="2.5" fill="none" stroke="currentColor" />
    <circle cx="30" cy="58" r="2.5" fill="none" stroke="currentColor" />
    <line x1="0" y1="30" x2="22" y2="30" stroke-width="1" stroke-dasharray="3 2" />
    <circle cx="2" cy="30" r="2.5" fill="none" stroke="currentColor" />
    <line x1="38" y1="30" x2="60" y2="30" stroke-width="1" stroke-dasharray="3 2" />
    <circle cx="58" cy="30" r="2.5" fill="none" stroke="currentColor" />
  </svg>`,
}
