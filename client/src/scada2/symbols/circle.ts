import type { SymbolDef } from './index'

export const circleSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="30" cy="30" r="25" />
    <circle cx="30" cy="5" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="55" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="5" cy="30" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="55" cy="30" r="2.5" fill="currentColor" stroke="none" />
  </svg>`,
}
