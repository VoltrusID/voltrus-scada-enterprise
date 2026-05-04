import type { SymbolDef } from './index'

export const currentTransformerSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 50" fill="none" stroke="currentColor" stroke-width="2">
    <line x1="0" y1="25" x2="18" y2="25" />
    <line x1="42" y1="25" x2="60" y2="25" />
    <rect x="18" y="8" width="24" height="34" rx="3" />
    <circle cx="30" cy="25" r="10" />
    <text x="30" y="29" text-anchor="middle" font-size="9" font-weight="bold" fill="currentColor" stroke="none">CT</text>
    <circle cx="2" cy="25" r="2.5" fill="none" stroke="currentColor" />
    <circle cx="58" cy="25" r="2.5" fill="none" stroke="currentColor" />
  </svg>`,
}
