import type { SymbolDef } from './index'

export const circuitBreakerSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 70" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="10" y="5" width="40" height="60" rx="3" />
    <line x1="30" y1="20" x2="30" y2="35" />
    <circle cx="30" cy="17" r="4" fill="currentColor" stroke="none" opacity="0.3" />
    <rect x="18" y="42" width="24" height="10" rx="2" stroke-dasharray="3 2" />
    <text x="30" y="50" text-anchor="middle" font-size="7" fill="currentColor" stroke="none">ON</text>
    <circle cx="5" cy="35" r="3" fill="none" stroke="currentColor" />
    <line x1="8" y1="35" x2="10" y2="35" />
    <circle cx="55" cy="35" r="3" fill="none" stroke="currentColor" />
    <line x1="50" y1="35" x2="52" y2="35" />
    <path d="M 22 30 Q 30 22, 38 30" stroke-dasharray="2 2" />
  </svg>`,
}
