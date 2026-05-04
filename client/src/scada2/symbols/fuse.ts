import type { SymbolDef } from './index'

export const fuseSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 50" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="10" y="12" width="40" height="26" rx="4" />
    <line x1="0" y1="25" x2="10" y2="25" />
    <line x1="50" y1="25" x2="60" y2="25" />
    <path d="M 16 25 Q 20 18, 25 25 Q 30 32, 35 25 Q 40 18, 44 25" stroke-width="1.5" />
    <circle cx="2" cy="25" r="2.5" fill="none" stroke="currentColor" />
    <circle cx="58" cy="25" r="2.5" fill="none" stroke="currentColor" />
  </svg>`,
}
