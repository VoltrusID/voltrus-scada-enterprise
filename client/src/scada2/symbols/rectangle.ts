import type { SymbolDef } from './index'

export const rectangleSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="5" y="5" width="50" height="50" rx="0" />
    <circle cx="5" cy="5" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="55" cy="5" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="5" cy="55" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="55" cy="55" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="5" cy="30" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="55" cy="30" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="5" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="55" r="2.5" fill="currentColor" stroke="none" />
  </svg>`,
}
