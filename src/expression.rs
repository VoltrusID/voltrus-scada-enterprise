use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Register(String),
    Func(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
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
                tokens.push(Token::Func(name));
            }
            _ => return Err(format!("Unexpected character '{}' at position {i}", chars[i])),
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    Register(String),
    Func(String, Box<Expr>),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    UnaryMinus(Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
enum BinOp { Add, Sub, Mul, Div }

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
        let expr = self.parse_expr()?;
        if self.pos < self.tokens.len() {
            return Err(format!("Unexpected token at position {}", self.pos));
        }
        Ok(expr)
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

    // atom -> Number | Register | Func '(' expr ')' | '(' expr ')'
    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => { self.advance(); Ok(Expr::Number(n)) }
            Some(Token::Register(name)) => { self.advance(); Ok(Expr::Register(name)) }
            Some(Token::Func(name)) => {
                self.advance();
                match self.advance() {
                    Some(Token::LParen) => {}
                    _ => return Err(format!("Expected '(' after function '{name}'")),
                }
                let arg = self.parse_expr()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(Expr::Func(name, Box::new(arg))),
                    _ => Err(format!("Expected ')' after function argument")),
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("Expected ')'".into()),
                }
            }
            _ => Err(format!("Unexpected token at position {}", self.pos)),
        }
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
        Expr::Func(name, arg) => {
            let v = eval_expr(arg, values)?;
            match name.as_str() {
                "sqrt" => Ok(v.sqrt()),
                "abs" => Ok(v.abs()),
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
        // Apparent power: sqrt(P^2) = P
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
        // Realistic: Vline = sqrt(3) * Vphase ≈ 1.732 * 220 = 381.05
        let v = evaluate("sqrt(3) * {Voltage_L1}", &vals()).unwrap();
        assert!((v - 381.05).abs() < 0.5);
    }
}
