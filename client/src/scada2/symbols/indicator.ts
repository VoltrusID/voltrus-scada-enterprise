import type { SymbolDef } from './index'

export const indicatorSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="8" y="8" width="44" height="44" rx="2" />
    <rect x="14" y="14" width="32" height="18" rx="1" fill="currentColor" fill-opacity="0.1" />
    <text x="30" y="28" text-anchor="middle" font-size="10" fill="currentColor" stroke="none" font-family="monospace">42.5</text>
    <text x="30" y="48" text-anchor="middle" font-size="7" fill="currentColor" stroke="none">TII</text>
    <circle cx="4" cy="30" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="56" cy="30" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="4" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="56" r="2.5" fill="currentColor" stroke="none" />
  </svg>`,
}
