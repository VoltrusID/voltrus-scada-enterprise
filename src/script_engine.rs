use std::time::Instant;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Validate JS syntax using QuickJS parsing. Returns None if valid.
pub fn validate_syntax(source: &str) -> Option<String> {
    if source.trim().is_empty() {
        return Some("Script source is empty".to_string());
    }
    let rt = match rquickjs::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Some(format!("Failed to create JS runtime: {}", e)),
    };
    let ctx = match rquickjs::Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(e) => return Some(format!("Failed to create JS context: {}", e)),
    };
    let source_owned = source.to_string();
    let result = ctx.with(|ctx| -> Option<String> {
        use rquickjs::CatchResultExt;
        let result = ctx.eval::<rquickjs::Value, _>(source_owned.as_bytes()).catch(&ctx);
        match result {
            Ok(_) => None,
            Err(rquickjs::CaughtError::Exception(exc)) => {
                Some(exc.message().unwrap_or_else(|| "Unknown syntax error".to_string()))
            }
            Err(rquickjs::CaughtError::Value(val)) => Some(format!("{:?}", val)),
            Err(rquickjs::CaughtError::Error(e)) => Some(format!("{}", e)),
        }
    });
    result
}

/// Execute a script with the given input and resource limits.
pub fn execute_script(
    source: &str,
    input: &Value,
    cpu_limit_ms: u64,
    memory_limit_bytes: u64,
) -> ScriptResult {
    let start = Instant::now();
    let input_json = input.to_string();
    let result = execute_with_timeout(source, &input_json, cpu_limit_ms, memory_limit_bytes);
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

fn execute_with_timeout(
    source: &str,
    input_json: &str,
    cpu_limit_ms: u64,
    memory_limit_bytes: u64,
) -> Result<String, String> {
    // Security pattern blocking
    if source.contains("require(") || source.contains("import ") {
        return Err("Scripts cannot use require() or import statements".to_string());
    }
    if source.contains("fetch(") || source.contains("XMLHttpRequest") {
        return Err("Scripts cannot make network requests".to_string());
    }
    if source.contains("process.") || source.contains("global.") {
        return Err("Scripts cannot access process or global objects".to_string());
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let source = source.to_string();
    let input_json = input_json.to_string();

    std::thread::spawn(move || {
        let result = run_script_in_quickjs(&source, &input_json, memory_limit_bytes, cpu_limit_ms);
        let _ = tx.send(result);
    });

    let timeout = std::time::Duration::from_millis(cpu_limit_ms);
    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            Err(format!("Script execution timed out after {}ms", cpu_limit_ms))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("Script execution thread panicked".to_string())
        }
    }
}

fn run_script_in_quickjs(
    source: &str,
    input_json: &str,
    memory_limit_bytes: u64,
    cpu_limit_ms: u64,
) -> Result<String, String> {
    use rquickjs::CatchResultExt;

    let rt = rquickjs::Runtime::new()
        .map_err(|e| format!("Failed to create JS runtime: {}", e))?;

    rt.set_memory_limit(memory_limit_bytes as usize);
    rt.set_max_stack_size(256 * 1024);
    rt.set_gc_threshold(256 * 1024);

    // CPU timeout via interrupt handler
    let start = Instant::now();
    let cpu_limit = cpu_limit_ms as u128;
    rt.set_interrupt_handler(Some(Box::new(move || {
        start.elapsed().as_millis() > cpu_limit
    })));

    let ctx = rquickjs::Context::full(&rt)
        .map_err(|e| format!("Failed to create JS context: {}", e))?;

    let source = source.to_string();
    let input_json = input_json.to_string();

    ctx.with(|ctx| -> Result<String, String> {
        let input_val = ctx
            .json_parse(input_json.as_bytes())
            .map_err(|e| format!("Failed to parse input JSON: {}", e))?;
        ctx.globals()
            .set("input", input_val)
            .map_err(|e| format!("Failed to set input global: {}", e))?;

        let wrapped_source = format!(
            "(function() {{\n{}\nif (typeof main === 'function') {{ return main(input); }}\nreturn undefined;\n}})()",
            source
        );

        let result = ctx.eval::<rquickjs::Value, _>(wrapped_source.as_bytes()).catch(&ctx);

        match result {
            Ok(val) => {
                if val.is_undefined() {
                    return Ok("{}".to_string());
                }
                match ctx.json_stringify(val) {
                    Ok(Some(s)) => Ok(s.to_string().map_err(|e| format!("{}", e))?),
                    Ok(None) => Ok("{}".to_string()),
                    Err(e) => Err(format!("Failed to serialize result: {}", e)),
                }
            }
            Err(rquickjs::CaughtError::Exception(exc)) => {
                let msg = exc.message().unwrap_or_else(|| "Unknown error".to_string());
                let mut err = format!("Script error: {}", msg);
                if let Some(stack) = exc.stack() {
                    let first_lines = stack.lines().take(3).collect::<Vec<_>>().join("\n");
                    err = format!("{}\n{}", err, first_lines);
                }
                Err(err)
            }
            Err(rquickjs::CaughtError::Value(val)) => Err(format!("Script threw: {:?}", val)),
            Err(rquickjs::CaughtError::Error(e)) => {
                let msg = format!("{}", e);
                if msg.contains("memory") || msg.contains("allocation") {
                    Err(format!(
                        "Script exceeded memory limit ({} bytes)",
                        memory_limit_bytes
                    ))
                } else {
                    Err(format!("Runtime error: {}", msg))
                }
            }
        }
    })
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

    #[test]
    fn test_execute_simple_return() {
        let source = "function main(input) { return { power: input.voltage * input.current }; }";
        let input = json!({"voltage": 220, "current": 5});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(result.success, "Expected success, got error: {:?}", result.error);
        let output: serde_json::Value = serde_json::from_str(result.output.unwrap().as_str()).unwrap();
        assert_eq!(output["power"], 1100);
    }

    #[test]
    fn test_execute_syntax_error() {
        let source = "function main( { return 1; }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Script error"));
    }

    #[test]
    fn test_execute_timeout() {
        let source = "while(true) {}";
        let input = json!({});
        let result = execute_script(source, &input, 200, 1048576);
        assert!(!result.success);
        let err = result.error.unwrap();
        assert!(err.contains("timed out") || err.contains("interrupt") || err.contains("Internal"),
            "Expected timeout error, got: {}", err);
    }

    #[test]
    fn test_execute_memory_limit() {
        let source = "var arr = []; while(true) { arr.push(new Array(1024)); }";
        let input = json!({});
        let result = execute_script(source, &input, 5000, 65536);
        assert!(!result.success);
    }

    #[test]
    fn test_execute_no_main_returns_empty() {
        let source = "var x = 1 + 1;";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(result.success);
        assert_eq!(result.output.unwrap(), "{}");
    }

    #[test]
    fn test_execute_blocks_process() {
        let source = "function main() { process.exit(1); }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("process"));
    }
}
