use super::Storage;
use anyhow::Result;

impl Storage {
    // ── Seed ──

    pub async fn seed_database(&self) -> Result<()> {
        let existing = self.get_user_by_username("admin").await?;
        if existing.is_none() {
            use rand::Rng;
            let password: String = rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            let password_hash = crate::api::auth::hash_password(&password);

            self.create_user("admin", &password_hash, "", "admin").await?;
            self.set_must_change_password("admin", true).await?;

            tracing::warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            tracing::warn!("  GENERATED ADMIN PASSWORD: {}", password);
            tracing::warn!("  You will be required to change this on first login.");
            tracing::warn!("  Save this password now — it will NOT be shown again.");
            tracing::warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        // Seed default RBAC roles if none exist
        if let Err(e) = self.seed_default_roles().await {
            tracing::warn!("Failed to seed default roles: {}", e);
        }

        // Seed PLTS data sources if none exist
        let all_data_sources = self.list_data_sources(None).await?;
        if all_data_sources.is_empty() {
            self.seed_plts_data_sources().await?;
        }

        Ok(())
    }

    pub(crate) async fn seed_plts_data_sources(&self) -> Result<()> {
        // Seed devices
        if !self.device_exists("plts_1").await? {
            self.create_device("plts_1", "PLTS 1", 0).await?;
        }
        if !self.device_exists("plts_2").await? {
            self.create_device("plts_2", "PLTS 2", 1).await?;
        }

        // Data source definitions matching Modbus register names from config.yaml
        // (key, name, unit, color, min, max, warn, crit, base, sort, decimals, source_type, source_config)
        let data_source_defs: &[(&str, &str, &str, &str, f64, f64, f64, f64, f64, i64, i64, &str, &str)] = &[
            ("voltage", "Voltage", "V", "#f59e0b", 0.0, 450.0, 400.0, 420.0, 380.0, 0, 1, "modbus", "{}"),
            ("current", "Current", "A", "#06b6d4", 0.0, 100.0, 85.0, 95.0, 65.0, 1, 2, "modbus", "{}"),
            ("power", "Power", "W", "#a855f7", 0.0, 50000.0, 40000.0, 45000.0, 24700.0, 2, 0, "modbus", "{}"),
            ("frequency", "Frequency", "Hz", "#22c55e", 45.0, 55.0, 51.0, 52.0, 50.0, 3, 2, "modbus", "{}"),
            ("power_factor", "Power Factor", "", "#ef4444", 0.0, 1.0, 0.95, 0.98, 0.92, 4, 3, "modbus", "{}"),
            ("energy_total", "Energy Total", "kWh", "#8b5cf6", 0.0, 999999.0, 800000.0, 900000.0, 158732.0, 5, 1, "modbus", "{}"),
        ];

        let mut plts1_data_source_ids: Vec<i64> = Vec::new();
        let mut plts2_data_source_ids: Vec<i64> = Vec::new();

        for (i, def) in data_source_defs.iter().enumerate() {
            let plts1_id = self.create_data_source(
                def.0, def.1, def.2, def.3, def.4, def.5, def.6, def.7,
                def.8, def.9, def.10, def.11, def.12, "plts_1", None, "sparkline",
            ).await?;
            plts1_data_source_ids.push(plts1_id);

            let plts2_id = self.create_data_source(
                def.0, def.1, def.2, def.3, def.4, def.5, def.6, def.7,
                def.8, i as i64 + 6, def.10, def.11, def.12, "plts_2", None, "sparkline",
            ).await?;
            plts2_data_source_ids.push(plts2_id);
        }

        // Assign data sources to screen "plts_1"
        for (i, &sid) in plts1_data_source_ids.iter().enumerate() {
            self.add_screen_data_source("plts_1", sid, i as i64).await?;
        }

        // Assign data sources to screen "plts_2"
        for (i, &sid) in plts2_data_source_ids.iter().enumerate() {
            self.add_screen_data_source("plts_2", sid, i as i64).await?;
        }

        tracing::info!("Seeded PLTS data sources for plts_1 and plts_2");
        Ok(())
    }
}
