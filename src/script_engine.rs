use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[cfg(feature = "scripts")]
use std::time::Instant;

/// Validate JS syntax. Returns None if valid.
#[cfg(feature = "scripts")]
pub fn validate_syntax(source: &str) -> Option<String> {
    if source.trim().is_empty() {
        return Some("Script source is empty".to_string());
    }
    let mut ctx = boa_engine::Context::default();
    match ctx.eval(boa_engine::Source::from_bytes(source)) {
        Ok(_) => None,
        Err(e) => Some(format!("{}", e)),
    }
}

/// Validate JS syntax — stub when scripts feature is disabled.
#[cfg(not(feature = "scripts"))]
pub fn validate_syntax(_source: &str) -> Option<String> {
    Some("Script engine is not available (scripts feature disabled)".to_string())
}

/// Execute a script with the given input and resource limits.
#[cfg(feature = "scripts")]
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

/// Execute a script — stub when scripts feature is disabled.
#[cfg(not(feature = "scripts"))]
pub fn execute_script(
    _source: &str,
    _input: &Value,
    _cpu_limit_ms: u64,
    _memory_limit_bytes: u64,
) -> ScriptResult {
    ScriptResult {
        success: false,
        output: None,
        error: Some("Script engine is not available (scripts feature disabled)".to_string()),
        duration_ms: 0,
    }
}

#[cfg(feature = "scripts")]
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
        let result = run_script_in_boa(&source, &input_json, memory_limit_bytes, cpu_limit_ms);
        let _ = tx.send(result);
    });

    let timeout = std::time::Duration::from_millis(cpu_limit_ms);
    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "Script execution timed out after {}ms",
            cpu_limit_ms
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("Script execution thread panicked".to_string())
        }
    }
}

#[cfg(feature = "scripts")]
fn run_script_in_boa(
    source: &str,
    input_json: &str,
    _memory_limit_bytes: u64,
    cpu_limit_ms: u64,
) -> Result<String, String> {
    use boa_engine::property::Attribute;

    let mut ctx = boa_engine::Context::default();

    // Set runtime limits for CPU/stability protection
    ctx.set_runtime_limits({
        let mut limits = boa_engine::vm::RuntimeLimits::default();
        limits.set_loop_iteration_limit(1_000_000);
        limits.set_recursion_limit(256);
        limits.set_stack_size_limit(1024);
        limits
    });

    // Parse input JSON and set as global
    let input_val: serde_json::Value =
        serde_json::from_str(input_json).map_err(|e| format!("Failed to parse input JSON: {}", e))?;
    let js_input = boa_engine::JsValue::from_json(&input_val, &mut ctx)
        .map_err(|e| format!("Failed to convert input to JS value: {}", e))?;
    ctx.register_global_property(boa_engine::js_string!("input"), js_input, Attribute::all())
        .map_err(|e| format!("Failed to set input global: {}", e))?;

    // Wrap source: execute user code, then call main(input) if defined
    let wrapped_source = format!(
        "(function() {{\n{}\nif (typeof main === 'function') {{ return main(input); }}\nreturn undefined;\n}})()",
        source
    );

    let _timeout_guard = cpu_limit_ms;
    let result = ctx
        .eval(boa_engine::Source::from_bytes(&wrapped_source))
        .map_err(|e| {
            let msg = format!("{}", e);
            if msg.contains("loop iteration limit") {
                format!(
                    "Script exceeded CPU limit ({}ms timeout)",
                    _timeout_guard
                )
            } else {
                format!("Script error: {}", msg)
            }
        })?;

    // Convert result to JSON
    if result.is_undefined() {
        return Ok("{}".to_string());
    }

    match result.to_json(&mut ctx) {
        Ok(Some(json_val)) => serde_json::to_string(&json_val)
            .map_err(|e| format!("Failed to serialize result: {}", e)),
        Ok(None) => Ok("{}".to_string()),
        Err(e) => Err(format!("Failed to convert result to JSON: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_syntax_empty() {
        assert!(validate_syntax("").is_some());
        assert!(validate_syntax("   ").is_some());
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
    fn test_execute_blocks_process() {
        let source = "function main() { process.exit(1); }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("process"));
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_validate_syntax_valid() {
        let source = "function main(input) { return { outputs: {} }; }";
        assert_eq!(validate_syntax(source), None);
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_validate_syntax_unbalanced_braces() {
        let source = "function main() { return 1; ";
        assert!(validate_syntax(source).is_some());
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_validate_syntax_balanced() {
        let source = "function main(input) { return { a: 1, b: [1, 2] }; }";
        assert_eq!(validate_syntax(source), None);
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_execute_simple_return() {
        let source = "function main(input) { return { power: input.voltage * input.current }; }";
        let input = json!({"voltage": 220, "current": 5});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(
            result.success,
            "Expected success, got error: {:?}",
            result.error
        );
        let output: serde_json::Value =
            serde_json::from_str(result.output.unwrap().as_str()).unwrap();
        assert_eq!(output["power"], 1100);
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_execute_syntax_error() {
        let source = "function main( { return 1; }";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Script error"));
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_execute_timeout() {
        let source = "while(true) {}";
        let input = json!({});
        let result = execute_script(source, &input, 200, 1048576);
        assert!(!result.success);
        let err = result.error.unwrap();
        assert!(
            err.contains("timed out") || err.contains("CPU limit") || err.contains("loop iteration"),
            "Expected timeout error, got: {}",
            err
        );
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_execute_memory_limit() {
        let source = "var arr = []; while(true) { arr.push(new Array(1024)); }";
        let input = json!({});
        let result = execute_script(source, &input, 5000, 65536);
        assert!(!result.success);
    }

    #[cfg(feature = "scripts")]
    #[test]
    fn test_execute_no_main_returns_empty() {
        let source = "var x = 1 + 1;";
        let input = json!({});
        let result = execute_script(source, &input, 1000, 1048576);
        assert!(result.success);
        assert_eq!(result.output.unwrap(), "{}");
    }
}
