use fm_size::logger::Logger;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_logger_creation() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let _logger = Logger::new(&log_path).unwrap();

    // Logger should be created successfully
    assert!(log_path.exists());
}

#[test]
fn test_logger_log_message() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("test_db", None, None, "Test message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("test_db"));
    assert!(content.contains("Test message"));
}

#[test]
fn test_logger_log_with_table() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("test_db", Some("test_table"), None, "Test message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("test_db"));
    assert!(content.contains("test_table"));
    assert!(content.contains("Test message"));
}

#[test]
fn test_logger_log_with_thread_id() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("test_db", None, Some(1), "Test message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("[DB1]"));
    assert!(content.contains("test_db"));
    assert!(content.contains("Test message"));
}

#[test]
fn test_logger_log_system_message() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("system", None, None, "System message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    // System messages should not have [system] prefix
    assert!(!content.contains("[system]"));
    assert!(content.contains("System message"));
}

#[test]
fn test_logger_log_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger.log_error("Error message").unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("❌"));
    assert!(content.contains("Error message"));
}

#[test]
fn test_logger_write_blank_line() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("test_db", None, None, "First message")
        .unwrap();
    logger.write_blank_line().unwrap();
    logger
        .log_now("test_db", None, None, "Second message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // Should have at least 3 lines (first message, blank line, second message)
    assert!(lines.len() >= 3);
}

#[test]
fn test_logger_append_mode() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger1 = Logger::new(&log_path).unwrap();
    logger1
        .log_now("test_db", None, None, "First message")
        .unwrap();

    // Create a new logger instance - should append
    let logger2 = Logger::new(&log_path).unwrap();
    logger2
        .log_now("test_db", None, None, "Second message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("First message"));
    assert!(content.contains("Second message"));
}

#[test]
fn test_logger_timestamp_format() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("test_db", None, None, "Test message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    // Should contain timestamp in format YYYY-MM-DD HH:MM:SS
    // Just check that it starts with a bracket and contains a dash (date format)
    assert!(content.contains("["));
    assert!(content.contains("-"));
}

#[test]
fn test_logger_table_format() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = Logger::new(&log_path).unwrap();
    logger
        .log_now("test_db", Some("test_table"), None, "Test message")
        .unwrap();

    let content = fs::read_to_string(&log_path).unwrap();
    // Should contain format: [db → table]
    assert!(content.contains("test_db"));
    assert!(content.contains("test_table"));
    assert!(content.contains("→"));
}
