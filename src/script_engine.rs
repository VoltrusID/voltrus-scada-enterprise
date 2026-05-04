use std::time::Instant;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Validate JS syntax. Returns None if valid, Some(error_message) if invalid.
pub fn validate_syntax(source: &str) -> Option<String> {
    if source.trim().is_empty() {
        return Some("Script source is empty".to_string());
    }
    let mut brace_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut paren_depth = 0i32;
    for ch in source.chars() {
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }
    }
    if brace_depth != 0 {
        return Some(format!("Unbalanced braces: {} unclosed", brace_depth));
    }
    if bracket_depth != 0 {
        return Some(format!("Unbalanced brackets: {} unclosed", bracket_depth));
    }
    if paren_depth != 0 {
        return Some(format!("Unbalanced parentheses: {} unclosed", paren_depth));
    }
    None
}

/// Execute a script with the given input and resource limits.
pub fn execute_script(
    source: &str,
    input: &Value,
    cpu_limit_ms: u64,
    _memory_limit_bytes: u64,
) -> ScriptResult {
    let start = Instant::now();

    let input_json = input.to_string();
    let result = execute_with_timeout(source, &input_json, std::time::Duration::from_millis(cpu_limit_ms));

    let duration_ms = start.elapsed().as_millis() as u64;
    match result {
        Ok(output) => ScriptResult {
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
        },
        Err(e) => ScriptResult {
            success: false,
            output: None,
            error: Some(e),
            duration_ms,
        },
    }
}

fn execute_with_timeout(source: &str, _input_json: &str, _timeout: std::time::Duration) -> Result<String, String> {
    // Check for blocked patterns
    if source.contains("require(") || source.contains("import ") {
        return Err("Scripts cannot use require() or import statements".to_string());
    }
    if source.contains("fetch(") || source.contains("XMLHttpRequest") {
        return Err("Scripts cannot make network requests".to_string());
    }
    if source.contains("process.") || source.contains("global.") {
        return Err("Scripts cannot access process or global objects".to_string());
    }

    // TODO: Full QuickJS-WASM execution via wasmtime
    Err("Script engine QuickJS-WASM runtime not yet initialized".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_syntax_valid() {
        let source = "function main(input) { return { outputs: {} }; }";
        assert_eq!(validate_syntax(source), None);
    }

    #[test]
    fn test_validate_syntax_empty() {
        assert!(validate_syntax("").is_some());
        assert!(validate_syntax("   ").is_some());
    }

    #[test]
    fn test_validate_syntax_unbalanced_braces() {
        let source = "function main() { return 1; ";
        assert!(validate_syntax(source).is_some());
    }

    #[test]
    fn test_validate_syntax_balanced() {
        let source = "function main(input) { return { a: 1, b: [1, 2] }; }";
        assert_eq!(validate_syntax(source), None);
    }

    #[test]
    fn test_execute_blocks_require() {
        let source = "function main() { require('fs'); }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("require"));
    }

    #[test]
    fn test_execute_blocks_fetch() {
        let source = "function main() { fetch('http://evil.com'); }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("network"));
    }
}
