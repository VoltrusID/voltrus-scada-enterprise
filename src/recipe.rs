use anyhow::{Context, Result};
use tracing;

use crate::models::Recipe;
use crate::storage::Storage;

/// Load a recipe's values to the target PLC/device.
/// Currently logs the write operations. Actual Modbus/OPC-UA writes
/// can be implemented when device connection pooling is available.
pub async fn load_recipe_to_plc(recipe: &Recipe, storage: &Storage) -> Result<()> {
    let values: std::collections::HashMap<String, f64> =
        serde_json::from_str(&recipe.values)
            .with_context(|| format!("Failed to parse recipe {} values JSON", recipe.id))?;

    tracing::info!(
        "Loading recipe '{}' (v{}) to device '{}': {} registers",
        recipe.name,
        recipe.version,
        recipe.device_id,
        values.len(),
    );

    for (register_name, value) in &values {
        tracing::info!(
            "  [DRY-RUN] Would write {} = {} to device {}",
            register_name,
            value,
            recipe.device_id,
        );
    }

    // Log the recipe load event for audit trail
    if let Err(e) = storage.log_audit_event(
        "recipe_load",
        "system",
        &format!("recipe:{}:device:{}", recipe.id, recipe.device_id),
        Some(&format!(
            "Loaded recipe '{}' v{} to device '{}'",
            recipe.name, recipe.version, recipe.device_id
        )),
    ).await {
        tracing::warn!("Failed to log recipe load audit event: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_recipe_values() {
        let json = r#"{"temp": 25.0, "pressure": 1.5}"#;
        let values: std::collections::HashMap<String, f64> =
            serde_json::from_str(json).unwrap();
        assert_eq!(values.len(), 2);
        assert!((values["temp"] - 25.0).abs() < 0.001);
        assert!((values["pressure"] - 1.5).abs() < 0.001);
    }
}
