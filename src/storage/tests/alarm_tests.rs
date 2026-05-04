use super::*;

#[tokio::test]
async fn test_create_and_list_alarm() {
    let storage = Storage::new_in_memory().unwrap();

    let id = storage.create_alarm("tag1", "critical", 100.0, 90.0, "Over threshold", 5.0).await.unwrap();
    assert!(id > 0);

    let alarms = storage.list_alarms(Some("active"), 50).await.unwrap();
    assert_eq!(alarms.len(), 1);
    assert_eq!(alarms[0].source_key, "tag1");
    assert_eq!(alarms[0].severity, "critical");
    assert_eq!(alarms[0].deadband, 5.0);
    assert_eq!(alarms[0].state, "active");
}

#[tokio::test]
async fn test_create_alarm_with_info_severity() {
    let storage = Storage::new_in_memory().unwrap();

    let id = storage.create_alarm("tag1", "info", 50.0, 0.0, "Informational", 0.0).await.unwrap();
    assert!(id > 0);

    let alarms = storage.list_alarms(Some("all"), 50).await.unwrap();
    assert_eq!(alarms.len(), 1);
    assert_eq!(alarms[0].severity, "info");
    assert_eq!(alarms[0].priority, "info");
}

#[tokio::test]
async fn test_acknowledge_alarm() {
    let storage = Storage::new_in_memory().unwrap();

    let id = storage.create_alarm("tag1", "warning", 75.0, 70.0, "Warning", 2.0).await.unwrap();
    storage.acknowledge_alarm(id, "admin", 300).await.unwrap();

    let alarms = storage.list_alarms(Some("acknowledged"), 50).await.unwrap();
    assert_eq!(alarms.len(), 1);
    assert_eq!(alarms[0].acknowledged_by, Some("admin".to_string()));
    assert_eq!(alarms[0].state, "acknowledged");
    assert!(alarms[0].cooldown_until.is_some());
}

#[tokio::test]
async fn test_shelf_and_unshelf_alarm() {
    let storage = Storage::new_in_memory().unwrap();

    let id = storage.create_alarm("tag1", "warning", 75.0, 70.0, "Warning", 0.0).await.unwrap();
    let now = crate::timeutil::now_secs();
    let shelved_until = now + 3600;
    storage.shelf_alarm(id, shelved_until, "admin", "Maintenance window").await.unwrap();

    let alarms = storage.list_alarms(Some("all"), 50).await.unwrap();
    assert_eq!(alarms[0].state, "shelved");
    assert_eq!(alarms[0].shelved_by, Some("admin".to_string()));
    assert_eq!(alarms[0].shelved_reason, Some("Maintenance window".to_string()));

    storage.unshelf_alarm(id).await.unwrap();
    let alarms = storage.list_alarms(Some("all"), 50).await.unwrap();
    assert_eq!(alarms[0].state, "active");
    assert!(alarms[0].shelved_until.is_none());
}

#[tokio::test]
async fn test_auto_unshelf_expired_alarms() {
    let storage = Storage::new_in_memory().unwrap();

    let id = storage.create_alarm("tag1", "warning", 75.0, 70.0, "Warning", 0.0).await.unwrap();
    let now = crate::timeutil::now_secs();
    let shelved_until = now - 10; // already expired
    storage.shelf_alarm(id, shelved_until, "admin", "Expired").await.unwrap();

    let alarms = storage.list_alarms(Some("all"), 50).await.unwrap();
    assert_eq!(alarms[0].state, "shelved");

    storage.auto_unshelf_expired_alarms().await.unwrap();

    let alarms = storage.list_alarms(Some("all"), 50).await.unwrap();
    assert_eq!(alarms[0].state, "active");
    assert!(alarms[0].shelved_until.is_none());
}

#[tokio::test]
async fn test_resolve_active_alarms_with_deadband() {
    let storage = Storage::new_in_memory().unwrap();

    // Create alarm with deadband of 5.0 (threshold 100)
    let id = storage.create_alarm("tag1", "critical", 110.0, 100.0, "Over threshold", 5.0).await.unwrap();

    // Value is 96: 96 < 100 - 5 = 95? No, so alarm should NOT resolve
    storage.resolve_active_alarms("tag1", Some(96.0)).await.unwrap();
    let alarms = storage.list_alarms(Some("active"), 50).await.unwrap();
    assert_eq!(alarms.len(), 1);
    assert!(alarms[0].resolved_at.is_none());

    // Value is 94: 94 < 100 - 5 = 95? Yes, so alarm should resolve
    storage.resolve_active_alarms("tag1", Some(94.0)).await.unwrap();
    let alarms = storage.list_alarms(Some("all"), 50).await.unwrap();
    assert_eq!(alarms.len(), 1);
    assert!(alarms[0].resolved_at.is_some());
    assert_eq!(alarms[0].state, "resolved");
}

#[tokio::test]
async fn test_alarm_summary() {
    let storage = Storage::new_in_memory().unwrap();

    storage.create_alarm("tag1", "critical", 100.0, 90.0, "Critical", 0.0).await.unwrap();
    storage.create_alarm("tag2", "warning", 80.0, 70.0, "Warning", 0.0).await.unwrap();
    storage.create_alarm("tag3", "info", 50.0, 0.0, "Info", 0.0).await.unwrap();

    let summary = storage.get_alarm_summary().await.unwrap();
    assert_eq!(summary.active_critical, 1);
    assert_eq!(summary.active_warning, 1);
    assert_eq!(summary.active_info, 1);
}

#[tokio::test]
async fn test_alarm_audit_log() {
    let storage = Storage::new_in_memory().unwrap();

    let id = storage.create_alarm("tag1", "warning", 75.0, 70.0, "Warning", 0.0).await.unwrap();
    storage.acknowledge_alarm(id, "admin", 0).await.unwrap();

    let audit = storage.list_alarm_audit(Some(id), None, None, 50).await.unwrap();
    assert!(!audit.is_empty());
    assert!(audit.iter().any(|e| e.action == "created"));
    assert!(audit.iter().any(|e| e.action == "acknowledged"));
}
