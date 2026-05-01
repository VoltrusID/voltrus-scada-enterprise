use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Register(String),
    Func(String),
    Ident(String),
    StringLit(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
    Dot,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    And,
    Or,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => { i += 1; }
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            '.' => { tokens.push(Token::Dot); i += 1; }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Gte);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Lte);
                    i += 2;
                } else {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Eq);
                    i += 2;
                } else {
                    return Err(format!("Unexpected '=' at position {i}"));
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::And);
                    i += 2;
                } else {
                    return Err(format!("Unexpected '&' at position {i}"));
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token::Or);
                    i += 2;
                } else {
                    return Err(format!("Unexpected '|' at position {i}"));
                }
            }
            '"' => {
                let start = i + 1;
                let mut j = start;
                while j < chars.len() && chars[j] != '"' {
                    j += 1;
                }
                if j >= chars.len() {
                    return Err(format!("Unclosed string literal at position {i}"));
                }
                let s: String = chars[start..j].iter().collect();
                tokens.push(Token::StringLit(s));
                i = j + 1;
            }
            '{' => {
                let start = i + 1;
                let end = chars[i..].iter().position(|&c| c == '}')
                    .ok_or_else(|| format!("Unclosed '{{' at position {i}"))?;
                let name: String = chars[start..i + end].iter().collect();
                if name.is_empty() {
                    return Err(format!("Empty register name at position {i}"));
                }
                tokens.push(Token::Register(name));
                i += end + 1;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num: f64 = num_str.parse()
                    .map_err(|_| format!("Invalid number '{num_str}'"))?;
                tokens.push(Token::Number(num));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(name));
            }
            _ => return Err(format!("Unexpected character '{}' at position {i}", chars[i])),
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BinOp { Add, Sub, Mul, Div }

#[derive(Debug, Clone, Copy, PartialEq)]
enum CompareOp { Gt, Lt, Gte, Lte, Eq }

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    Register(String),
    StringLit(String),
    JsonPath(Vec<String>),
    Func(String, Vec<Expr>),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    UnaryMinus(Box<Expr>),
    Compare { op: CompareOp, left: Box<Expr>, right: Box<Expr> },
    LogicalAnd(Box<Expr>, Box<Expr>),
    LogicalOr(Box<Expr>, Box<Expr>),
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }

    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    fn parse(&mut self) -> Result<Expr, String> {
        let expr = self.parse_or()?;
        if self.pos < self.tokens.len() {
            return Err(format!("Unexpected token at position {}", self.pos));
        }
        Ok(expr)
    }

    // or -> and ( '||' and )*
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        loop {
            match self.peek() {
                Some(Token::Or) => {
                    self.advance();
                    let right = self.parse_and()?;
                    left = Expr::LogicalOr(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // and -> comparison ( '&&' comparison )*
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            match self.peek() {
                Some(Token::And) => {
                    self.advance();
                    let right = self.parse_comparison()?;
                    left = Expr::LogicalAnd(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // comparison -> expr ( ('>' | '<' | '>=' | '<=' | '==') expr )*
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_expr()?;
        loop {
            match self.peek() {
                Some(Token::Gt) | Some(Token::Lt) | Some(Token::Gte) | Some(Token::Lte) | Some(Token::Eq) => {
                    let op = match self.advance().unwrap() {
                        Token::Gt => CompareOp::Gt,
                        Token::Lt => CompareOp::Lt,
                        Token::Gte => CompareOp::Gte,
                        Token::Lte => CompareOp::Lte,
                        Token::Eq => CompareOp::Eq,
                        _ => unreachable!(),
                    };
                    let right = self.parse_expr()?;
                    left = Expr::Compare { op, left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // expr -> term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) | Some(Token::Minus) => {
                    let op = match self.advance().unwrap() {
                        Token::Plus => BinOp::Add,
                        Token::Minus => BinOp::Sub,
                        _ => unreachable!(),
                    };
                    let right = self.parse_term()?;
                    left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // term -> unary (('*' | '/') unary)*
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) | Some(Token::Slash) => {
                    let op = match self.advance().unwrap() {
                        Token::Star => BinOp::Mul,
                        Token::Slash => BinOp::Div,
                        _ => unreachable!(),
                    };
                    let right = self.parse_unary()?;
                    left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // unary -> '-' unary | atom
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryMinus(Box::new(expr)));
        }
        self.parse_atom()
    }

    // atom -> Number | Register | StringLit | Ident ('.' Ident)* | Ident '(' args ')' | '(' or ')'
    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => { self.advance(); Ok(Expr::Number(n)) }
            Some(Token::Register(name)) => { self.advance(); Ok(Expr::Register(name)) }
            Some(Token::StringLit(s)) => { self.advance(); Ok(Expr::StringLit(s)) }
            Some(Token::Func(name)) => {
                self.advance();
                self.expect_lparen(&name)?;
                let args = self.parse_args()?;
                Ok(Expr::Func(name, args))
            }
            Some(Token::Ident(name)) => {
                self.advance();
                // Check for JSON path: ident ('.' ident)+
                if matches!(self.peek(), Some(Token::Dot)) {
                    let mut path = vec![name];
                    while matches!(self.peek(), Some(Token::Dot)) {
                        self.advance();
                        match self.advance() {
                            Some(Token::Ident(part)) | Some(Token::Func(part)) => {
                                path.push(part);
                            }
                            _ => return Err(format!("Expected identifier after '.' in JSON path at position {}", self.pos)),
                        }
                    }
                    Ok(Expr::JsonPath(path))
                } else if matches!(self.peek(), Some(Token::LParen)) {
                    // Function call
                    self.advance();
                    let args = self.parse_args()?;
                    Ok(Expr::Func(name, args))
                } else {
                    // Just an identifier - treat as single-element JSON path for flexibility
                    Ok(Expr::JsonPath(vec![name]))
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_or()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("Expected ')'".into()),
                }
            }
            _ => Err(format!("Unexpected token at position {}", self.pos)),
        }
    }

    fn expect_lparen(&mut self, name: &str) -> Result<(), String> {
        match self.advance() {
            Some(Token::LParen) => Ok(()),
            _ => Err(format!("Expected '(' after function '{name}'")),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::RParen)) {
            self.advance();
            return Ok(args);
        }
        loop {
            args.push(self.parse_or()?);
            match self.advance() {
                Some(Token::Comma) => continue,
                Some(Token::RParen) => break,
                _ => return Err("Expected ',' or ')' in function arguments".into()),
            }
        }
        Ok(args)
    }
}

pub fn evaluate(input: &str, values: &HashMap<String, f64>) -> Result<f64, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;
    eval_expr(&expr, values)
}

fn eval_expr(expr: &Expr, values: &HashMap<String, f64>) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Register(name) => values.get(name).copied()
            .ok_or_else(|| format!("Unknown register: '{name}'")),
        Expr::StringLit(_) => Err("String literal not supported in numeric evaluation".into()),
        Expr::JsonPath(_) => Err("JSON path not supported in numeric evaluation".into()),
        Expr::Func(name, args) => {
            match name.as_str() {
                "sqrt" => {
                    if args.len() != 1 { return Err(format!("{name} expects 1 argument")); }
                    let v = eval_expr(&args[0], values)?;
                    Ok(v.sqrt())
                }
                "abs" => {
                    if args.len() != 1 { return Err(format!("{name} expects 1 argument")); }
                    let v = eval_expr(&args[0], values)?;
                    Ok(v.abs())
                }
                "round" => {
                    if args.len() != 1 { return Err(format!("{name} expects 1 argument")); }
                    let v = eval_expr(&args[0], values)?;
                    Ok(v.round())
                }
                "clamp" => {
                    if args.len() != 3 { return Err(format!("{name} expects 3 arguments")); }
                    let v = eval_expr(&args[0], values)?;
                    let min = eval_expr(&args[1], values)?;
                    let max = eval_expr(&args[2], values)?;
                    Ok(v.clamp(min, max))
                }
                "avg" => {
                    if args.is_empty() { return Err("avg expects at least 1 argument".into()); }
                    let sum: f64 = args.iter().map(|a| eval_expr(a, values)).collect::<Result<Vec<_>, _>>()?.iter().sum();
                    Ok(sum / args.len() as f64)
                }
                "if" => {
                    if args.len() != 3 { return Err("if expects 3 arguments".into()); }
                    let cond = eval_expr(&args[0], values)?;
                    if cond != 0.0 {
                        eval_expr(&args[1], values)
                    } else {
                        eval_expr(&args[2], values)
                    }
                }
                _ => Err(format!("Unknown function: '{name}'")),
            }
        }
        Expr::BinOp { op, left, right } => {
            let l = eval_expr(left, values)?;
            let r = eval_expr(right, values)?;
            Ok(match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => {
                    if r == 0.0 { return Err("Division by zero".into()); }
                    l / r
                }
            })
        }
        Expr::UnaryMinus(inner) => Ok(-eval_expr(inner, values)?),
        Expr::Compare { .. } => Err("Comparison not supported in numeric evaluation".into()),
        Expr::LogicalAnd(_, _) | Expr::LogicalOr(_, _) => Err("Logical operation not supported in numeric evaluation".into()),
    }
}

pub fn evaluate_json(input: &str, context: &Value) -> Result<Value, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;
    eval_expr_json(&expr, context)
}

fn eval_expr_json(expr: &Expr, context: &Value) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)))),
        Expr::StringLit(s) => Ok(Value::String(s.clone())),
        Expr::Register(name) => {
            resolve_json_path(context, &[name.clone()])
        }
        Expr::JsonPath(path) => resolve_json_path(context, path),
        Expr::Func(name, args) => {
            match name.as_str() {
                "sqrt" => {
                    if args.len() != 1 { return Err(format!("{name} expects 1 argument")); }
                    let v = to_f64(&eval_expr_json(&args[0], context)?)?;
                    Ok(Value::Number(serde_json::Number::from_f64(v.sqrt()).unwrap_or_else(|| serde_json::Number::from(0))))
                }
                "abs" => {
                    if args.len() != 1 { return Err(format!("{name} expects 1 argument")); }
                    let v = to_f64(&eval_expr_json(&args[0], context)?)?;
                    Ok(Value::Number(serde_json::Number::from_f64(v.abs()).unwrap_or_else(|| serde_json::Number::from(0))))
                }
                "round" => {
                    if args.len() != 1 { return Err(format!("{name} expects 1 argument")); }
                    let v = to_f64(&eval_expr_json(&args[0], context)?)?;
                    Ok(Value::Number(serde_json::Number::from_f64(v.round()).unwrap_or_else(|| serde_json::Number::from(0))))
                }
                "clamp" => {
                    if args.len() != 3 { return Err(format!("{name} expects 3 arguments")); }
                    let v = to_f64(&eval_expr_json(&args[0], context)?)?;
                    let min = to_f64(&eval_expr_json(&args[1], context)?)?;
                    let max = to_f64(&eval_expr_json(&args[2], context)?)?;
                    Ok(Value::Number(serde_json::Number::from_f64(v.clamp(min, max)).unwrap_or_else(|| serde_json::Number::from(0))))
                }
                "avg" => {
                    if args.is_empty() { return Err("avg expects at least 1 argument".into()); }
                    let sum: f64 = args.iter().map(|a| eval_expr_json(a, context).and_then(|v| to_f64(&v))).collect::<Result<Vec<_>, _>>()?.iter().sum();
                    Ok(Value::Number(serde_json::Number::from_f64(sum / args.len() as f64).unwrap_or_else(|| serde_json::Number::from(0))))
                }
                "if" => {
                    if args.len() != 3 { return Err("if expects 3 arguments".into()); }
                    let cond = to_bool(&eval_expr_json(&args[0], context)?);
                    if cond {
                        eval_expr_json(&args[1], context)
                    } else {
                        eval_expr_json(&args[2], context)
                    }
                }
                _ => Err(format!("Unknown function: '{name}'")),
            }
        }
        Expr::BinOp { op, left, right } => {
            let l = to_f64(&eval_expr_json(left, context)?)?;
            let r = to_f64(&eval_expr_json(right, context)?)?;
            let result = match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => {
                    if r == 0.0 { return Err("Division by zero".into()); }
                    l / r
                }
            };
            Ok(Value::Number(serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0))))
        }
        Expr::UnaryMinus(inner) => {
            let v = to_f64(&eval_expr_json(inner, context)?)?;
            Ok(Value::Number(serde_json::Number::from_f64(-v).unwrap_or_else(|| serde_json::Number::from(0))))
        }
        Expr::Compare { op, left, right } => {
            let l = eval_expr_json(left, context)?;
            let r = eval_expr_json(right, context)?;
            Ok(Value::Bool(compare_values(*op, &l, &r)))
        }
        Expr::LogicalAnd(left, right) => {
            let l = to_bool(&eval_expr_json(left, context)?);
            if !l {
                return Ok(Value::Bool(false));
            }
            let r = to_bool(&eval_expr_json(right, context)?);
            Ok(Value::Bool(r))
        }
        Expr::LogicalOr(left, right) => {
            let l = to_bool(&eval_expr_json(left, context)?);
            if l {
                return Ok(Value::Bool(true));
            }
            let r = to_bool(&eval_expr_json(right, context)?);
            Ok(Value::Bool(r))
        }
    }
}

fn resolve_json_path(context: &Value, path: &[String]) -> Result<Value, String> {
    let mut current = context;
    for segment in path {
        match current {
            Value::Object(map) => {
                current = map.get(segment)
                    .ok_or_else(|| format!("Path segment '{}' not found", segment))?;
            }
            _ => return Err(format!("Cannot resolve path segment '{}' in non-object", segment)),
        }
    }
    Ok(current.clone())
}

fn to_f64(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| "Invalid number".into()),
        Value::String(s) => s.parse().map_err(|_| format!("Cannot convert string '{}' to number", s)),
        _ => Err(format!("Cannot convert {:?} to number", value)),
    }
}

fn to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|v| v != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Null => false,
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn compare_values(op: CompareOp, left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => matches!(op, CompareOp::Eq),
        (Value::Null, _) | (_, Value::Null) => false,
        (Value::Number(ln), Value::Number(rn)) => {
            let l = ln.as_f64().unwrap_or(0.0);
            let r = rn.as_f64().unwrap_or(0.0);
            match op {
                CompareOp::Gt => l > r,
                CompareOp::Lt => l < r,
                CompareOp::Gte => l >= r,
                CompareOp::Lte => l <= r,
                CompareOp::Eq => (l - r).abs() < f64::EPSILON,
            }
        }
        (Value::String(ls), Value::String(rs)) => {
            match op {
                CompareOp::Gt => ls > rs,
                CompareOp::Lt => ls < rs,
                CompareOp::Gte => ls >= rs,
                CompareOp::Lte => ls <= rs,
                CompareOp::Eq => ls == rs,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vals() -> HashMap<String, f64> {
        HashMap::from([
            ("Voltage_L1".into(), 220.0),
            ("Voltage_L2".into(), 221.0),
            ("Voltage_L3".into(), 219.0),
            ("Active_Power".into(), 5000.0),
        ])
    }

    #[test]
    fn test_single_register() {
        assert_eq!(evaluate("{Voltage_L1}", &vals()).unwrap(), 220.0);
    }

    #[test]
    fn test_addition() {
        assert_eq!(evaluate("{Voltage_L1} + {Voltage_L2}", &vals()).unwrap(), 441.0);
    }

    #[test]
    fn test_multiplication_with_constant() {
        assert_eq!(evaluate("{Active_Power} * 0.001", &vals()).unwrap(), 5.0);
    }

    #[test]
    fn test_precedence() {
        assert_eq!(evaluate("{Voltage_L1} + {Voltage_L2} * 0.5", &vals()).unwrap(), 330.5);
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate("({Voltage_L1} + {Voltage_L2}) * 0.5", &vals()).unwrap(), 220.5);
    }

    #[test]
    fn test_subtraction_and_division() {
        assert_eq!(evaluate("({Active_Power} - {Voltage_L1}) / 2", &vals()).unwrap(), 2390.0);
    }

    #[test]
    fn test_three_registers() {
        assert_eq!(evaluate("{Voltage_L1} + {Voltage_L2} + {Voltage_L3}", &vals()).unwrap(), 660.0);
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(evaluate("-{Voltage_L1}", &vals()).unwrap(), -220.0);
    }

    #[test]
    fn test_unknown_register() {
        assert!(evaluate("{Unknown_Reg}", &vals()).is_err());
    }

    #[test]
    fn test_division_by_zero() {
        assert!(evaluate("{Voltage_L1} / 0", &vals()).is_err());
    }

    #[test]
    fn test_unclosed_brace() {
        assert!(tokenize("{Voltage_L1").is_err());
    }

    #[test]
    fn test_empty_expression() {
        assert!(evaluate("", &vals()).is_err());
    }

    #[test]
    fn test_invalid_character() {
        assert!(tokenize("{Voltage_L1} & {Voltage_L2}").is_err());
    }

    #[test]
    fn test_whitespace_ignored() {
        assert_eq!(
            evaluate("  {Voltage_L1}  +  {Voltage_L2}  ", &vals()).unwrap(),
            441.0
        );
    }

    #[test]
    fn test_sqrt() {
        let v = evaluate("sqrt(9)", &HashMap::new()).unwrap();
        assert!((v - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_sqrt_with_register() {
        let v = evaluate("sqrt({Active_Power})", &vals()).unwrap();
        assert!((v - 70.7107).abs() < 0.01);
    }

    #[test]
    fn test_sqrt_in_expression() {
        let v = evaluate("sqrt({Active_Power} * {Active_Power})", &vals()).unwrap();
        assert!((v - 5000.0).abs() < 0.1);
    }

    #[test]
    fn test_abs() {
        let v = evaluate("abs(-{Voltage_L1})", &vals()).unwrap();
        assert_eq!(v, 220.0);
    }

    #[test]
    fn test_unknown_function() {
        assert!(evaluate("foo(1)", &HashMap::new()).is_err());
    }

    #[test]
    fn test_sqrt_line_voltage() {
        let v = evaluate("sqrt(3) * {Voltage_L1}", &vals()).unwrap();
        assert!((v - 381.05).abs() < 0.5);
    }

    // New tests for JSON and extended features
    #[test]
    fn test_json_path() {
        let context = serde_json::json!({
            "payload": {
                "metrics": {
                    "temperature": 85.0
                }
            }
        });
        let result = evaluate_json("payload.metrics.temperature", &context).unwrap();
        assert_eq!(result, serde_json::json!(85.0));
    }

    #[test]
    fn test_comparison_gt() {
        let context = serde_json::json!({
            "payload": {
                "metrics": {
                    "temperature": 85.0
                }
            }
        });
        let result = evaluate_json("payload.metrics.temperature > 80", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_comparison_lt() {
        let context = serde_json::json!({
            "temp": 75.0
        });
        let result = evaluate_json("temp < 80", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_comparison_eq() {
        let context = serde_json::json!({
            "temp": 80.0
        });
        let result = evaluate_json("temp == 80", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_logical_and() {
        let context = serde_json::json!({
            "a": true,
            "b": true
        });
        let result = evaluate_json("a && b", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_logical_and_short_circuit() {
        let context = serde_json::json!({
            "a": false,
            "b": true
        });
        let result = evaluate_json("a && b", &context).unwrap();
        assert_eq!(result, serde_json::json!(false));
    }

    #[test]
    fn test_logical_or() {
        let context = serde_json::json!({
            "a": false,
            "b": true
        });
        let result = evaluate_json("a || b", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_logical_or_short_circuit() {
        let context = serde_json::json!({
            "a": true,
            "b": false
        });
        let result = evaluate_json("a || b", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_if_function() {
        let context = serde_json::json!({
            "payload": {
                "metrics": {
                    "temperature": 85.0
                }
            }
        });
        let result = evaluate_json("if(payload.metrics.temperature > 80, \"CRITICAL\", \"OK\")", &context).unwrap();
        assert_eq!(result, serde_json::json!("CRITICAL"));
    }

    #[test]
    fn test_if_function_false() {
        let context = serde_json::json!({
            "payload": {
                "metrics": {
                    "temperature": 75.0
                }
            }
        });
        let result = evaluate_json("if(payload.metrics.temperature > 80, \"CRITICAL\", \"OK\")", &context).unwrap();
        assert_eq!(result, serde_json::json!("OK"));
    }

    #[test]
    fn test_clamp_function() {
        let context = serde_json::json!({
            "val": 150.0
        });
        let result = evaluate_json("clamp(val, 0, 100)", &context).unwrap();
        assert_eq!(result, serde_json::json!(100.0));
    }

    #[test]
    fn test_avg_function() {
        let context = serde_json::json!({
            "a": 10.0,
            "b": 20.0,
            "c": 30.0
        });
        let result = evaluate_json("avg(a, b, c)", &context).unwrap();
        assert_eq!(result, serde_json::json!(20.0));
    }

    #[test]
    fn test_complex_combined_expression() {
        let context = serde_json::json!({
            "payload": {
                "metrics": {
                    "temperature": 85.0,
                    "pressure": 90.0
                }
            }
        });
        let expr = "payload.metrics.temperature > 80 && payload.metrics.pressure < 100";
        let result = evaluate_json(expr, &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_string_literal_comparison() {
        let context = serde_json::json!({
            "status": "CRITICAL"
        });
        let result = evaluate_json("status == \"CRITICAL\"", &context).unwrap();
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_gte_lte_operators() {
        let context = serde_json::json!({
            "temp": 80.0
        });
        assert_eq!(evaluate_json("temp >= 80", &context).unwrap(), serde_json::json!(true));
        assert_eq!(evaluate_json("temp <= 80", &context).unwrap(), serde_json::json!(true));
        assert_eq!(evaluate_json("temp >= 81", &context).unwrap(), serde_json::json!(false));
        assert_eq!(evaluate_json("temp <= 79", &context).unwrap(), serde_json::json!(false));
    }

    #[test]
    fn test_null_comparison() {
        let context = serde_json::json!({
            "a": null,
            "b": null,
            "c": 1.0
        });
        assert_eq!(evaluate_json("a == b", &context).unwrap(), serde_json::json!(true));
        assert_eq!(evaluate_json("a == c", &context).unwrap(), serde_json::json!(false));
        assert_eq!(evaluate_json("a > c", &context).unwrap(), serde_json::json!(false));
    }

    #[test]
    fn test_old_evaluate_still_works_with_functions() {
        let mut map = HashMap::new();
        map.insert("x".into(), 25.0);
        let v = evaluate("sqrt({x})", &map).unwrap();
        assert!((v - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_round_function() {
        let context = serde_json::json!({
            "val": 3.7
        });
        let result = evaluate_json("round(val)", &context).unwrap();
        assert_eq!(result, serde_json::json!(4.0));
    }

    #[test]
    fn test_old_sqrt_abs_in_json() {
        let context = serde_json::json!({
            "val": -9.0
        });
        let result = evaluate_json("sqrt(abs(val))", &context).unwrap();
        assert_eq!(result, serde_json::json!(3.0));
    }
}
