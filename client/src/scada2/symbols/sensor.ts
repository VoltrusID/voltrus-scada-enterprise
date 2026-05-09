import type { SymbolDef } from './index'

export const sensorSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="30" cy="30" r="22" />
    <text x="30" y="25" text-anchor="middle" font-size="10" fill="currentColor" stroke="none" font-weight="bold">T</text>
    <line x1="20" y1="30" x2="40" y2="30" stroke-width="1.5" />
    <text x="30" y="42" text-anchor="middle" font-size="7" fill="currentColor" stroke="none">TIT</text>
    <circle cx="30" cy="4" r="2.5" fill="currentColor" stroke="none" />
    <circle cx="30" cy="56" r="2.5" fill="currentColor" stroke="none" />
  </svg>`,
}
