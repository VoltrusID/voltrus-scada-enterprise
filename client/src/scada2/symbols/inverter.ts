import type { SymbolDef } from './index'

export const inverterSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 80 60" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="5" y="5" width="70" height="50" rx="3" />
    <text x="15" y="22" font-size="8" fill="currentColor" stroke="none" font-weight="bold">DC</text>
    <line x1="28" y1="30" x2="52" y2="30" />
    <polygon points="52,26 60,30 52,34" fill="currentColor" stroke="none" />
    <text x="62" y="22" font-size="8" fill="currentColor" stroke="none" font-weight="bold">AC</text>
    <path d="M 30 38 Q 35 33, 40 38 Q 45 43, 50 38" />
    <circle cx="40" cy="48" r="3" fill="currentColor" stroke="none" opacity="0.7" />
    <circle cx="2" cy="20" r="3" fill="none" stroke="currentColor" />
    <circle cx="78" cy="20" r="3" fill="none" stroke="currentColor" />
  </svg>`,
}
