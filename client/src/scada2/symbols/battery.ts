import type { SymbolDef } from './index'

export const batterySymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 70 80" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="10" y="10" width="50" height="65" rx="3" />
    <rect x="25" y="5" width="20" height="5" rx="1" fill="currentColor" stroke="none" opacity="0.3" />
    <rect x="16" y="50" width="38" height="8" rx="1" fill="currentColor" stroke="none" opacity="0.2" />
    <rect x="16" y="38" width="38" height="8" rx="1" fill="currentColor" stroke="none" opacity="0.3" />
    <rect x="16" y="26" width="38" height="8" rx="1" fill="currentColor" stroke="none" opacity="0.4" />
    <text x="25" y="3" text-anchor="middle" font-size="6" fill="currentColor" stroke="none" font-weight="bold">+</text>
    <text x="45" y="3" text-anchor="middle" font-size="6" fill="currentColor" stroke="none" font-weight="bold">-</text>
    <circle cx="25" cy="2" r="2" fill="none" stroke="currentColor" />
    <circle cx="45" cy="2" r="2" fill="none" stroke="currentColor" />
  </svg>`,
}
