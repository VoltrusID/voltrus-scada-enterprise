//! Statistical Process Control (SPC) calculations.
//!
//! Provides X-bar & R, CUSUM, EWMA, P-chart, and NP-chart functions
//! using the crate's `SpcResult`, `SpcPoint`, and `ControlLimits` types.

use crate::models::{ControlLimits, SpcPoint, SpcResult};

// ── SPC constants (subgroup_size 2..=10, indexed by size - 2) ──

const A2: [f64; 9] = [1.880, 1.023, 0.729, 0.577, 0.483, 0.419, 0.373, 0.337, 0.308];
const D3: [f64; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.076, 0.136, 0.184, 0.223];
const D4: [f64; 9] = [3.267, 2.575, 2.282, 2.115, 2.004, 1.924, 1.864, 1.816, 1.777];

/// X-bar & R chart calculation.
///
/// Divides `values` into subgroups of `subgroup_size` (2-10), computes the
/// mean (X-bar) and range (R) for each subgroup, and derives control limits
/// using standard SPC constants A2, D3, D4.
///
/// Returns `((xbar_result, r_result))` on success.
pub fn xbar_r(
    values: &[f64],
    subgroup_size: usize,
    timestamps: &[i64],
) -> Result<(SpcResult, SpcResult), String> {
    if subgroup_size < 2 || subgroup_size > 10 {
        return Err(format!(
            "subgroup_size must be between 2 and 10, got {}",
            subgroup_size
        ));
    }

    let idx = subgroup_size - 2;

    // Divide into subgroups, skipping incomplete final subgroup
    let num_subgroups = values.len() / subgroup_size;
    if num_subgroups == 0 {
        return Err("not enough values for even one subgroup".to_string());
    }

    let mut xbars: Vec<f64> = Vec::with_capacity(num_subgroups);
    let mut ranges: Vec<f64> = Vec::with_capacity(num_subgroups);
    let mut xbar_points: Vec<SpcPoint> = Vec::with_capacity(num_subgroups);
    let mut r_points: Vec<SpcPoint> = Vec::with_capacity(num_subgroups);

    for i in 0..num_subgroups {
        let start = i * subgroup_size;
        let end = start + subgroup_size;
        let subgroup = &values[start..end];

        let mean: f64 = subgroup.iter().sum::<f64>() / subgroup_size as f64;
        let min_val = subgroup.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = subgroup.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        xbars.push(mean);
        ranges.push(range);

        // Use the timestamp of the first element in the subgroup
        let ts = if i < timestamps.len() {
            timestamps[start]
        } else {
            0
        };

        xbar_points.push(SpcPoint {
            index: i,
            value: mean,
            timestamp: ts,
            subgroup_values: Some(subgroup.to_vec()),
        });

        r_points.push(SpcPoint {
            index: i,
            value: range,
            timestamp: ts,
            subgroup_values: Some(subgroup.to_vec()),
        });
    }

    let xbar_avg: f64 = xbars.iter().sum::<f64>() / num_subgroups as f64;
    let r_avg: f64 = ranges.iter().sum::<f64>() / num_subgroups as f64;

    // X-bar control limits
    let xbar_ucl = xbar_avg + A2[idx] * r_avg;
    let xbar_lcl = xbar_avg - A2[idx] * r_avg;

    // R control limits
    let r_ucl = D4[idx] * r_avg;
    let r_lcl = D3[idx] * r_avg;

    // Violations
    let xbar_violations: Vec<usize> = xbars
        .iter()
        .enumerate()
        .filter(|&(_, &v)| v > xbar_ucl || v < xbar_lcl)
        .map(|(i, _)| i)
        .collect();

    let r_violations: Vec<usize> = ranges
        .iter()
        .enumerate()
        .filter(|&(_, &v)| v > r_ucl || v < r_lcl)
        .map(|(i, _)| i)
        .collect();

    let xbar_in_control = if num_subgroups > 0 {
        (1.0 - (xbar_violations.len() as f64 / num_subgroups as f64)) * 100.0
    } else {
        100.0
    };

    let r_in_control = if num_subgroups > 0 {
        (1.0 - (r_violations.len() as f64 / num_subgroups as f64)) * 100.0
    } else {
        100.0
    };

    let xbar_result = SpcResult {
        chart_type: "xbar".to_string(),
        data_points: xbar_points,
        control_limits: ControlLimits {
            ucl: xbar_ucl,
            cl: xbar_avg,
            lcl: xbar_lcl,
        },
        violations: xbar_violations,
        in_control_percent: xbar_in_control,
    };

    let r_result = SpcResult {
        chart_type: "r".to_string(),
        data_points: r_points,
        control_limits: ControlLimits {
            ucl: r_ucl,
            cl: r_avg,
            lcl: r_lcl,
        },
        violations: r_violations,
        in_control_percent: r_in_control,
    };

    Ok((xbar_result, r_result))
}

/// CUSUM (Cumulative Sum) control chart.
///
/// Tracks cumulative deviation from `target` with slack `k` and decision
/// interval `h`. A point violates when `|S| > h`.
pub fn cusum(
    values: &[f64],
    target: f64,
    k: f64,
    h: f64,
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if values.is_empty() {
        return Err("values must not be empty".to_string());
    }
    if h <= 0.0 {
        return Err("h must be positive".to_string());
    }

    let n = values.len();
    let mut cusum_pos = 0.0_f64;
    let mut cusum_neg = 0.0_f64;
    let mut points: Vec<SpcPoint> = Vec::with_capacity(n);
    let mut violations: Vec<usize> = Vec::new();

    for (i, &x) in values.iter().enumerate() {
        cusum_pos = (cusum_pos + (x - target) - k).max(0.0);
        cusum_neg = (cusum_neg + (x - target) + k).min(0.0);

        let value = cusum_pos.max(cusum_neg.abs());

        let ts = if i < timestamps.len() {
            timestamps[i]
        } else {
            0
        };

        points.push(SpcPoint {
            index: i,
            value,
            timestamp: ts,
            subgroup_values: None,
        });

        if value > h {
            violations.push(i);
        }
    }

    let in_control = (1.0 - (violations.len() as f64 / n as f64)) * 100.0;

    Ok(SpcResult {
        chart_type: "cusum".to_string(),
        data_points: points,
        control_limits: ControlLimits {
            ucl: h,
            cl: 0.0,
            lcl: -h,
        },
        violations,
        in_control_percent: in_control,
    })
}

/// EWMA (Exponentially Weighted Moving Average) control chart.
///
/// Uses smoothing parameter `lambda` (0 < lambda <= 1) and L=3 sigma limits.
/// The EWMA statistic starts at `target` and converges with each observation.
pub fn ewma(
    values: &[f64],
    target: f64,
    lambda: f64,
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if values.is_empty() {
        return Err("values must not be empty".to_string());
    }
    if lambda <= 0.0 || lambda > 1.0 {
        return Err("lambda must be in (0, 1]".to_string());
    }

    let n = values.len();

    // Estimate process sigma from data
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values
        .iter()
        .map(|&v| (v - mean).powi(2))
        .sum::<f64>()
        / n as f64;
    let sigma = variance.sqrt();
    if sigma < f64::EPSILON {
        return Err("sigma is zero — data has no variation".to_string());
    }

    let l = 3.0; // 3-sigma
    let mut z = target;
    let mut points: Vec<SpcPoint> = Vec::with_capacity(n);
    let mut violations: Vec<usize> = Vec::new();

    // Steady-state variance for final displayed limits
    let steady_var = sigma * (lambda / (2.0 - lambda)).sqrt();
    let steady_ucl = target + l * steady_var;
    let steady_lcl = target - l * steady_var;

    for (i, &x) in values.iter().enumerate() {
        z = lambda * x + (1.0 - lambda) * z;

        // Exact EWMA variance at point i
        let ewma_var = sigma
            * (lambda / (2.0 - lambda) * (1.0 - (1.0 - lambda).powi(2 * ((i + 1) as i32))))
                .sqrt();

        let point_ucl = target + l * ewma_var;
        let point_lcl = target - l * ewma_var;

        let ts = if i < timestamps.len() {
            timestamps[i]
        } else {
            0
        };

        points.push(SpcPoint {
            index: i,
            value: z,
            timestamp: ts,
            subgroup_values: None,
        });

        if z > point_ucl || z < point_lcl {
            violations.push(i);
        }
    }

    let in_control = (1.0 - (violations.len() as f64 / n as f64)) * 100.0;

    Ok(SpcResult {
        chart_type: "ewma".to_string(),
        data_points: points,
        control_limits: ControlLimits {
            ucl: steady_ucl,
            cl: target,
            lcl: steady_lcl,
        },
        violations,
        in_control_percent: in_control,
    })
}

/// P-chart (proportion defective).
///
/// Each entry in `defectives` is the count of defective items in the
/// corresponding sample whose size is given by `sample_sizes[i]`.
pub fn p_chart(
    defectives: &[u32],
    sample_sizes: &[u32],
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if defectives.is_empty() {
        return Err("defectives must not be empty".to_string());
    }
    if defectives.len() != sample_sizes.len() {
        return Err("defectives and sample_sizes must have the same length".to_string());
    }
    if sample_sizes.iter().any(|&n| n == 0) {
        return Err("sample sizes must be > 0".to_string());
    }

    let total_defective: u64 = defectives.iter().map(|&d| d as u64).sum();
    let total_inspected: u64 = sample_sizes.iter().map(|&n| n as u64).sum();
    let p_bar = total_defective as f64 / total_inspected as f64;

    let n = defectives.len();
    let mut points: Vec<SpcPoint> = Vec::with_capacity(n);
    let mut violations: Vec<usize> = Vec::new();

    for (i, (&d, &sz)) in defectives.iter().zip(sample_sizes.iter()).enumerate() {
        let p = d as f64 / sz as f64;
        let se = (p_bar * (1.0 - p_bar) / sz as f64).sqrt();
        let ucl = p_bar + 3.0 * se;
        let lcl = (p_bar - 3.0 * se).max(0.0);

        let ts = if i < timestamps.len() {
            timestamps[i]
        } else {
            0
        };

        points.push(SpcPoint {
            index: i,
            value: p,
            timestamp: ts,
            subgroup_values: None,
        });

        if p > ucl || p < lcl {
            violations.push(i);
        }
    }

    // Use overall limits for the result's control_limits
    let avg_n = total_inspected as f64 / n as f64;
    let overall_se = (p_bar * (1.0 - p_bar) / avg_n).sqrt();

    let in_control = (1.0 - (violations.len() as f64 / n as f64)) * 100.0;

    Ok(SpcResult {
        chart_type: "p".to_string(),
        data_points: points,
        control_limits: ControlLimits {
            ucl: p_bar + 3.0 * overall_se,
            cl: p_bar,
            lcl: (p_bar - 3.0 * overall_se).max(0.0),
        },
        violations,
        in_control_percent: in_control,
    })
}

/// NP-chart (number defective).
///
/// Fixed `sample_size` across all samples. `defectives[i]` is the count of
/// defective items in sample `i`.
pub fn np_chart(
    defectives: &[u32],
    sample_size: u32,
    timestamps: &[i64],
) -> Result<SpcResult, String> {
    if defectives.is_empty() {
        return Err("defectives must not be empty".to_string());
    }
    if sample_size == 0 {
        return Err("sample_size must be > 0".to_string());
    }

    let n = defectives.len();
    let total_defective: u64 = defectives.iter().map(|&d| d as u64).sum();
    let total_inspected = sample_size as u64 * n as u64;
    let p_bar = total_defective as f64 / total_inspected as f64;
    let np_bar = sample_size as f64 * p_bar;

    let se = (np_bar * (1.0 - p_bar)).sqrt();
    let ucl = np_bar + 3.0 * se;
    let lcl = (np_bar - 3.0 * se).max(0.0);

    let mut points: Vec<SpcPoint> = Vec::with_capacity(n);
    let mut violations: Vec<usize> = Vec::new();

    for (i, &d) in defectives.iter().enumerate() {
        let ts = if i < timestamps.len() {
            timestamps[i]
        } else {
            0
        };

        let d_f64 = d as f64;
        points.push(SpcPoint {
            index: i,
            value: d_f64,
            timestamp: ts,
            subgroup_values: None,
        });

        if d_f64 > ucl || d_f64 < lcl {
            violations.push(i);
        }
    }

    let in_control = (1.0 - (violations.len() as f64 / n as f64)) * 100.0;

    Ok(SpcResult {
        chart_type: "np".to_string(),
        data_points: points,
        control_limits: ControlLimits {
            ucl,
            cl: np_bar,
            lcl,
        },
        violations,
        in_control_percent: in_control,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xbar_r_basic() {
        // 20 values, subgroup_size=4 => 5 subgroups
        let values: Vec<f64> = vec![
            10.0, 11.0, 12.0, 13.0, // subgroup 0: xbar=11.5, r=3.0
            14.0, 15.0, 16.0, 17.0, // subgroup 1
            10.0, 12.0, 14.0, 16.0, // subgroup 2
            11.0, 13.0, 15.0, 17.0, // subgroup 3
            12.0, 14.0, 16.0, 18.0, // subgroup 4
        ];
        let timestamps: Vec<i64> = (0..20).collect();

        let (xbar_res, r_res) = xbar_r(&values, 4, &timestamps).unwrap();

        // 5 subgroups
        assert_eq!(xbar_res.data_points.len(), 5);
        assert_eq!(r_res.data_points.len(), 5);

        // xbar[0] = (10+11+12+13)/4 = 11.5
        assert!((xbar_res.data_points[0].value - 11.5).abs() < 1e-9);

        // r[0] = 13 - 10 = 3.0
        assert!((r_res.data_points[0].value - 3.0).abs() < 1e-9);

        // chart types
        assert_eq!(xbar_res.chart_type, "xbar");
        assert_eq!(r_res.chart_type, "r");
    }

    #[test]
    fn test_xbar_r_invalid_subgroup() {
        let values = vec![1.0, 2.0, 3.0];
        let ts: Vec<i64> = vec![0, 1, 2];

        // Size 0 should error
        assert!(xbar_r(&values, 0, &ts).is_err());

        // Size 11 should error
        assert!(xbar_r(&values, 11, &ts).is_err());
    }

    #[test]
    fn test_cusum_basic() {
        // Data centered on target=10 — should produce no violations
        let values: Vec<f64> = vec![
            10.1, 9.9, 10.0, 9.8, 10.2, 10.0, 9.9, 10.1, 10.0, 10.0,
            9.9, 10.1, 10.0, 9.8, 10.2, 10.0, 10.1, 9.9, 10.0, 10.0,
        ];
        let timestamps: Vec<i64> = (0..20).collect();

        let result = cusum(&values, 10.0, 0.5, 5.0, &timestamps).unwrap();
        assert_eq!(result.chart_type, "cusum");
        assert_eq!(result.data_points.len(), 20);
        assert!(
            result.violations.is_empty(),
            "stable data around target should have no CUSUM violations, got {:?}",
            result.violations
        );
        assert!((result.control_limits.ucl - 5.0).abs() < 1e-9);
        assert!((result.control_limits.lcl - (-5.0)).abs() < 1e-9);
        assert!((result.control_limits.cl - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_ewma_basic() {
        let values: Vec<f64> = vec![10.0, 11.0, 9.5, 10.5, 10.0, 9.8, 10.2, 10.1];
        let timestamps: Vec<i64> = (0..8).collect();

        let result = ewma(&values, 10.0, 0.2, &timestamps).unwrap();

        assert_eq!(result.chart_type, "ewma");
        assert_eq!(result.data_points.len(), 8);

        // First point: z = 0.2*10.0 + 0.8*10.0 = 10.0
        assert!((result.data_points[0].value - 10.0).abs() < 1e-9);

        // Second point: z = 0.2*11.0 + 0.8*10.0 = 10.2
        assert!((result.data_points[1].value - 10.2).abs() < 1e-9);
    }

    #[test]
    fn test_p_chart_basic() {
        // 8 samples, each of size 50
        let defectives: Vec<u32> = vec![3, 5, 2, 4, 6, 3, 5, 4];
        let sample_sizes: Vec<u32> = vec![50; 8];
        let timestamps: Vec<i64> = (0..8).collect();

        let result = p_chart(&defectives, &sample_sizes, &timestamps).unwrap();

        assert_eq!(result.chart_type, "p");
        assert_eq!(result.data_points.len(), 8);
        assert!(result.control_limits.cl > 0.0, "cl (p_bar) should be > 0");

        // p_bar = 32/400 = 0.08
        let expected_p_bar: f64 = 32.0 / 400.0;
        assert!((result.control_limits.cl - expected_p_bar).abs() < 1e-9);
    }

    #[test]
    fn test_np_chart_basic() {
        let defectives: Vec<u32> = vec![3, 5, 2, 4, 6, 3, 5, 4];
        let timestamps: Vec<i64> = (0..8).collect();

        let result = np_chart(&defectives, 50, &timestamps).unwrap();

        assert_eq!(result.chart_type, "np");
        assert_eq!(result.data_points.len(), 8);

        // np_bar = 50 * (32 / 400) = 4.0
        assert!((result.control_limits.cl - 4.0).abs() < 1e-9);
        assert!(result.control_limits.ucl > result.control_limits.cl);
        assert!(result.control_limits.lcl < result.control_limits.cl);
    }
}
