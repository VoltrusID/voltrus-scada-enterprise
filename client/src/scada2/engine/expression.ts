import type { ExprContext } from './types'

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

const enum Tk {
  Number,
  String,
  Ident,
  LParen,
  RParen,
  Comma,
  Plus,
  Minus,
  Star,
  Slash,
  Gt,
  Lt,
  Gte,
  Lte,
  Eq,
  Neq,
  And,
  Or,
  Not,
  Eof,
}

interface Token {
  t: Tk
  v: string
}

// ---------------------------------------------------------------------------
// AST node types
// ---------------------------------------------------------------------------

type AstNode =
  | { kind: 'num'; v: number }
  | { kind: 'str'; v: string }
  | { kind: 'bool'; v: boolean }
  | { kind: 'ident'; v: string }
  | { kind: 'unary'; op: '-' | '!'; arg: AstNode }
  | { kind: 'bin'; op: string; l: AstNode; r: AstNode }
  | { kind: 'ternary'; cond: AstNode; ifTrue: AstNode; ifFalse: AstNode }
  | { kind: 'call'; fn: string; args: AstNode[] }

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

function tokenize(src: string): Token[] {
  const tokens: Token[] = []
  let i = 0

  while (i < src.length) {
    const ch = src[i]

    // whitespace
    if (ch === ' ' || ch === '\t' || ch === '\r' || ch === '\n') {
      i++
      continue
    }

    // number
    if (ch >= '0' && ch <= '9' || (ch === '.' && i + 1 < src.length && src[i + 1] >= '0' && src[i + 1] <= '9')) {
      let num = ''
      while (i < src.length && ((src[i] >= '0' && src[i] <= '9') || src[i] === '.')) {
        num += src[i++]
      }
      tokens.push({ t: Tk.Number, v: num })
      continue
    }

    // single-quoted string
    if (ch === "'") {
      i++
      let s = ''
      while (i < src.length && src[i] !== "'") {
        s += src[i++]
      }
      if (i < src.length) i++ // skip closing quote
      tokens.push({ t: Tk.String, v: s })
      continue
    }

    // identifiers / keywords
    if ((ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch === '_') {
      let id = ''
      while (i < src.length && ((src[i] >= 'a' && src[i] <= 'z') || (src[i] >= 'A' && src[i] <= 'Z') || (src[i] >= '0' && src[i] <= '9') || src[i] === '_')) {
        id += src[i++]
      }
      tokens.push({ t: Tk.Ident, v: id })
      continue
    }

    // two-char operators
    if (i + 1 < src.length) {
      const two = src[i] + src[i + 1]
      if (two === '>=') { tokens.push({ t: Tk.Gte, v: two }); i += 2; continue }
      if (two === '<=') { tokens.push({ t: Tk.Lte, v: two }); i += 2; continue }
      if (two === '==') { tokens.push({ t: Tk.Eq, v: two }); i += 2; continue }
      if (two === '!=') { tokens.push({ t: Tk.Neq, v: two }); i += 2; continue }
      if (two === '&&') { tokens.push({ t: Tk.And, v: two }); i += 2; continue }
      if (two === '||') { tokens.push({ t: Tk.Or, v: two }); i += 2; continue }
    }

    // single-char operators / punctuation
    switch (ch) {
      case '(': tokens.push({ t: Tk.LParen, v: ch }); i++; break
      case ')': tokens.push({ t: Tk.RParen, v: ch }); i++; break
      case ',': tokens.push({ t: Tk.Comma, v: ch }); i++; break
      case '+': tokens.push({ t: Tk.Plus, v: ch }); i++; break
      case '-': tokens.push({ t: Tk.Minus, v: ch }); i++; break
      case '*': tokens.push({ t: Tk.Star, v: ch }); i++; break
      case '/': tokens.push({ t: Tk.Slash, v: ch }); i++; break
      case '>': tokens.push({ t: Tk.Gt, v: ch }); i++; break
      case '<': tokens.push({ t: Tk.Lt, v: ch }); i++; break
      case '!': tokens.push({ t: Tk.Not, v: ch }); i++; break
      default: i++ // skip unknown characters
    }
  }

  tokens.push({ t: Tk.Eof, v: '' })
  return tokens
}

// ---------------------------------------------------------------------------
// Recursive-descent parser
// ---------------------------------------------------------------------------

class Parser {
  private pos = 0

  constructor(private tokens: Token[]) {}

  private peek(): Token {
    return this.tokens[this.pos] ?? { t: Tk.Eof, v: '' }
  }

  private advance(): Token {
    const tk = this.peek()
    this.pos++
    return tk
  }

  private expect(t: Tk): Token {
    const tk = this.advance()
    if (tk.t !== t) return { t: Tk.Eof, v: '' }
    return tk
  }

  /** Entry point: handles if(cond, true, false) ternary or falls through to or */
  parse(): AstNode | null {
    return this.parseTernary()
  }

  private parseTernary(): AstNode | null {
    const tk = this.peek()
    if (tk.t === Tk.Ident && tk.v === 'if' && this.tokens[this.pos + 1]?.t === Tk.LParen) {
      this.advance() // consume 'if'
      this.advance() // consume '('
      const cond = this.parseOr()
      if (!cond) return null
      if (this.advance().t !== Tk.Comma) return null
      const ifTrue = this.parseOr()
      if (!ifTrue) return null
      if (this.advance().t !== Tk.Comma) return null
      const ifFalse = this.parseOr()
      if (!ifFalse) return null
      if (this.expect(Tk.RParen).t !== Tk.RParen) return null
      return { kind: 'ternary', cond, ifTrue, ifFalse }
    }
    return this.parseOr()
  }

  private parseOr(): AstNode | null {
    let left = this.parseAnd()
    if (!left) return null
    while (true) {
      const tk = this.peek()
      if (tk.t === Tk.Or || (tk.t === Tk.Ident && tk.v === 'or')) {
        this.advance()
        const right = this.parseAnd()
        if (!right) return null
        left = { kind: 'bin', op: '||', l: left, r: right }
      } else break
    }
    return left
  }

  private parseAnd(): AstNode | null {
    let left = this.parseComparison()
    if (!left) return null
    while (true) {
      const tk = this.peek()
      if (tk.t === Tk.And || (tk.t === Tk.Ident && tk.v === 'and')) {
        this.advance()
        const right = this.parseComparison()
        if (!right) return null
        left = { kind: 'bin', op: '&&', l: left, r: right }
      } else break
    }
    return left
  }

  private parseComparison(): AstNode | null {
    let left = this.parseAdd()
    if (!left) return null
    while (true) {
      const tk = this.peek()
      if (tk.t === Tk.Gt || tk.t === Tk.Lt || tk.t === Tk.Gte || tk.t === Tk.Lte || tk.t === Tk.Eq || tk.t === Tk.Neq) {
        const op = this.advance().v
        const right = this.parseAdd()
        if (!right) return null
        left = { kind: 'bin', op, l: left, r: right }
      } else break
    }
    return left
  }

  private parseAdd(): AstNode | null {
    let left = this.parseMul()
    if (!left) return null
    while (true) {
      const tk = this.peek()
      if (tk.t === Tk.Plus || tk.t === Tk.Minus) {
        const op = this.advance().v
        const right = this.parseMul()
        if (!right) return null
        left = { kind: 'bin', op, l: left, r: right }
      } else break
    }
    return left
  }

  private parseMul(): AstNode | null {
    let left = this.parseUnary()
    if (!left) return null
    while (true) {
      const tk = this.peek()
      if (tk.t === Tk.Star || tk.t === Tk.Slash) {
        const op = this.advance().v
        const right = this.parseUnary()
        if (!right) return null
        left = { kind: 'bin', op, l: left, r: right }
      } else break
    }
    return left
  }

  private parseUnary(): AstNode | null {
    const tk = this.peek()
    if (tk.t === Tk.Minus) {
      this.advance()
      const arg = this.parseUnary()
      if (!arg) return null
      return { kind: 'unary', op: '-', arg }
    }
    if (tk.t === Tk.Not || (tk.t === Tk.Ident && tk.v === 'not')) {
      this.advance()
      const arg = this.parseUnary()
      if (!arg) return null
      return { kind: 'unary', op: '!', arg }
    }
    return this.parsePrimary()
  }

  private parsePrimary(): AstNode | null {
    const tk = this.peek()

    // number
    if (tk.t === Tk.Number) {
      this.advance()
      return { kind: 'num', v: parseFloat(tk.v) }
    }

    // string
    if (tk.t === Tk.String) {
      this.advance()
      return { kind: 'str', v: tk.v }
    }

    // parenthesized expression
    if (tk.t === Tk.LParen) {
      this.advance()
      const inner = this.parseTernary()
      if (!inner) return null
      if (this.expect(Tk.RParen).t !== Tk.RParen) return null
      return inner
    }

    // identifier, boolean literals, or function call
    if (tk.t === Tk.Ident) {
      this.advance()
      const name = tk.v

      // boolean literals
      if (name === 'true') return { kind: 'bool', v: true }
      if (name === 'false') return { kind: 'bool', v: false }

      // function call: abs(...) or other identifier(...)
      if (this.peek().t === Tk.LParen) {
        this.advance() // consume '('
        const args: AstNode[] = []
        if (this.peek().t !== Tk.RParen) {
          const first = this.parseOr()
          if (!first) return null
          args.push(first)
          while (this.peek().t === Tk.Comma) {
            this.advance()
            const next = this.parseOr()
            if (!next) return null
            args.push(next)
          }
        }
        if (this.expect(Tk.RParen).t !== Tk.RParen) return null
        return { kind: 'call', fn: name, args }
      }

      return { kind: 'ident', v: name }
    }

    return null
  }
}

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

function evalNode(node: AstNode, ctx: ExprContext): string | number | boolean {
  switch (node.kind) {
    case 'num':
      return node.v
    case 'str':
      return node.v
    case 'bool':
      return node.v
    case 'ident': {
      const val = ctx[node.v]
      if (val === undefined) return 0
      return val
    }
    case 'unary': {
      const arg = evalNode(node.arg, ctx)
      if (node.op === '-') return -(arg as number)
      return !toBool(arg)
    }
    case 'bin': {
      const l = evalNode(node.l, ctx)
      const r = evalNode(node.r, ctx)
      return evalBinOp(node.op, l, r)
    }
    case 'ternary': {
      const cond = evalNode(node.cond, ctx)
      return toBool(cond) ? evalNode(node.ifTrue, ctx) : evalNode(node.ifFalse, ctx)
    }
    case 'call': {
      if (node.fn === 'abs') {
        if (node.args.length !== 1) return 0
        const v = evalNode(node.args[0], ctx)
        return Math.abs(v as number)
      }
      return 0
    }
  }
}

function evalBinOp(op: string, l: string | number | boolean, r: string | number | boolean): string | number | boolean {
  switch (op) {
    case '+': return (l as number) + (r as number)
    case '-': return (l as number) - (r as number)
    case '*': return (l as number) * (r as number)
    case '/': {
      const rv = r as number
      return rv === 0 ? 0 : (l as number) / rv
    }
    case '>':  return (l as number) > (r as number)
    case '<':  return (l as number) < (r as number)
    case '>=': return (l as number) >= (r as number)
    case '<=': return (l as number) <= (r as number)
    case '==': return l === r
    case '!=': return l !== r
    case '&&': return toBool(l) && toBool(r)
    case '||': return toBool(l) || toBool(r)
    default:   return 0
  }
}

function toBool(v: string | number | boolean): boolean {
  if (typeof v === 'boolean') return v
  if (typeof v === 'number') return v !== 0
  return v !== '' && v !== '0' && v !== 'false'
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Evaluate an expression string against a context object.
 *
 * Supports:
 * - Arithmetic: +, -, *, /
 * - Comparison: >, <, >=, <=, ==, !=
 * - Boolean: && (and), || (or), ! (not)
 * - Ternary: if(cond, ifTrue, ifFalse)
 * - Functions: abs(...)
 * - Variables: any key from the ExprContext
 * - Literals: numbers, single-quoted strings, true/false
 *
 * Returns '' (empty string) on parse or evaluation errors. Never throws.
 */
export function evalExpression(expr: string, ctx: ExprContext): string | number | boolean {
  try {
    const trimmed = expr.trim()
    if (!trimmed) return ''
    const tokens = tokenize(trimmed)
    const parser = new Parser(tokens)
    const ast = parser.parse()
    if (!ast) return ''
    return evalNode(ast, ctx)
  } catch {
    return ''
  }
}
