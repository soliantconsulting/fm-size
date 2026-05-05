use fm_size::utils::{apply_quantity_limit, calculate_percentage, SizeUnit};

#[test]
fn test_size_unit_conversion() {
    let bytes = 1024 * 1024; // 1 MB

    assert_eq!(SizeUnit::Bytes.convert_bytes(bytes), 1048576.0);
    assert_eq!(SizeUnit::Kilobytes.convert_bytes(bytes), 1024.0);
    assert_eq!(SizeUnit::Megabytes.convert_bytes(bytes), 1.0);
    assert_eq!(SizeUnit::Gigabytes.convert_bytes(bytes), 1.0 / 1024.0);
}

#[test]
fn test_size_unit_from_str() {
    assert!(SizeUnit::from_str("b").is_ok());
    assert!(SizeUnit::from_str("kb").is_ok());
    assert!(SizeUnit::from_str("mb").is_ok());
    assert!(SizeUnit::from_str("gb").is_ok());
    assert!(SizeUnit::from_str("invalid").is_err());
}

#[test]
fn test_calculate_percentage() {
    assert_eq!(calculate_percentage(50, 100), 50.0);
    assert_eq!(calculate_percentage(25, 100), 25.0);
    assert_eq!(calculate_percentage(0, 100), 0.0);
    assert_eq!(calculate_percentage(100, 0), 0.0); // Division by zero protection
}

#[test]
fn test_apply_quantity_limit() {
    struct Item {
        value: u64,
    }

    let items = vec![
        Item { value: 100 },
        Item { value: 50 },
        Item { value: 200 },
        Item { value: 75 },
    ];

    let limited = apply_quantity_limit(items, Some(2), |item| item.value);
    assert_eq!(limited.len(), 2);
    assert_eq!(limited[0].value, 200);
    assert_eq!(limited[1].value, 100);

    let unlimited = apply_quantity_limit(
        vec![Item { value: 10 }, Item { value: 20 }],
        None,
        |item| item.value,
    );
    assert_eq!(unlimited.len(), 2);
}

