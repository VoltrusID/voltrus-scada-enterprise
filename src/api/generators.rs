use crate::models::Sensor;

/// Generate a pseudo-random value from a seed. Equivalent to the Hono worker's
/// `pseudoRandom` function: fract(sin(x * 12.9898 + 78.233) * 43758.5453)
pub fn pseudo_random(seed: f64) -> f64 {
    let x = (seed * 12.9898 + 78.233).sin() * 43758.5453;
    x - x.floor()
}

/// Generate a sensor value based on its `source_type` and configuration.
/// `now` is a time factor (seconds since epoch as f64).
pub fn generate_value(sensor: &Sensor, now: f64) -> f64 {
    match sensor.source_type.as_str() {
        "constant" => sensor.base_val,
        "none" | "api" => sensor.base_val,
        "noise" => {
            let noise = pseudo_random(now + sensor.id as f64) * 2.0 - 1.0;
            let range = sensor.max_val - sensor.min_val;
            (sensor.base_val + noise * range * 0.05).clamp(sensor.min_val, sensor.max_val)
        }
        "sine" => {
            let config: serde_json::Value =
                serde_json::from_str(&sensor.source_config).unwrap_or(serde_json::json!({}));
            let period = config["period"].as_f64().unwrap_or(60.0);
            let amplitude = config["amplitude"].as_f64().unwrap_or(sensor.max_val - sensor.min_val);
            let phase = config["phase"].as_f64().unwrap_or(0.0);
            let range = sensor.max_val - sensor.min_val;
            let amp = amplitude.min(range / 2.0);
            let val = sensor.base_val
                + ((now / period) * std::f64::consts::PI * 2.0 + phase).sin() * amp * 0.5;
            val.clamp(sensor.min_val, sensor.max_val)
        }
        "step" => {
            // Occasional jumps using pseudo-random trigger
            let step_seed = (now / 30.0).floor();
            let should_jump = pseudo_random(step_seed + sensor.id as f64) > 0.7;
            if should_jump {
                let jump_target = pseudo_random(step_seed + sensor.id as f64 * 3.7);
                let range = sensor.max_val - sensor.min_val;
                (sensor.min_val + jump_target * range).clamp(sensor.min_val, sensor.max_val)
            } else {
                sensor.base_val
            }
        }
        "sawtooth" => {
            let config: serde_json::Value =
                serde_json::from_str(&sensor.source_config).unwrap_or(serde_json::json!({}));
            let period = config["period"].as_f64().unwrap_or(120.0);
            let range = sensor.max_val - sensor.min_val;
            let phase = (now % period) / period;
            (sensor.min_val + phase * range).clamp(sensor.min_val, sensor.max_val)
        }
        _ => random_walk(sensor, now), // default: random_walk
    }
}

fn random_walk(sensor: &Sensor, now: f64) -> f64 {
    let range = sensor.max_val - sensor.min_val;
    let noise = (now * 0.5 + sensor.id as f64 * 1.7).sin() * 0.3
        + (now * 1.3 + sensor.id as f64 * 2.1).sin() * 0.15;
    (sensor.base_val + noise * range * 0.08).clamp(sensor.min_val, sensor.max_val)
}

/// Parse a range string like "30m", "1h", "6h", "24h" into (start_seconds_ago, interval_seconds).
/// Returns (start_timestamp, end_timestamp) as seconds since epoch.
pub fn parse_range(range: &str) -> (i64, i64) {
    let now = chrono::Utc::now().timestamp();
    let secs = match range {
        "30m" => 30 * 60,
        "1h" => 60 * 60,
        "6h" => 6 * 60 * 60,
        "24h" => 24 * 60 * 60,
        "7d" => 7 * 24 * 60 * 60,
        "30d" => 30 * 24 * 60 * 60,
        _ => 30 * 60, // default 30m
    };
    (now - secs, now)
}

/// Get the interval in milliseconds for a given range string.
pub fn interval_for_range(range: &str) -> u64 {
    match range {
        "30m" => 10_000,
        "1h" => 20_000,
        "6h" => 120_000,
        "24h" => 480_000,
        "7d" => 3_360_000,
        "30d" => 14_400_000,
        _ => 10_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sensor(id: i64, source_type: &str, base_val: f64, min_val: f64, max_val: f64) -> Sensor {
        Sensor {
            id,
            key: "test".to_string(),
            name: "Test".to_string(),
            unit: "V".to_string(),
            color: "#fff".to_string(),
            min_val,
            max_val,
            warn_val: 0.0,
            crit_val: 0.0,
            base_val,
            sort_order: 0,
            decimals: 1,
            source_type: source_type.to_string(),
            source_config: "{}".to_string(),
            device_id: "test".to_string(),
            equipment_id: None,
            chart_type: "sparkline".to_string(),
        }
    }

    #[test]
    fn test_constant() {
        let s = make_sensor(1, "constant", 42.0, 0.0, 100.0);
        assert_eq!(generate_value(&s, 100.0), 42.0);
        assert_eq!(generate_value(&s, 200.0), 42.0);
    }

    #[test]
    fn test_noise_within_range() {
        let s = make_sensor(1, "noise", 50.0, 0.0, 100.0);
        for t in 0..100 {
            let v = generate_value(&s, t as f64);
            assert!(v >= 0.0 && v <= 100.0, "value {v} out of range at t={t}");
        }
    }

    #[test]
    fn test_sine_within_range() {
        let s = make_sensor(1, "sine", 50.0, 0.0, 100.0);
        for t in 0..100 {
            let v = generate_value(&s, t as f64);
            assert!(v >= 0.0 && v <= 100.0, "value {v} out of range at t={t}");
        }
    }

    #[test]
    fn test_random_walk_within_range() {
        let s = make_sensor(1, "random_walk", 50.0, 0.0, 100.0);
        for t in 0..100 {
            let v = generate_value(&s, t as f64);
            assert!(v >= 0.0 && v <= 100.0, "value {v} out of range at t={t}");
        }
    }

    #[test]
    fn test_sawtooth_within_range() {
        let s = make_sensor(1, "sawtooth", 50.0, 0.0, 100.0);
        for t in 0..100 {
            let v = generate_value(&s, t as f64);
            assert!(v >= 0.0 && v <= 100.0, "value {v} out of range at t={t}");
        }
    }

    #[test]
    fn test_step_within_range() {
        let s = make_sensor(1, "step", 50.0, 0.0, 100.0);
        for t in 0..100 {
            let v = generate_value(&s, t as f64);
            assert!(v >= 0.0 && v <= 100.0, "value {v} out of range at t={t}");
        }
    }

    #[test]
    fn test_pseudo_random() {
        let v = pseudo_random(1.0);
        assert!(v >= 0.0 && v < 1.0);
        // Deterministic
        assert_eq!(v, pseudo_random(1.0));
    }

    #[test]
    fn test_parse_range() {
        let (start, end) = parse_range("30m");
        assert_eq!(end - start, 30 * 60);

        let (start, end) = parse_range("1h");
        assert_eq!(end - start, 3600);

        let (start, end) = parse_range("6h");
        assert_eq!(end - start, 6 * 3600);

        let (start, end) = parse_range("24h");
        assert_eq!(end - start, 24 * 3600);
    }

    #[test]
    fn test_interval_for_range() {
        assert_eq!(interval_for_range("30m"), 10_000);
        assert_eq!(interval_for_range("1h"), 20_000);
        assert_eq!(interval_for_range("unknown"), 10_000);
    }
}
