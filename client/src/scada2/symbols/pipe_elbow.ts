import type { SymbolDef } from './index'

export const pipeElbowSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <path d="M 5 8 L 52 8 Q 57 8, 57 13 L 57 52" />
    <path d="M 5 22 L 38 22 Q 43 22, 43 27 L 43 52" />
    <circle cx="3" cy="15" r="3" fill="none" stroke="currentColor" />
    <circle cx="47" cy="57" r="3" fill="none" stroke="currentColor" />
  </svg>`,
}
