# SCADA 2D Fabric.js Redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the raw SVG SCADA editor with a Fabric.js canvas-based implementation featuring expression-based visual rules, rich labels, and AI-scaffoldable JSON serialization.

**Architecture:** Fabric.js canvas renders all SCADA elements (equipment, connections, labels) as custom subclasses. A render loop reads live sensor data from the existing dashboard store, evaluates visual rule expressions, and updates canvas object properties. Serialization uses Fabric.js built-in `toJSON()`/`loadFromJSON()` with custom revivers.

**Tech Stack:** Fabric.js 6, React 19, TypeScript, existing custom dashboard store

---

## File Structure

### New files (in `client/src/scada2/`)

| File | Responsibility |
|------|---------------|
| `ScadaView.tsx` | Top-level wrapper: mode switching, state management, save/load |
| `ScadaCanvas.tsx` | Fabric.js canvas mount, zoom/pan, tool dispatch |
| `ScadaToolbar.tsx` | Edit toolbar buttons (edit, tools, save, label) |
| `ScadaPalette.tsx` | Equipment type picker for adding elements |
| `ScadaEditPanel.tsx` | Property panel: name, bindings, visual rules, labels editor |
| `engine/types.ts` | All type definitions for the SCADA system |
| `engine/expression.ts` | Recursive descent expression evaluator |
| `engine/bindings.ts` | Resolves sensor data into expression context |
| `engine/render-loop.ts` | Live data tick: evaluate rules, update objects |
| `engine/serializer.ts` | Canvas JSON <-> API format conversion |
| `objects/ScadaEquipment.ts` | Custom Fabric.Group for equipment |
| `objects/ScadaConnection.ts` | Custom Fabric.Polyline for connections |
| `objects/ScadaLabel.ts` | Custom Fabric.Text for labels |
| `symbols/index.ts` | Symbol registry mapping type -> SVG string + ports |
| `symbols/*.ts` | One file per equipment type (20 files) |
| `defaults.ts` | Default sizes, port positions, connection colors |

### Modified files

| File | Change |
|------|--------|
| `client/package.json` | Add `fabric` dependency |
| `client/src/App.tsx` | Import `ScadaView` from `scada2/` instead of `scada/` |

### Deleted files

| Path | Reason |
|------|--------|
| `client/src/scada/` (entire directory) | Replaced by `scada2/` |

---

### Task 1: Install Fabric.js and create type definitions

**Files:**
- Modify: `client/package.json`
- Create: `client/src/scada2/engine/types.ts`
- Create: `client/src/scada2/defaults.ts`

- [ ] **Step 1: Install fabric**

```bash
cd client && npm install fabric
```

- [ ] **Step 2: Create engine/types.ts**

```ts
// client/src/scada2/engine/types.ts
import type { ScadaEquipmentType, ScadaConnectionType } from '../../types'

export interface SensorBinding {
  sensor_id: number
  alias?: string
}

export interface VisualRule {
  target: 'fill' | 'stroke' | 'opacity' | 'visible' | 'text' | 'strokeWidth'
  expression: string
}

export interface AttachedLabel {
  template: string
  offset: [number, number]
  style?: { fontSize?: number; fill?: string; fontWeight?: string }
}

export interface PortDef {
  x: number
  y: number
  side: 'top' | 'bottom' | 'left' | 'right'
}

export interface ScadaEquipmentProps {
  id: string
  equipment_type: ScadaEquipmentType
  name: string
  bindings: SensorBinding[]
  visual_rules: VisualRule[]
  attached_labels: AttachedLabel[]
  ports: PortDef[]
}

export interface ScadaConnectionProps {
  id: string
  from_id: string
  to_id: string
  connection_type: ScadaConnectionType
  visual_rules: VisualRule[]
  attached_labels: AttachedLabel[]
}

export interface ScadaLabelProps {
  id: string
  template: string
  bindings: SensorBinding[]
  visual_rules: VisualRule[]
}

export interface ExprContext {
  value: number
  warn: number
  crit: number
  min: number
  max: number
  status: string
  [key: string]: number | string
}

export type ScadaTool = 'select' | 'connect' | 'label' | 'delete'

export interface ScadaCanvasState {
  editMode: boolean
  activeTool: ScadaTool
  selectedId: string | null
  pendingType: ScadaEquipmentType | null
  connectFrom: { eqId: string; port: PortDef } | null
}
```

- [ ] **Step 3: Create defaults.ts**

```ts
// client/src/scada2/defaults.ts
import type { ScadaEquipmentType } from '../types'
import type { PortDef } from './engine/types'

export const DEFAULT_SIZES: Record<ScadaEquipmentType, { width: number; height: number }> = {
  solar_panel: { width: 100, height: 60 },
  inverter: { width: 80, height: 60 },
  gateway: { width: 80, height: 70 },
  transformer: { width: 90, height: 80 },
  battery: { width: 70, height: 80 },
  meter: { width: 60, height: 60 },
  circuit_breaker: { width: 60, height: 70 },
  busbar: { width: 120, height: 30 },
  isolator: { width: 60, height: 60 },
  earth: { width: 40, height: 60 },
  fuse: { width: 60, height: 50 },
  capacitor: { width: 60, height: 60 },
  generator: { width: 70, height: 70 },
  motor: { width: 70, height: 70 },
  surge_arrester: { width: 50, height: 70 },
  current_transformer: { width: 60, height: 50 },
  voltage_transformer: { width: 60, height: 50 },
  contactor: { width: 60, height: 60 },
  relay: { width: 60, height: 60 },
  load: { width: 60, height: 60 },
}

export const PORT_POSITIONS: Record<ScadaEquipmentType, PortDef[]> = {
  solar_panel: [
    { x: 0.2, y: 1, side: 'bottom' },
    { x: 0.8, y: 1, side: 'bottom' },
  ],
  inverter: [
    { x: 0, y: 0.3, side: 'left' },
    { x: 1, y: 0.3, side: 'right' },
  ],
  gateway: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  battery: [
    { x: 0.3, y: 0, side: 'top' },
    { x: 0.7, y: 0, side: 'top' },
  ],
  meter: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  circuit_breaker: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  busbar: [
    { x: 0.15, y: 0, side: 'top' },
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.85, y: 0, side: 'top' },
    { x: 0.15, y: 1, side: 'bottom' },
    { x: 0.5, y: 1, side: 'bottom' },
    { x: 0.85, y: 1, side: 'bottom' },
  ],
  isolator: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  earth: [{ x: 0.5, y: 0, side: 'top' }],
  fuse: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  capacitor: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  generator: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  motor: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  surge_arrester: [{ x: 0.5, y: 0, side: 'top' }],
  current_transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  voltage_transformer: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  contactor: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
  ],
  relay: [
    { x: 0, y: 0.5, side: 'left' },
    { x: 1, y: 0.5, side: 'right' },
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
  load: [
    { x: 0.5, y: 0, side: 'top' },
    { x: 0.5, y: 1, side: 'bottom' },
  ],
}

export const CONNECTION_COLORS: Record<string, string> = {
  power: '#f59e0b',
  data: '#3b82f6',
  flow: '#22c55e',
}
```

- [ ] **Step 4: Commit**

```bash
git add client/package.json client/package-lock.json client/src/scada2/engine/types.ts client/src/scada2/defaults.ts
git commit -m "feat(scada2): install fabric and add type definitions"
```

---

### Task 2: Build the expression engine

**Files:**
- Create: `client/src/scada2/engine/expression.ts`

This is a safe, AST-based recursive descent parser. No dynamic code execution.

- [ ] **Step 1: Create expression.ts**

```ts
// client/src/scada2/engine/expression.ts
import type { ExprContext } from './types'

type Token =
  | { type: 'number'; value: number }
  | { type: 'string'; value: string }
  | { type: 'ident'; value: string }
  | { type: 'op'; value: string }
  | { type: 'lparen' }
  | { type: 'rparen' }
  | { type: 'comma' }

type AST =
  | { type: 'literal'; value: string | number | boolean }
  | { type: 'var'; name: string }
  | { type: 'unary'; op: string; operand: AST }
  | { type: 'binary'; op: string; left: AST; right: AST }
  | { type: 'ternary'; cond: AST; ifTrue: AST; ifFalse: AST }
  | { type: 'call'; name: string; args: AST[] }

function tokenize(expr: string): Token[] {
  const tokens: Token[] = []
  let i = 0
  while (i < expr.length) {
    const ch = expr[i]
    if (/\s/.test(ch)) { i++; continue }
    if (ch === '(') { tokens.push({ type: 'lparen' }); i++; continue }
    if (ch === ')') { tokens.push({ type: 'rparen' }); i++; continue }
    if (ch === ',') { tokens.push({ type: 'comma' }); i++; continue }
    if (ch === "'" || ch === '"') {
      let s = ''
      i++
      while (i < expr.length && expr[i] !== ch) { s += expr[i]; i++ }
      i++
      tokens.push({ type: 'string', value: s })
      continue
    }
    if (/[0-9.]/.test(ch)) {
      let n = ''
      while (i < expr.length && /[0-9.]/.test(expr[i])) { n += expr[i]; i++ }
      tokens.push({ type: 'number', value: parseFloat(n) })
      continue
    }
    if (/[a-zA-Z_]/.test(ch)) {
      let id = ''
      while (i < expr.length && /[a-zA-Z0-9_]/.test(expr[i])) { id += expr[i]; i++ }
      tokens.push({ type: 'ident', value: id })
      continue
    }
    const two = expr.slice(i, i + 2)
    if (['>=', '<=', '==', '!=', '&&', '||'].includes(two)) {
      tokens.push({ type: 'op', value: two }); i += 2; continue
    }
    if (['>', '<', '+', '-', '*', '/', '!'].includes(ch)) {
      tokens.push({ type: 'op', value: ch }); i++; continue
    }
    i++
  }
  return tokens
}

function parse(tokens: Token[]): AST {
  let pos = 0
  const peek = () => tokens[pos]
  const consume = () => tokens[pos++]

  function parseExpr(): AST {
    return parseTernary()
  }

  function parseTernary(): AST {
    const cond = parseOr()
    const t = peek()
    if (t?.type === 'comma') {
      consume()
      const ifTrue = parseExpr()
      const t2 = peek()
      if (t2?.type === 'comma') {
        consume()
        const ifFalse = parseExpr()
        return { type: 'ternary', cond, ifTrue, ifFalse }
      }
      return { type: 'ternary', cond, ifTrue, ifFalse: { type: 'literal', value: '' } }
    }
    return cond
  }

  function parseOr(): AST {
    let left = parseAnd()
    while (peek()?.type === 'op' && (peek()!.value === '||' || (peek() as { type: string; value: string }).value === 'or')) {
      consume()
      left = { type: 'binary', op: '||', left, right: parseAnd() }
    }
    return left
  }

  function parseAnd(): AST {
    let left = parseComparison()
    while (peek()?.type === 'op' && (peek()!.value === '&&' || (peek() as { type: string; value: string }).value === 'and')) {
      consume()
      left = { type: 'binary', op: '&&', left, right: parseComparison() }
    }
    return left
  }

  function parseComparison(): AST {
    let left = parseAdd()
    const t = peek()
    if (t?.type === 'op' && ['>', '<', '>=', '<=', '==', '!='].includes(t.value)) {
      consume()
      return { type: 'binary', op: t.value, left, right: parseAdd() }
    }
    return left
  }

  function parseAdd(): AST {
    let left = parseMul()
    while (peek()?.type === 'op' && ['+', '-'].includes((peek() as { value: string }).value)) {
      const op = consume().value as string
      left = { type: 'binary', op, left, right: parseMul() }
    }
    return left
  }

  function parseMul(): AST {
    let left = parseUnary()
    while (peek()?.type === 'op' && ['*', '/'].includes((peek() as { value: string }).value)) {
      const op = consume().value as string
      left = { type: 'binary', op, left, right: parseUnary() }
    }
    return left
  }

  function parseUnary(): AST {
    const t = peek()
    if (t?.type === 'op' && t.value === '-') {
      consume()
      return { type: 'unary', op: '-', operand: parseUnary() }
    }
    if (t?.type === 'op' && (t.value === '!' || t.value === 'not')) {
      consume()
      return { type: 'unary', op: '!', operand: parseUnary() }
    }
    return parsePrimary()
  }

  function parsePrimary(): AST {
    const t = peek()
    if (!t) return { type: 'literal', value: '' }

    if (t.type === 'number') { consume(); return { type: 'literal', value: t.value } }
    if (t.type === 'string') { consume(); return { type: 'literal', value: t.value } }

    if (t.type === 'ident') {
      const name = (consume() as { value: string }).value
      if (name === 'true') return { type: 'literal', value: true }
      if (name === 'false') return { type: 'literal', value: false }
      if (name === 'if' || name === 'abs') {
        const next = peek()
        if (next?.type === 'lparen') {
          consume()
          const args: AST[] = []
          while (peek()?.type !== 'rparen' && pos < tokens.length) {
            args.push(parseExpr())
            if (peek()?.type === 'comma') consume()
          }
          if (peek()?.type === 'rparen') consume()
          if (name === 'if' && args.length === 3) {
            return { type: 'ternary', cond: args[0], ifTrue: args[1], ifFalse: args[2] }
          }
          return { type: 'call', name, args }
        }
      }
      return { type: 'var', name }
    }

    if (t.type === 'lparen') {
      consume()
      const inner = parseExpr()
      if (peek()?.type === 'rparen') consume()
      return inner
    }

    consume()
    return { type: 'literal', value: '' }
  }

  return parseExpr()
}

function evaluate(ast: AST, ctx: ExprContext): string | number | boolean {
  switch (ast.type) {
    case 'literal': return ast.value
    case 'var': {
      const v = ctx[ast.name]
      return v !== undefined ? v : 0
    }
    case 'unary': {
      const val = evaluate(ast.operand, ctx)
      if (ast.op === '-') return -(val as number)
      if (ast.op === '!') return !val
      return val
    }
    case 'binary': {
      const l = evaluate(ast.left, ctx) as number
      const r = evaluate(ast.right, ctx) as number
      switch (ast.op) {
        case '+': return l + r
        case '-': return l - r
        case '*': return l * r
        case '/': return r !== 0 ? l / r : 0
        case '>': return l > r
        case '<': return l < r
        case '>=': return l >= r
        case '<=': return l <= r
        case '==': return l === r
        case '!=': return l !== r
        case '&&': return !!l && !!r
        case '||': return !!l || !!r
        default: return 0
      }
    }
    case 'ternary': {
      const cond = evaluate(ast.cond, ctx)
      return cond ? evaluate(ast.ifTrue, ctx) : evaluate(ast.ifFalse, ctx)
    }
    case 'call': {
      if (ast.name === 'abs') return Math.abs(evaluate(ast.args[0], ctx) as number)
      return ''
    }
  }
}

export function evalExpression(expr: string, ctx: ExprContext): string | number | boolean {
  try {
    const tokens = tokenize(expr)
    const ast = parse(tokens)
    return evaluate(ast, ctx)
  } catch {
    return ''
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/scada2/engine/expression.ts
git commit -m "feat(scada2): add expression engine (AST-based evaluator)"
```

---

### Task 3: Build the symbol system

**Files:**
- Create: `client/src/scada2/symbols/index.ts`
- Create: `client/src/scada2/symbols/<name>.ts` (20 files)

Convert each existing SVG React component to a plain SVG string. The SVG strings are used by `fabric.loadSVGFromString()`. Use `currentColor` for stroke/fill so visual rules can override via Fabric.js `stroke`/`fill` on the group.

- [ ] **Step 1: Create symbols/index.ts with SymbolDef type and registry**

```ts
// client/src/scada2/symbols/index.ts
import type { ScadaEquipmentType } from '../../types'

export interface SymbolDef {
  svg: string
}

export { transformerSymbol } from './transformer'
export { meterSymbol } from './meter'
export { inverterSymbol } from './inverter'
export { solarPanelSymbol } from './solar_panel'
export { gatewaySymbol } from './gateway'
export { batterySymbol } from './battery'
export { circuitBreakerSymbol } from './circuit_breaker'
export { busbarSymbol } from './busbar'
export { isolatorSymbol } from './isolator'
export { earthSymbol } from './earth'
export { fuseSymbol } from './fuse'
export { capacitorSymbol } from './capacitor'
export { generatorSymbol } from './generator'
export { motorSymbol } from './motor'
export { surgeArresterSymbol } from './surge_arrester'
export { currentTransformerSymbol } from './current_transformer'
export { voltageTransformerSymbol } from './voltage_transformer'
export { contactorSymbol } from './contactor'
export { relaySymbol } from './relay'
export { loadSymbol } from './load'

import { transformerSymbol } from './transformer'
import { meterSymbol } from './meter'
import { inverterSymbol } from './inverter'
import { solarPanelSymbol } from './solar_panel'
import { gatewaySymbol } from './gateway'
import { batterySymbol } from './battery'
import { circuitBreakerSymbol } from './circuit_breaker'
import { busbarSymbol } from './busbar'
import { isolatorSymbol } from './isolator'
import { earthSymbol } from './earth'
import { fuseSymbol } from './fuse'
import { capacitorSymbol } from './capacitor'
import { generatorSymbol } from './generator'
import { motorSymbol } from './motor'
import { surgeArresterSymbol } from './surge_arrester'
import { currentTransformerSymbol } from './current_transformer'
import { voltageTransformerSymbol } from './voltage_transformer'
import { contactorSymbol } from './contactor'
import { relaySymbol } from './relay'
import { loadSymbol } from './load'

export const SYMBOLS: Record<ScadaEquipmentType, SymbolDef> = {
  transformer: transformerSymbol,
  meter: meterSymbol,
  inverter: inverterSymbol,
  solar_panel: solarPanelSymbol,
  gateway: gatewaySymbol,
  battery: batterySymbol,
  circuit_breaker: circuitBreakerSymbol,
  busbar: busbarSymbol,
  isolator: isolatorSymbol,
  earth: earthSymbol,
  fuse: fuseSymbol,
  capacitor: capacitorSymbol,
  generator: generatorSymbol,
  motor: motorSymbol,
  surge_arrester: surgeArresterSymbol,
  current_transformer: currentTransformerSymbol,
  voltage_transformer: voltageTransformerSymbol,
  contactor: contactorSymbol,
  relay: relaySymbol,
  load: loadSymbol,
}
```

- [ ] **Step 2: Create all 20 symbol files**

For each symbol, extract the inner SVG markup from the existing React component in `client/src/scada/assets/`, replace `{statusColor}` with `currentColor`, and convert JSX attributes to HTML attributes (`strokeWidth` -> `stroke-width`, `textAnchor` -> `text-anchor`, `fontWeight` -> `font-weight`, `fontSize` -> `font-size`).

Example pattern — each file follows this structure:

```ts
// client/src/scada2/symbols/<name>.ts
import type { SymbolDef } from './index'

export const <name>Symbol: SymbolDef = {
  svg: `<svg viewBox="0 0 W H" fill="none" stroke="currentColor" stroke-width="2">
    <!-- paths from existing React component -->
  </svg>`,
}
```

Concrete examples:

`client/src/scada2/symbols/transformer.ts`:
```ts
import type { SymbolDef } from './index'

export const transformerSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 90 80" fill="none" stroke="currentColor" stroke-width="2">
    <path d="M 25 15 Q 10 15, 10 25 Q 10 35, 25 35 Q 10 35, 10 45 Q 10 55, 25 55 Q 10 55, 10 65 Q 10 75, 25 75" />
    <path d="M 65 15 Q 80 15, 80 25 Q 80 35, 65 35 Q 80 35, 80 45 Q 80 55, 65 55 Q 80 55, 80 65 Q 80 75, 65 75" />
    <line x1="38" y1="10" x2="38" y2="80" stroke-width="3" />
    <line x1="52" y1="10" x2="52" y2="80" stroke-width="3" />
    <circle cx="5" cy="15" r="3" fill="none" stroke="currentColor" />
    <circle cx="5" cy="75" r="3" fill="none" stroke="currentColor" />
    <circle cx="85" cy="15" r="3" fill="none" stroke="currentColor" />
    <circle cx="85" cy="75" r="3" fill="none" stroke="currentColor" />
    <text x="20" y="8" font-size="7" fill="currentColor" stroke="none">P</text>
    <text x="65" y="8" font-size="7" fill="currentColor" stroke="none">S</text>
  </svg>`,
}
```

`client/src/scada2/symbols/meter.ts`:
```ts
import type { SymbolDef } from './index'

export const meterSymbol: SymbolDef = {
  svg: `<svg viewBox="0 0 60 60" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="30" cy="30" r="25" />
    <text x="30" y="36" text-anchor="middle" font-size="16" fill="currentColor" stroke="none" font-weight="bold">M</text>
    <line x1="0" y1="30" x2="5" y2="30" />
    <circle cx="2" cy="30" r="2" fill="currentColor" stroke="none" />
    <line x1="55" y1="30" x2="60" y2="30" />
    <circle cx="58" cy="30" r="2" fill="currentColor" stroke="none" />
  </svg>`,
}
```

`client/src/scada2/symbols/inverter.ts`:
```ts
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
```

Follow the same pattern for all 20 types. Read each existing file from `client/src/scada/assets/<Name>Svg.tsx`, extract the SVG markup, replace `{statusColor}` with `currentColor`, convert JSX attributes to HTML.

- [ ] **Step 3: Commit**

```bash
git add client/src/scada2/symbols/
git commit -m "feat(scada2): add symbol system with SVG strings for all 20 equipment types"
```

---

### Task 4: Create bindings resolver and render loop

**Files:**
- Create: `client/src/scada2/engine/bindings.ts`
- Create: `client/src/scada2/engine/render-loop.ts`

- [ ] **Step 1: Create bindings.ts**

```ts
// client/src/scada2/engine/bindings.ts
import type { SensorBinding, ExprContext } from './types'
import type { SensorConfig, SensorData } from '../../types'

export function resolveContext(
  bindings: SensorBinding[],
  currentValues: Record<string, SensorData>,
  sensors: SensorConfig[],
): ExprContext[] {
  return bindings.map(b => {
    const val = currentValues[String(b.sensor_id)]
    const cfg = sensors.find(s => s.id === b.sensor_id)
    const ctx: ExprContext = {
      value: val?.value ?? 0,
      warn: cfg?.warn_val ?? 0,
      crit: cfg?.crit_val ?? 0,
      min: cfg?.min_val ?? 0,
      max: cfg?.max_val ?? 0,
      status: val?.status ?? 'normal',
    }
    if (b.alias) {
      ctx[b.alias] = val?.value ?? 0
    }
    return ctx
  })
}

export function resolveTemplate(
  template: string,
  bindings: SensorBinding[],
  currentValues: Record<string, SensorData>,
  sensors: SensorConfig[],
): string {
  return template.replace(/\$\{([^}]+)\}/g, (_match, expr: string) => {
    const parts = expr.trim().split('.')
    if (parts.length === 2) {
      const [sensorRef, prop] = parts
      const binding = bindings.find(b => b.alias === sensorRef || String(b.sensor_id) === sensorRef)
      if (binding) {
        const cfg = sensors.find(s => s.id === binding.sensor_id)
        const val = currentValues[String(binding.sensor_id)]
        switch (prop) {
          case 'value': return val ? val.value.toFixed(cfg?.decimals ?? 1) : '--'
          case 'name': return cfg?.name ?? ''
          case 'unit': return cfg?.unit ?? ''
          case 'status': return val?.status ?? ''
        }
      }
    }
    // Simple ${value}, ${name}, ${unit} for single-binding labels
    if (bindings.length > 0) {
      const b = bindings[0]
      const cfg = sensors.find(s => s.id === b.sensor_id)
      const val = currentValues[String(b.sensor_id)]
      switch (expr.trim()) {
        case 'value': return val ? val.value.toFixed(cfg?.decimals ?? 1) : '--'
        case 'name': return cfg?.name ?? ''
        case 'unit': return cfg?.unit ?? ''
      }
    }
    return _match
  })
}
```

- [ ] **Step 2: Create render-loop.ts**

```ts
// client/src/scada2/engine/render-loop.ts
import { fabric } from 'fabric'
import { evalExpression } from './expression'
import { resolveContext, resolveTemplate } from './bindings'
import type { VisualRule } from './types'
import type { SensorConfig, SensorData } from '../../types'

export function startRenderLoop(
  canvas: fabric.Canvas,
  getState: () => { currentValues: Record<string, SensorData> },
  sensors: SensorConfig[],
): () => void {
  let rafId: number | null = null
  let lastValues: Record<string, SensorData> = {}

  function tick() {
    const { currentValues } = getState()
    const changed = currentValues !== lastValues
    lastValues = currentValues

    if (changed) {
      canvas.getObjects().forEach(obj => {
        const scada = (obj as any)._scada
        if (!scada) return

        // Evaluate visual rules
        if (scada.visual_rules?.length > 0 && scada.bindings?.length > 0) {
          const contexts = resolveContext(scada.bindings, currentValues, sensors)
          for (const rule of scada.visual_rules as VisualRule[]) {
            for (const ctx of contexts) {
              const result = evalExpression(rule.expression, ctx)
              applyRule(obj, rule.target, result)
            }
          }
        }

        // Update attached labels
        if (scada.attached_labels?.length > 0 && obj._labelObjects) {
          for (const labelObj of obj._labelObjects as fabric.Text[]) {
            const tmpl = (labelObj as any)._template
            if (tmpl) {
              labelObj.set('text', resolveTemplate(tmpl, scada.bindings || [], currentValues, sensors))
            }
          }
        }

        // Update standalone labels
        if (scada.template && (obj as any).type === 'ScadaLabel') {
          const text = resolveTemplate(scada.template, scada.bindings || [], currentValues, sensors)
          obj.set('text', text)
        }
      })
      canvas.renderAll()
    }

    rafId = requestAnimationFrame(tick)
  }

  rafId = requestAnimationFrame(tick)
  return () => { if (rafId !== null) cancelAnimationFrame(rafId) }
}

function applyRule(obj: fabric.Object, target: string, value: string | number | boolean) {
  switch (target) {
    case 'fill':
      obj.set('fill', value as string)
      if ((obj as any)._objects) {
        (obj as fabric.Group).getObjects().forEach((child: fabric.Object) => {
          if (child.type === 'path' || child.type === 'circle' || child.type === 'rect') {
            child.set('stroke', value as string)
          }
        })
      }
      break
    case 'stroke':
      obj.set('stroke', value as string)
      break
    case 'opacity':
      obj.set('opacity', value as number)
      break
    case 'visible':
      obj.set('visible', value as boolean)
      break
    case 'strokeWidth':
      obj.set('strokeWidth', value as number)
      break
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add client/src/scada2/engine/bindings.ts client/src/scada2/engine/render-loop.ts
git commit -m "feat(scada2): add bindings resolver and live render loop"
```

---

### Task 5: Create serializer

**Files:**
- Create: `client/src/scada2/engine/serializer.ts`

Maps between Fabric.js canvas JSON and the existing API format (`ScadaEquipmentItem` / `ScadaConnectionItem`).

- [ ] **Step 1: Create serializer.ts**

```ts
// client/src/scada2/engine/serializer.ts
import { fabric } from 'fabric'
import type { ScadaEquipmentItem, ScadaConnectionItem } from '../../types'
import type { ScadaEquipmentProps, ScadaConnectionProps } from './types'
import { DEFAULT_SIZES } from '../defaults'
import { createEquipmentObject } from '../objects/ScadaEquipment'
import { createConnectionObject } from '../objects/ScadaConnection'

export async function loadFromAPI(
  canvas: fabric.Canvas,
  equipment: ScadaEquipmentItem[],
  connections: ScadaConnectionItem[],
): Promise<void> {
  // Clear only non-grid objects
  const toRemove = canvas.getObjects().filter(o => (o as any)._scada)
  toRemove.forEach(o => canvas.remove(o))

  for (const eq of equipment) {
    const obj = await createEquipmentObject(eq.equipment_type, {
      id: eq.equipment_id,
      name: eq.equipment_name,
      x: eq.x,
      y: eq.y,
      width: eq.width,
      height: eq.height,
      rotation: eq.rotation,
      bindings: JSON.parse(eq.sensor_bindings || '[]').map((id: number) => ({ sensor_id: id })),
    })
    canvas.add(obj)
  }

  for (const conn of connections) {
    const points: [number, number][] = JSON.parse(conn.waypoints || '[]')
    const obj = createConnectionObject({
      id: conn.connection_id,
      from_id: conn.from_equipment,
      to_id: conn.to_equipment,
      connection_type: conn.connection_type,
      points,
      visual_rules: [],
      attached_labels: [],
    }, canvas)
    if (obj) canvas.add(obj)
  }

  canvas.renderAll()
}

export function exportToAPI(canvas: fabric.Canvas): {
  equipment: ScadaEquipmentItem[]
  connections: ScadaConnectionItem[]
} {
  const equipment: ScadaEquipmentItem[] = []
  const connections: ScadaConnectionItem[] = []

  canvas.getObjects().forEach(obj => {
    const scada = (obj as any)._scada
    if (!scada) return

    if (scada.type === 'ScadaEquipment') {
      const props = scada as ScadaEquipmentProps
      equipment.push({
        equipment_id: props.id,
        equipment_type: props.equipment_type,
        equipment_name: props.name,
        x: Math.round((obj.left ?? 0) * 100) / 100,
        y: Math.round((obj.top ?? 0) * 100) / 100,
        width: Math.round((obj.width ?? DEFAULT_SIZES[props.equipment_type].width) * (obj.scaleX ?? 1)),
        height: Math.round((obj.height ?? DEFAULT_SIZES[props.equipment_type].height) * (obj.scaleY ?? 1)),
        rotation: Math.round((obj.angle ?? 0) * 10) / 10,
        sensor_bindings: JSON.stringify(props.bindings.map((b: { sensor_id: number }) => b.sensor_id)),
      })
    }

    if (scada.type === 'ScadaConnection') {
      const props = scada as ScadaConnectionProps
      const polyline = obj as fabric.Polyline
      const pts = polyline.points?.map(p => [Math.round(p.x), Math.round(p.y)] as [number, number]) ?? []
      connections.push({
        connection_id: props.id,
        from_equipment: props.from_id,
        to_equipment: props.to_id,
        waypoints: JSON.stringify(pts),
        connection_type: props.connection_type,
      })
    }
  })

  return { equipment, connections }
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/scada2/engine/serializer.ts
git commit -m "feat(scada2): add serializer for API <-> canvas conversion"
```

---

### Task 6: Create Fabric.js custom objects

**Files:**
- Create: `client/src/scada2/objects/ScadaEquipment.ts`
- Create: `client/src/scada2/objects/ScadaConnection.ts`
- Create: `client/src/scada2/objects/ScadaLabel.ts`

- [ ] **Step 1: Create ScadaEquipment.ts**

```ts
// client/src/scada2/objects/ScadaEquipment.ts
import { fabric } from 'fabric'
import type { ScadaEquipmentType } from '../../types'
import type { ScadaEquipmentProps, PortDef } from '../engine/types'
import { SYMBOLS } from '../symbols'
import { DEFAULT_SIZES, PORT_POSITIONS } from '../defaults'

export async function createEquipmentObject(
  type: ScadaEquipmentType,
  overrides: Partial<ScadaEquipmentProps & { x: number; y: number; width: number; height: number; rotation: number }> = {},
): Promise<fabric.Group> {
  const symbol = SYMBOLS[type]
  const defaults = DEFAULT_SIZES[type]
  const ports = PORT_POSITIONS[type]

  const objects = await new Promise<fabric.Object[]>((resolve, reject) => {
    fabric.loadSVGFromString(symbol.svg, (objs) => {
      if (objs) resolve(objs)
      else reject(new Error(`Failed to load SVG for ${type}`))
    })
  })

  const group = fabric.util.groupSVGElements(objects, {
    width: overrides.width ?? defaults.width,
    height: overrides.height ?? defaults.height,
  })

  group.set({
    left: overrides.x ?? 100,
    top: overrides.y ?? 100,
    angle: overrides.rotation ?? 0,
    scaleX: (overrides.width ?? defaults.width) / (group.width || defaults.width),
    scaleY: (overrides.height ?? defaults.height) / (group.height || defaults.height),
    hasControls: true,
    hasBorders: true,
    borderColor: '#06b6d4',
    cornerColor: '#06b6d4',
    cornerSize: 8,
    transparentCorners: false,
  })

  const props: ScadaEquipmentProps & { type: string } = {
    type: 'ScadaEquipment',
    id: overrides.id ?? `scada_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
    equipment_type: type,
    name: overrides.name ?? type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
    bindings: overrides.bindings ?? [],
    visual_rules: [],
    attached_labels: [],
    ports,
  }
  ;(group as any)._scada = props

  return group
}
```

- [ ] **Step 2: Create ScadaConnection.ts**

```ts
// client/src/scada2/objects/ScadaConnection.ts
import { fabric } from 'fabric'
import type { ScadaConnectionProps } from '../engine/types'
import { CONNECTION_COLORS } from '../defaults'

export function createConnectionObject(
  props: ScadaConnectionProps,
  canvas: fabric.Canvas,
): fabric.Polyline | null {
  const fromObj = canvas.getObjects().find(o => (o as any)._scada?.id === props.from_id)
  const toObj = canvas.getObjects().find(o => (o as any)._scada?.id === props.to_id)
  if (!fromObj || !toObj) return null

  const fromCenter = fromObj.getCenterPoint()
  const toCenter = toObj.getCenterPoint()

  const points = props.points.length > 0
    ? props.points.map(p => new fabric.Point(p[0], p[1]))
    : [new fabric.Point(fromCenter.x, fromCenter.y), new fabric.Point(toCenter.x, toCenter.y)]

  const color = CONNECTION_COLORS[props.connection_type] ?? CONNECTION_COLORS.power

  const polyline = new fabric.Polyline(points, {
    fill: 'transparent',
    stroke: color,
    strokeWidth: props.connection_type === 'power' ? 3 : 2,
    strokeDashArray: props.connection_type === 'data' ? [8, 4] : undefined,
    strokeLineCap: 'round',
    strokeLineJoin: 'round',
    selectable: true,
    hasControls: true,
    hasBorders: true,
    borderColor: '#06b6d4',
    evented: true,
    objectCaching: false,
  })

  ;(polyline as any)._scada = { ...props, type: 'ScadaConnection' }

  return polyline
}

export function updateConnectionPoints(
  polyline: fabric.Polyline,
  canvas: fabric.Canvas,
): void {
  const scada = (polyline as any)._scada
  if (!scada) return

  const fromObj = canvas.getObjects().find(o => (o as any)._scada?.id === scada.from_id)
  const toObj = canvas.getObjects().find(o => (o as any)._scada?.id === scada.to_id)
  if (!fromObj || !toObj) return

  const fromCenter = fromObj.getCenterPoint()
  const toCenter = toObj.getCenterPoint()

  if (!scada.points || scada.points.length === 0) {
    polyline.set({ points: [fromCenter, toCenter] } as any)
  }
}
```

- [ ] **Step 3: Create ScadaLabel.ts**

```ts
// client/src/scada2/objects/ScadaLabel.ts
import { fabric } from 'fabric'
import type { ScadaLabelProps } from '../engine/types'

export function createLabelObject(
  props: Partial<ScadaLabelProps & { x: number; y: number }>,
): fabric.Text {
  const text = new fabric.Text(props.template ?? 'Label', {
    left: props.x ?? 100,
    top: props.y ?? 100,
    fontSize: 14,
    fill: '#ccc',
    fontFamily: 'monospace',
    selectable: true,
    hasControls: true,
    hasBorders: true,
    borderColor: '#06b6d4',
    cornerColor: '#06b6d4',
    cornerSize: 8,
    editable: true,
  })

  ;(text as any)._scada = {
    type: 'ScadaLabel',
    id: props.id ?? `label_${Date.now()}`,
    template: props.template ?? 'Label',
    bindings: props.bindings ?? [],
    visual_rules: props.visual_rules ?? [],
  }

  return text
}
```

- [ ] **Step 4: Commit**

```bash
git add client/src/scada2/objects/
git commit -m "feat(scada2): add Fabric.js custom objects for equipment, connection, label"
```

---

### Task 7: Create ScadaCanvas component

**Files:**
- Create: `client/src/scada2/ScadaCanvas.tsx`

- [ ] **Step 1: Create ScadaCanvas.tsx**

```tsx
// client/src/scada2/ScadaCanvas.tsx
import React, { useRef, useEffect } from 'react'
import { fabric } from 'fabric'
import type { SensorConfig, SensorData } from '../types'
import type { ScadaCanvasState } from './engine/types'
import { createEquipmentObject } from './objects/ScadaEquipment'
import { updateConnectionPoints } from './objects/ScadaConnection'
import { createLabelObject } from './objects/ScadaLabel'
import { startRenderLoop } from './engine/render-loop'

interface Props {
  sensors: SensorConfig[]
  getState: () => { currentValues: Record<string, SensorData> }
  canvasState: ScadaCanvasState
  onStateChange: (update: Partial<ScadaCanvasState>) => void
  onSaveNeeded: () => void
  onSensorClick: (sensorId: number) => void
}

export const ScadaCanvas: React.FC<Props> = ({
  sensors, getState, canvasState, onStateChange, onSaveNeeded, onSensorClick,
}) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<fabric.Canvas | null>(null)
  const stateRef = useRef(canvasState)
  stateRef.current = canvasState

  // Initialize canvas
  useEffect(() => {
    if (!containerRef.current) return
    const el = containerRef.current.querySelector('canvas')!
    const canvas = new fabric.Canvas(el, {
      width: containerRef.current.clientWidth,
      height: containerRef.current.clientHeight || 600,
      backgroundColor: '#060e09',
      selection: true,
      preserveObjectStacking: true,
    })
    canvasRef.current = canvas

    // Grid
    const gridSize = 20
    for (let x = 0; x < 3000; x += gridSize) {
      canvas.add(new fabric.Line([x, 0, x, 2000], {
        stroke: '#0e1f16', strokeWidth: 0.5, selectable: false, evented: false, excludeFromExport: true,
      }))
    }
    for (let y = 0; y < 2000; y += gridSize) {
      canvas.add(new fabric.Line([0, y, 3000, y], {
        stroke: '#0e1f16', strokeWidth: 0.5, selectable: false, evented: false, excludeFromExport: true,
      }))
    }

    const stopLoop = startRenderLoop(canvas, getState, sensors)

    const ro = new ResizeObserver(() => {
      if (containerRef.current) {
        canvas.setDimensions({
          width: containerRef.current.clientWidth,
          height: containerRef.current.clientHeight,
        })
      }
    })
    ro.observe(containerRef.current)

    ;(window as any).__scadaCanvas = canvas

    return () => {
      stopLoop()
      ro.disconnect()
      canvas.dispose()
      canvasRef.current = null
      ;(window as any).__scadaCanvas = undefined
    }
  }, [])

  // Tool handling
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    canvas.isDrawingMode = false
    canvas.selection = canvasState.activeTool === 'select'

    canvas.getObjects().forEach(obj => {
      const scada = (obj as any)._scada
      if (!scada) return
      obj.set({
        selectable: canvasState.editMode,
        evented: canvasState.editMode || scada.type === 'ScadaEquipment',
        hasControls: canvasState.editMode && canvasState.activeTool === 'select',
      })
    })
    canvas.renderAll()
  }, [canvasState.editMode, canvasState.activeTool])

  // Object selection
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const onSelect = (e: fabric.IEvent) => {
      const target = e.selected?.[0]
      if (target && (target as any)._scada) {
        onStateChange({ selectedId: (target as any)._scada.id })
      } else {
        onStateChange({ selectedId: null })
      }
    }
    canvas.on('selection:created', onSelect)
    canvas.on('selection:updated', onSelect)
    canvas.on('selection:cleared', () => onStateChange({ selectedId: null }))
    return () => {
      canvas.off('selection:created', onSelect)
      canvas.off('selection:updated', onSelect)
      canvas.off('selection:cleared')
    }
  }, [onStateChange])

  // Object modified (drag/resize/rotate)
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const onModified = () => { onSaveNeeded() }
    const onMoving = () => {
      canvas?.getObjects().forEach(obj => {
        if ((obj as any)._scada?.type === 'ScadaConnection') {
          updateConnectionPoints(obj as fabric.Polyline, canvas!)
        }
      })
    }
    canvas.on('object:modified', onModified)
    canvas.on('object:moving', onMoving)
    return () => {
      canvas.off('object:modified', onModified)
      canvas.off('object:moving', onMoving)
    }
  }, [onSaveNeeded])

  // Click to add equipment or label
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !canvasState.editMode) return

    const onMouseDown = (opt: fabric.IEvent) => {
      if (opt.target) return
      const state = stateRef.current

      if (state.pendingType) {
        const pointer = canvas.getScenePoint(opt.e)
        createEquipmentObject(state.pendingType, { x: pointer.x, y: pointer.y }).then(obj => {
          canvas.add(obj)
          canvas.setActiveObject(obj)
          canvas.renderAll()
          onStateChange({ pendingType: null, activeTool: 'select', selectedId: (obj as any)._scada.id })
          onSaveNeeded()
        })
      } else if (state.activeTool === 'label') {
        const pointer = canvas.getScenePoint(opt.e)
        const label = createLabelObject({ x: pointer.x, y: pointer.y, template: 'New Label' })
        canvas.add(label)
        canvas.setActiveObject(label)
        canvas.renderAll()
        onStateChange({ selectedId: (label as any)._scada.id })
        onSaveNeeded()
      }
    }
    canvas.on('mouse:down', onMouseDown)
    return () => { canvas.off('mouse:down', onMouseDown) }
  }, [canvasState.editMode, onStateChange, onSaveNeeded])

  // Zoom on wheel
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const onWheel = (opt: fabric.IEvent) => {
      const delta = (opt.e as WheelEvent).deltaY
      let zoom = canvas.getZoom()
      zoom *= 0.999 ** delta
      zoom = Math.max(0.1, Math.min(5, zoom))
      canvas.zoomToPoint(new fabric.Point((opt.e as MouseEvent).offsetX, (opt.e as MouseEvent).offsetY), zoom)
      opt.e.preventDefault()
      opt.e.stopPropagation()
    }
    canvas.on('mouse:wheel', onWheel)
    return () => { canvas.off('mouse:wheel', onWheel) }
  }, [])

  // Pan with middle mouse or alt+drag
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    let isPanning = false
    let lastPosX = 0
    let lastPosY = 0

    const onMouseDown = (opt: fabric.IEvent) => {
      const e = opt.e as MouseEvent
      if (e.button === 1 || (e.button === 0 && e.altKey)) {
        isPanning = true
        lastPosX = e.clientX
        lastPosY = e.clientY
        canvas.selection = false
      }
    }
    const onMouseMove = (opt: fabric.IEvent) => {
      if (!isPanning) return
      const e = opt.e as MouseEvent
      const vpt = canvas.viewportTransform!
      vpt[4] += e.clientX - lastPosX
      vpt[5] += e.clientY - lastPosY
      lastPosX = e.clientX
      lastPosY = e.clientY
      canvas.requestRenderAll()
    }
    const onMouseUp = () => {
      isPanning = false
      canvas.selection = stateRef.current.activeTool === 'select'
    }

    canvas.on('mouse:down', onMouseDown)
    canvas.on('mouse:move', onMouseMove)
    canvas.on('mouse:up', onMouseUp)
    return () => {
      canvas.off('mouse:down', onMouseDown)
      canvas.off('mouse:move', onMouseMove)
      canvas.off('mouse:up', onMouseUp)
    }
  }, [])

  return (
    <div style={{ position: 'relative', width: '100%', height: 'calc(100vh - 180px)', background: '#060e09', border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden' }}>
      <div ref={containerRef} style={{ width: '100%', height: '100%' }}>
        <canvas />
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/scada2/ScadaCanvas.tsx
git commit -m "feat(scada2): add ScadaCanvas component with Fabric.js mount"
```

---

### Task 8: Create ScadaToolbar and ScadaPalette

**Files:**
- Create: `client/src/scada2/ScadaToolbar.tsx`
- Create: `client/src/scada2/ScadaPalette.tsx`

- [ ] **Step 1: Create ScadaToolbar.tsx**

```tsx
// client/src/scada2/ScadaToolbar.tsx
import React from 'react'
import type { ScadaTool } from './engine/types'

interface Props {
  editMode: boolean
  activeTool: ScadaTool
  onToggleEdit: () => void
  onToolChange: (tool: ScadaTool) => void
  onSave: () => void
  hasChanges: boolean
  onZoomIn: () => void
  onZoomOut: () => void
  onFitView: () => void
}

const btn = (active: boolean): React.CSSProperties => ({
  padding: '6px 12px', borderRadius: 4,
  border: `1px solid ${active ? 'var(--cyan)' : 'var(--border)'}`,
  background: active ? 'rgba(34,197,94,.1)' : 'var(--card)',
  color: active ? 'var(--cyan)' : 'var(--fg)',
  cursor: 'pointer', fontSize: 12, fontWeight: 500,
})

export const ScadaToolbar: React.FC<Props> = ({
  editMode, activeTool, onToggleEdit, onToolChange, onSave, hasChanges, onZoomIn, onZoomOut, onFitView,
}) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
    <button onClick={onToggleEdit} style={btn(editMode)}>
      {editMode ? 'Done' : 'Edit'}
    </button>
    {editMode && (
      <>
        <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
        <button style={btn(activeTool === 'select')} onClick={() => onToolChange('select')}>Select</button>
        <button style={btn(activeTool === 'connect')} onClick={() => onToolChange('connect')}>Connect</button>
        <button style={btn(activeTool === 'label')} onClick={() => onToolChange('label')}>Label</button>
        <button style={btn(activeTool === 'delete')} onClick={() => onToolChange('delete')}>Delete</button>
        <div style={{ width: 1, height: 20, background: 'var(--border)' }} />
        <button onClick={onSave} disabled={!hasChanges} style={{
          padding: '6px 14px', borderRadius: 4,
          border: `1px solid ${hasChanges ? 'var(--cyan)' : 'var(--border)'}`,
          background: hasChanges ? 'var(--cyan)' : 'var(--card)',
          color: hasChanges ? '#fff' : 'var(--muted)',
          cursor: hasChanges ? 'pointer' : 'not-allowed',
          fontSize: 12, fontWeight: 600,
        }}>Save Layout</button>
      </>
    )}
    <div style={{ flex: 1 }} />
    <button onClick={onZoomIn} style={btn(false)}>+</button>
    <button onClick={onZoomOut} style={btn(false)}>-</button>
    <button onClick={onFitView} style={btn(false)}>Fit</button>
  </div>
)
```

- [ ] **Step 2: Create ScadaPalette.tsx**

Uses DOMPurify (already installed) to sanitize SVG content before rendering.

```tsx
// client/src/scada2/ScadaPalette.tsx
import React from 'react'
import DOMPurify from 'dompurify'
import { SCADA_EQUIPMENT_TYPES, type ScadaEquipmentType } from '../types'
import { SYMBOLS } from './symbols'

interface Props {
  visible: boolean
  onSelect: (type: ScadaEquipmentType) => void
  selectedType: ScadaEquipmentType | null
}

export const ScadaPalette: React.FC<Props> = ({ visible, onSelect, selectedType }) => {
  if (!visible) return null
  return (
    <div style={{
      position: 'absolute', top: 8, left: 8, zIndex: 10,
      background: 'var(--card)', border: '1px solid var(--border)',
      borderRadius: 8, padding: 8, display: 'flex', gap: 6, flexWrap: 'wrap',
      maxWidth: 360, maxHeight: 200, overflowY: 'auto',
      boxShadow: '0 2px 8px rgba(0,0,0,.08)',
    }}>
      {SCADA_EQUIPMENT_TYPES.map(({ value, label }) => {
        const color = selectedType === value ? 'var(--cyan)' : '#999'
        const svgHtml = DOMPurify.sanitize(
          SYMBOLS[value].svg
            .replace(/currentColor/g, color)
            .replace(/width="[^"]*"/, 'width="28"')
            .replace(/height="[^"]*"/, 'height="20"'),
        )
        return (
          <div key={value} onClick={() => onSelect(value)} style={{
            display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
            padding: '4px 6px', borderRadius: 6, cursor: 'pointer', minWidth: 52,
            border: `1px solid ${selectedType === value ? 'var(--cyan)' : 'transparent'}`,
            background: selectedType === value ? 'rgba(34,197,94,.1)' : 'transparent',
          }}>
            <div style={{ width: 28, height: 20 }} dangerouslySetInnerHTML={{ __html: svgHtml }} />
            <span style={{ fontSize: 8, color: selectedType === value ? 'var(--cyan)' : 'var(--muted)', textAlign: 'center', lineHeight: 1.2 }}>{label}</span>
          </div>
        )
      })}
    </div>
  )
}
```

- [ ] **Step 3: Commit**

```bash
git add client/src/scada2/ScadaToolbar.tsx client/src/scada2/ScadaPalette.tsx
git commit -m "feat(scada2): add toolbar and palette components"
```

---

### Task 9: Create ScadaEditPanel

**Files:**
- Create: `client/src/scada2/ScadaEditPanel.tsx`

Property panel for the selected object. Supports equipment, connection, and label editing with binding picker, visual rules editor, and attached labels editor.

- [ ] **Step 1: Create ScadaEditPanel.tsx**

```tsx
// client/src/scada2/ScadaEditPanel.tsx
import React, { useState } from 'react'
import type { SensorConfig } from '../types'
import type { ScadaEquipmentProps, ScadaConnectionProps, ScadaLabelProps, VisualRule, AttachedLabel } from './engine/types'

type SelectedObject =
  | { kind: 'equipment'; props: ScadaEquipmentProps }
  | { kind: 'connection'; props: ScadaConnectionProps }
  | { kind: 'label'; props: ScadaLabelProps }

interface Props {
  selected: SelectedObject | null
  sensors: SensorConfig[]
  onUpdate: (updates: Record<string, any>) => void
  onClose: () => void
}

const inputStyle: React.CSSProperties = {
  width: '100%', padding: '6px 8px', borderRadius: 4,
  border: '1px solid var(--border)', background: 'var(--bg)',
  color: 'var(--fg)', fontSize: 13, boxSizing: 'border-box',
}

const labelStyle: React.CSSProperties = {
  fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase',
  letterSpacing: '.05em', display: 'block', marginBottom: 4,
}

export const ScadaEditPanel: React.FC<Props> = ({ selected, sensors, onUpdate, onClose }) => {
  if (!selected) return null

  return (
    <div style={{ position: 'absolute', top: 0, right: 0, bottom: 0, width: 300, background: 'var(--card)', borderLeft: '1px solid var(--border)', padding: 16, overflowY: 'auto', zIndex: 20 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <span style={{ fontWeight: 600, fontSize: 14 }}>
          {selected.kind === 'equipment' ? 'Equipment' : selected.kind === 'connection' ? 'Connection' : 'Label'}
        </span>
        <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18 }}>x</button>
      </div>
      {selected.kind === 'equipment' && <EquipmentEditor props={selected.props} sensors={sensors} onUpdate={onUpdate} />}
      {selected.kind === 'connection' && <ConnectionEditor props={selected.props} onUpdate={onUpdate} />}
      {selected.kind === 'label' && <LabelEditor props={selected.props} sensors={sensors} onUpdate={onUpdate} />}
    </div>
  )
}

function EquipmentEditor({ props, sensors, onUpdate }: { props: ScadaEquipmentProps; sensors: SensorConfig[]; onUpdate: (u: Record<string, any>) => void }) {
  const [bindingSearch, setBindingSearch] = useState('')
  const filteredSensors = bindingSearch
    ? sensors.filter(s => s.name.toLowerCase().includes(bindingSearch.toLowerCase()))
    : sensors

  const toggleBinding = (sensorId: number) => {
    const has = props.bindings.some(b => b.sensor_id === sensorId)
    const next = has
      ? props.bindings.filter(b => b.sensor_id !== sensorId)
      : [...props.bindings, { sensor_id: sensorId }]
    onUpdate({ bindings: next })
  }

  const addRule = () => {
    onUpdate({ visual_rules: [...props.visual_rules, { target: 'fill', expression: "if(value > warn, 'red', 'green')" }] })
  }
  const updateRule = (idx: number, patch: Partial<VisualRule>) => {
    const next = [...props.visual_rules]
    next[idx] = { ...next[idx], ...patch }
    onUpdate({ visual_rules: next })
  }
  const removeRule = (idx: number) => {
    onUpdate({ visual_rules: props.visual_rules.filter((_, i) => i !== idx) })
  }
  const addLabel = () => {
    onUpdate({ attached_labels: [...props.attached_labels, { template: '${name}: ${value} ${unit}', offset: [0, -20] as [number, number] }] })
  }
  const updateLabel = (idx: number, patch: Partial<AttachedLabel>) => {
    const next = [...props.attached_labels]
    next[idx] = { ...next[idx], ...patch }
    onUpdate({ attached_labels: next })
  }
  const removeLabel = (idx: number) => {
    onUpdate({ attached_labels: props.attached_labels.filter((_, i) => i !== idx) })
  }

  return (
    <>
      <label style={{ display: 'block', marginBottom: 12 }}>
        <span style={labelStyle}>Name</span>
        <input value={props.name} onChange={e => onUpdate({ name: e.target.value })} style={{ ...inputStyle, marginTop: 0 }} />
      </label>
      <div style={{ marginBottom: 12 }}>
        <span style={labelStyle}>Type</span>
        <div style={{ fontSize: 13, textTransform: 'capitalize' }}>{props.equipment_type.replace(/_/g, ' ')}</div>
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12, marginBottom: 12 }}>
        <span style={labelStyle}>Sensor Bindings ({props.bindings.length})</span>
        <input placeholder="Search sensors..." value={bindingSearch} onChange={e => setBindingSearch(e.target.value)} style={{ ...inputStyle, fontSize: 12 }} />
        <div style={{ maxHeight: 150, overflowY: 'auto', marginTop: 4 }}>
          {filteredSensors.map(s => {
            const bound = props.bindings.some(b => b.sensor_id === s.id)
            return (
              <div key={s.id} onClick={() => toggleBinding(s.id)} style={{ padding: '3px 6px', borderRadius: 4, cursor: 'pointer', fontSize: 12, display: 'flex', justifyContent: 'space-between', background: bound ? 'rgba(34,197,94,.1)' : 'transparent', marginBottom: 1 }}>
                <span style={{ color: bound ? 'var(--cyan)' : 'var(--fg)' }}>{s.name}</span>
                <span style={{ fontSize: 10, color: 'var(--muted)' }}>{s.unit}</span>
              </div>
            )
          })}
        </div>
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12, marginBottom: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={labelStyle}>Visual Rules</span>
          <button onClick={addRule} style={{ fontSize: 11, padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer' }}>+ Add</button>
        </div>
        {props.visual_rules.map((rule, i) => (
          <div key={i} style={{ marginBottom: 8, padding: 8, background: 'var(--bg)', borderRadius: 4 }}>
            <select value={rule.target} onChange={e => updateRule(i, { target: e.target.value as VisualRule['target'] })} style={{ ...inputStyle, fontSize: 11, marginBottom: 4 }}>
              <option value="fill">Fill</option>
              <option value="stroke">Stroke</option>
              <option value="opacity">Opacity</option>
              <option value="visible">Visible</option>
            </select>
            <input value={rule.expression} onChange={e => updateRule(i, { expression: e.target.value })} placeholder="if(value > 220, 'red', 'green')" style={{ ...inputStyle, fontSize: 11 }} />
            <button onClick={() => removeRule(i)} style={{ fontSize: 10, color: '#ef4444', background: 'none', border: 'none', cursor: 'pointer', marginTop: 2 }}>Remove</button>
          </div>
        ))}
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={labelStyle}>Attached Labels</span>
          <button onClick={addLabel} style={{ fontSize: 11, padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer' }}>+ Add</button>
        </div>
        {props.attached_labels.map((lbl, i) => (
          <div key={i} style={{ marginBottom: 8, padding: 8, background: 'var(--bg)', borderRadius: 4 }}>
            <input value={lbl.template} onChange={e => updateLabel(i, { template: e.target.value })} placeholder="${name}: ${value} ${unit}" style={{ ...inputStyle, fontSize: 11, marginBottom: 4 }} />
            <button onClick={() => removeLabel(i)} style={{ fontSize: 10, color: '#ef4444', background: 'none', border: 'none', cursor: 'pointer' }}>Remove</button>
          </div>
        ))}
      </div>
    </>
  )
}

function ConnectionEditor({ props, onUpdate }: { props: ScadaConnectionProps; onUpdate: (u: Record<string, any>) => void }) {
  return (
    <>
      <div style={{ marginBottom: 12 }}>
        <span style={labelStyle}>Type</span>
        <select value={props.connection_type} onChange={e => onUpdate({ connection_type: e.target.value })} style={inputStyle}>
          <option value="power">Power</option>
          <option value="data">Data</option>
          <option value="flow">Flow</option>
        </select>
      </div>
      <div style={{ fontSize: 12, color: 'var(--muted)' }}>
        From: {props.from_id}<br />To: {props.to_id}
      </div>
    </>
  )
}

function LabelEditor({ props, sensors, onUpdate }: { props: ScadaLabelProps; sensors: SensorConfig[]; onUpdate: (u: Record<string, any>) => void }) {
  const addRule = () => {
    onUpdate({ visual_rules: [...props.visual_rules, { target: 'fill', expression: "if(value > warn, 'red', 'green')" }] })
  }
  const updateRule = (idx: number, patch: Partial<VisualRule>) => {
    const next = [...props.visual_rules]
    next[idx] = { ...next[idx], ...patch }
    onUpdate({ visual_rules: next })
  }
  const removeRule = (idx: number) => {
    onUpdate({ visual_rules: props.visual_rules.filter((_, i) => i !== idx) })
  }

  return (
    <>
      <label style={{ display: 'block', marginBottom: 12 }}>
        <span style={labelStyle}>Template</span>
        <input value={props.template} onChange={e => onUpdate({ template: e.target.value })} style={{ ...inputStyle, marginTop: 0 }} placeholder="${name}: ${value} ${unit}" />
      </label>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12, marginBottom: 12 }}>
        <span style={labelStyle}>Sensor Bindings</span>
        {sensors.slice(0, 20).map(s => {
          const bound = props.bindings.some(b => b.sensor_id === s.id)
          return (
            <div key={s.id} onClick={() => {
              const next = props.bindings.some(b => b.sensor_id === s.id)
                ? props.bindings.filter(b => b.sensor_id !== s.id)
                : [...props.bindings, { sensor_id: s.id }]
              onUpdate({ bindings: next })
            }} style={{ padding: '3px 6px', borderRadius: 4, cursor: 'pointer', fontSize: 12, display: 'flex', justifyContent: 'space-between', background: bound ? 'rgba(34,197,94,.1)' : 'transparent', marginBottom: 1 }}>
              <span style={{ color: bound ? 'var(--cyan)' : 'var(--fg)' }}>{s.name}</span>
              <span style={{ fontSize: 10, color: 'var(--muted)' }}>{s.unit}</span>
            </div>
          )
        })}
      </div>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={labelStyle}>Visual Rules</span>
          <button onClick={addRule} style={{ fontSize: 11, padding: '2px 8px', borderRadius: 3, border: '1px solid var(--border)', background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer' }}>+ Add</button>
        </div>
        {props.visual_rules.map((rule, i) => (
          <div key={i} style={{ marginBottom: 8, padding: 8, background: 'var(--bg)', borderRadius: 4 }}>
            <select value={rule.target} onChange={e => updateRule(i, { target: e.target.value as VisualRule['target'] })} style={{ ...inputStyle, fontSize: 11, marginBottom: 4 }}>
              <option value="fill">Fill</option>
              <option value="stroke">Stroke</option>
              <option value="opacity">Opacity</option>
              <option value="visible">Visible</option>
              <option value="text">Text</option>
            </select>
            <input value={rule.expression} onChange={e => updateRule(i, { expression: e.target.value })} style={{ ...inputStyle, fontSize: 11 }} />
            <button onClick={() => removeRule(i)} style={{ fontSize: 10, color: '#ef4444', background: 'none', border: 'none', cursor: 'pointer', marginTop: 2 }}>Remove</button>
          </div>
        ))}
      </div>
    </>
  )
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/scada2/ScadaEditPanel.tsx
git commit -m "feat(scada2): add property edit panel with bindings and rules editor"
```

---

### Task 10: Create top-level ScadaView

**Files:**
- Create: `client/src/scada2/ScadaView.tsx`

This wires everything together. Same interface as the existing `ScadaView.tsx` so it's a drop-in replacement.

- [ ] **Step 1: Create ScadaView.tsx**

```tsx
// client/src/scada2/ScadaView.tsx
import React, { useState, useCallback, useRef, useEffect, useMemo } from 'react'
import { fabric } from 'fabric'
import type { ScadaEquipmentItem, ScadaConnectionItem, SensorConfig } from '../types'
import { useDashboardSelector } from '../store/dashboardStore'
import { ScadaCanvas } from './ScadaCanvas'
import { ScadaToolbar } from './ScadaToolbar'
import { ScadaPalette } from './ScadaPalette'
import { ScadaEditPanel } from './ScadaEditPanel'
import { loadFromAPI, exportToAPI } from './engine/serializer'
import type { ScadaCanvasState, ScadaEquipmentProps, ScadaConnectionProps, ScadaLabelProps } from './engine/types'

interface Props {
  sensors: SensorConfig[]
  activeScreen: string
  scadaEquipment: ScadaEquipmentItem[]
  scadaConnections: ScadaConnectionItem[]
  onSaveEquipment: (items: ScadaEquipmentItem[]) => void
  onSaveConnections: (items: ScadaConnectionItem[]) => void
  onSensorClick: (sensorId: number) => void
}

type SelectedObject =
  | { kind: 'equipment'; props: ScadaEquipmentProps }
  | { kind: 'connection'; props: ScadaConnectionProps }
  | { kind: 'label'; props: ScadaLabelProps }

export const ScadaView: React.FC<Props> = ({
  sensors, activeScreen, scadaEquipment: serverEquipment, scadaConnections: serverConnections,
  onSaveEquipment, onSaveConnections, onSensorClick,
}) => {
  const currentValues = useDashboardSelector(s => s.currentValues)
  const getState = useCallback(() => ({ currentValues }), [currentValues])

  const [editMode, setEditMode] = useState(false)
  const [canvasState, setCanvasState] = useState<ScadaCanvasState>({
    editMode: false,
    activeTool: 'select',
    selectedId: null,
    pendingType: null,
    connectFrom: null,
  })
  const dirtyRef = useRef(false)
  const loadedScreenRef = useRef<string | null>(null)

  const handleStateChange = useCallback((update: Partial<ScadaCanvasState>) => {
    setCanvasState(prev => ({ ...prev, ...update }))
  }, [])

  const handleSaveNeeded = useCallback(() => { dirtyRef.current = true }, [])

  // Load data when screen changes
  useEffect(() => {
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas || loadedScreenRef.current === activeScreen) return
    loadedScreenRef.current = activeScreen
    loadFromAPI(canvas, serverEquipment, serverConnections)
    dirtyRef.current = false
  }, [activeScreen, serverEquipment, serverConnections])

  // Save
  const handleSave = useCallback(() => {
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas) return
    const { equipment, connections } = exportToAPI(canvas)
    onSaveEquipment(equipment)
    onSaveConnections(connections)
    dirtyRef.current = false
  }, [onSaveEquipment, onSaveConnections])

  const handleToggleEdit = useCallback(() => {
    if (editMode) {
      setCanvasState(prev => ({ ...prev, activeTool: 'select', pendingType: null, connectFrom: null }))
    }
    setEditMode(prev => !prev)
    setCanvasState(prev => ({ ...prev, editMode: !prev.editMode }))
  }, [editMode])

  // Zoom controls
  const handleZoomIn = useCallback(() => {
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas) return
    let zoom = canvas.getZoom() * 1.2
    zoom = Math.min(5, zoom)
    canvas.zoomToPoint(new fabric.Point(canvas.getWidth() / 2, canvas.getHeight() / 2), zoom)
  }, [])

  const handleZoomOut = useCallback(() => {
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas) return
    let zoom = canvas.getZoom() / 1.2
    zoom = Math.max(0.1, zoom)
    canvas.zoomToPoint(new fabric.Point(canvas.getWidth() / 2, canvas.getHeight() / 2), zoom)
  }, [])

  const handleFitView = useCallback(() => {
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas) return
    const objects = canvas.getObjects().filter(o => (o as any)._scada)
    if (objects.length === 0) return
    const group = new fabric.Group(objects)
    const bounds = group.getBoundingRect()
    const scale = Math.min(canvas.getWidth() / (bounds.width + 100), canvas.getHeight() / (bounds.height + 100), 2)
    canvas.setViewportTransform([1, 0, 0, 1, 0, 0])
    canvas.zoomToPoint(new fabric.Point(bounds.left + bounds.width / 2, bounds.top + bounds.height / 2), scale)
    group.destroy()
    canvas.discardActiveObject()
  }, [])

  // Selected object for edit panel
  const selectedObject: SelectedObject | null = useMemo(() => {
    if (!canvasState.selectedId) return null
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas) return null
    const obj = canvas.getObjects().find(o => (o as any)._scada?.id === canvasState.selectedId)
    if (!obj) return null
    const scada = (obj as any)._scada
    if (scada.type === 'ScadaEquipment') return { kind: 'equipment', props: scada as ScadaEquipmentProps }
    if (scada.type === 'ScadaConnection') return { kind: 'connection', props: scada as ScadaConnectionProps }
    if (scada.type === 'ScadaLabel') return { kind: 'label', props: scada as ScadaLabelProps }
    return null
  }, [canvasState.selectedId])

  const handleEditUpdate = useCallback((updates: Record<string, any>) => {
    const canvas = (window as any).__scadaCanvas as fabric.Canvas | undefined
    if (!canvas || !canvasState.selectedId) return
    const obj = canvas.getObjects().find(o => (o as any)._scada?.id === canvasState.selectedId)
    if (!obj) return
    Object.assign((obj as any)._scada, updates)
    canvas.renderAll()
    dirtyRef.current = true
  }, [canvasState.selectedId])

  return (
    <div style={{ position: 'relative' }}>
      <ScadaToolbar
        editMode={editMode}
        activeTool={canvasState.activeTool}
        onToggleEdit={handleToggleEdit}
        onToolChange={tool => setCanvasState(prev => ({ ...prev, activeTool: tool }))}
        onSave={handleSave}
        hasChanges={dirtyRef.current}
        onZoomIn={handleZoomIn}
        onZoomOut={handleZoomOut}
        onFitView={handleFitView}
      />
      <div style={{ position: 'relative' }}>
        <ScadaPalette
          visible={editMode && canvasState.activeTool === 'select'}
          onSelect={type => setCanvasState(prev => ({ ...prev, pendingType: type }))}
          selectedType={canvasState.pendingType}
        />
        <ScadaCanvas
          sensors={sensors}
          getState={getState}
          canvasState={canvasState}
          onStateChange={handleStateChange}
          onSaveNeeded={handleSaveNeeded}
          onSensorClick={onSensorClick}
        />
        {editMode && selectedObject && (
          <ScadaEditPanel
            selected={selectedObject}
            sensors={sensors}
            onUpdate={handleEditUpdate}
            onClose={() => setCanvasState(prev => ({ ...prev, selectedId: null }))}
          />
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/scada2/ScadaView.tsx
git commit -m "feat(scada2): add top-level ScadaView wrapper"
```

---

### Task 11: Wire into App.tsx and verify build

**Files:**
- Modify: `client/src/App.tsx`

- [ ] **Step 1: Update App.tsx import**

In `client/src/App.tsx`, change the ScadaView import from the old `scada/` to the new `scada2/`:

Find:
```ts
import { ScadaView } from './scada/ScadaView'
```
Replace with:
```ts
import { ScadaView } from './scada2/ScadaView'
```

- [ ] **Step 2: Verify build compiles**

```bash
cd client && npx tsc --noEmit
```

Expected: No type errors (or fix any that appear)

- [ ] **Step 3: Commit**

```bash
git add client/src/App.tsx
git commit -m "feat(scada2): wire new SCADA view into App.tsx"
```

---

### Task 12: Remove old scada directory

**Files:**
- Delete: `client/src/scada/` (entire directory)

- [ ] **Step 1: Verify no other imports reference old scada/**

```bash
grep -r "from.*['\"].*scada/" client/src/ --include="*.ts" --include="*.tsx" | grep -v scada2 | grep -v scada\.ts
```

Expected: No results

- [ ] **Step 2: Remove old directory**

```bash
rm -rf client/src/scada/
```

- [ ] **Step 3: Commit**

```bash
git add -A client/src/scada/
git commit -m "chore: remove old SVG-based scada directory (replaced by scada2)"
```

---

### Task 13: Manual smoke test and fixes

- [ ] **Step 1: Start dev server**

```bash
cd client && npm run dev
```

- [ ] **Step 2: Test editing flow**

1. Navigate to a screen
2. Switch to SCADA view
3. Click "Edit" -> toolbar should show tools
4. Click palette item -> click canvas -> equipment should appear
5. Drag equipment -> should move
6. Resize via corner handles
7. Click equipment -> edit panel should show on right
8. Add sensor binding -> select a sensor
9. Add visual rule -> set expression
10. Click "Save Layout"

- [ ] **Step 3: Test live mode**

1. Switch back to view mode (click "Done")
2. Verify equipment shows bound sensor values
3. Verify visual rules change colors based on live data

- [ ] **Step 4: Fix any issues found during testing**

Make targeted fixes for any bugs discovered. Commit each fix.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "fix(scada2): smoke test fixes"
```
