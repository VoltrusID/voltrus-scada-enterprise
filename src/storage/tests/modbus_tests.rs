use super::*;

#[tokio::test]
async fn test_count_modbus_devices_empty() {
    let storage = Storage::new_in_memory().unwrap();
    let count = storage.count_modbus_devices().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_count_modbus_devices_with_data() {
    let storage = Storage::new_in_memory().unwrap();
    storage
        .create_modbus_device(
            "dev_count_1",
            "Test Device",
            "192.168.1.1",
            502,
            1,
            1000,
            "big",
            &[],
        )
        .await
        .unwrap();
    let count = storage.count_modbus_devices().await.unwrap();
    assert_eq!(count, 1);
}
