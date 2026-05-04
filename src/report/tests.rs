use super::*;

#[test]
fn test_schedule_to_duration_daily() {
    let d = schedule_to_duration("daily");
    assert_eq!(d.as_secs(), 24 * 3600);
}

#[test]
fn test_schedule_to_duration_weekly() {
    let d = schedule_to_duration("weekly");
    assert_eq!(d.as_secs(), 7 * 24 * 3600);
}

#[test]
fn test_schedule_to_duration_monthly() {
    let d = schedule_to_duration("monthly");
    assert_eq!(d.as_secs(), 30 * 24 * 3600);
}

#[test]
fn test_schedule_to_duration_unknown_defaults_to_daily() {
    let d = schedule_to_duration("hourly");
    assert_eq!(d.as_secs(), 24 * 3600);
}

#[test]
fn test_report_filename() {
    let report = crate::models::Report {
        id: 1,
        name: "Daily Summary".to_string(),
        schedule: "daily".to_string(),
        enabled: true,
        device_ids: "all".to_string(),
        email_recipients: "[]".to_string(),
        last_generated: None,
        created_at: 1700000000,
    };
    let filename = report_filename(&report);
    assert!(filename.starts_with("report_"));
    assert!(filename.ends_with("_daily.html"));
}

#[test]
fn test_report_filename_weekly() {
    let report = crate::models::Report {
        id: 2,
        name: "Weekly Report".to_string(),
        schedule: "weekly".to_string(),
        enabled: true,
        device_ids: "[\"dev1\"]".to_string(),
        email_recipients: "[]".to_string(),
        last_generated: None,
        created_at: 1700000000,
    };
    let filename = report_filename(&report);
    assert!(filename.ends_with("_weekly.html"));
}

#[test]
fn test_build_ascii_chart() {
    let points = vec![
        crate::models::DataPoint {
            timestamp: 1,
            value: 10.0,
        },
        crate::models::DataPoint {
            timestamp: 2,
            value: 20.0,
        },
        crate::models::DataPoint {
            timestamp: 3,
            value: 15.0,
        },
        crate::models::DataPoint {
            timestamp: 4,
            value: 30.0,
        },
        crate::models::DataPoint {
            timestamp: 5,
            value: 25.0,
        },
    ];
    let chart = build_ascii_chart(&points);
    assert!(!chart.is_empty());
    // Should have some characters
    assert!(chart.len() >= 3);
}

#[test]
fn test_build_ascii_chart_empty() {
    let chart = build_ascii_chart(&[]);
    assert!(chart.is_empty());
}

#[test]
fn test_build_ascii_chart_single_point() {
    let chart = build_ascii_chart(&[crate::models::DataPoint {
        timestamp: 1,
        value: 42.0,
    }]);
    assert!(chart.is_empty());
}

#[test]
fn test_capitalize_first() {
    assert_eq!(capitalize_first("daily"), "Daily");
    assert_eq!(capitalize_first("weekly"), "Weekly");
    assert_eq!(capitalize_first("monthly"), "Monthly");
    assert_eq!(capitalize_first(""), "");
}

#[test]
fn test_html_escape() {
    assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    assert_eq!(html_escape("a & b"), "a &amp; b");
}
