import type { SymbolDef } from './index'

export const tankSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 80 80" fill="none" stroke="currentColor" stroke-width="2">
    <path d="M 15 20 L 15 72 Q 15 78, 21 78 L 59 78 Q 65 78, 65 72 L 65 20" />
    <path d="M 11 20 Q 11 8, 40 8 Q 69 8, 69 20 Z" />
    <line x1="15" y1="58" x2="65" y2="58" stroke-dasharray="4 3" />
    <line x1="15" y1="48" x2="65" y2="48" stroke-dasharray="4 3" />
    <line x1="15" y1="38" x2="65" y2="38" stroke-dasharray="4 3" />
    <line x1="15" y1="28" x2="65" y2="28" stroke-dasharray="4 3" />
    <line x1="5" y1="45" x2="15" y2="45" />
    <circle cx="3" cy="45" r="3" fill="none" stroke="currentColor" />
    <line x1="40" y1="0" x2="40" y2="8" />
    <circle cx="40" cy="3" r="3" fill="none" stroke="currentColor" />
    <line x1="65" y1="45" x2="75" y2="45" />
    <circle cx="77" cy="45" r="3" fill="none" stroke="currentColor" />
  </svg>`,
}
