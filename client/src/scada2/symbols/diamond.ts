import type { SymbolDef } from './index'

export const diamondSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <polygon points="30,5 55,30 30,55 5,30" />
    <circle cx="30" cy="5" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="55" cy="30" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="55" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="5" cy="30" r="2.5" fill="currentColor" stroke="none" />
  </svg>`,
}
