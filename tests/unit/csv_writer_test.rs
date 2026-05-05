use fm_size::csv_writer;
use fm_size::data_models::{DatabaseFile, Field, Table};
use chrono::Local;
use std::path::Path;
use tempfile::TempDir;

// Helper function to test formatting functions
// Note: These are private, so we test them indirectly through CSV output

#[test]
fn test_write_file_sizes_csv() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    
    let databases = vec![
        DatabaseFile {
            path: "/path/to/db1.fmp12".to_string(),
            name: "db1".to_string(),
            size_bytes: 1024 * 1024, // 1 MB
            account_name: None,
            password: None,
            ear_key: None,
            tables: Vec::new(),
        },
        DatabaseFile {
            path: "/path/to/db2.fmp12".to_string(),
            name: "db2".to_string(),
            size_bytes: 2 * 1024 * 1024, // 2 MB
            account_name: None,
            password: None,
            ear_key: None,
            tables: Vec::new(),
        },
    ];
    
    let durations = Vec::new();
    
    let csv_config = csv_writer::CsvWriterConfig {
        output_path,
        fmdevtool_ver: "21.1.1",
        fmsize_version: "0.1.0",
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &durations,
        Local::now(),
        Local::now(),
    ).unwrap();
    
    let csv_path = output_path.join("fm-size-dbs.csv");
    assert!(csv_path.exists());
    
    let content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("db1"));
    assert!(content.contains("db2"));
    assert!(content.contains("1048576")); // 1 MB in bytes
    assert!(content.contains("2097152")); // 2 MB in bytes
}

#[test]
fn test_write_file_sizes_csv_with_versions() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    
    let databases = vec![
        DatabaseFile {
            path: "/path/to/db1.fmp12".to_string(),
            name: "db1".to_string(),
            size_bytes: 1024 * 1024,
            account_name: None,
            password: None,
            ear_key: None,
            tables: Vec::new(),
        },
    ];
    
    let durations = Vec::new();
    
    let csv_config = csv_writer::CsvWriterConfig {
        output_path,
        fmdevtool_ver: "21.1.1",
        fmsize_version: "0.1.0",
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &durations,
        Local::now(),
        Local::now(),
    ).unwrap();
    
    let csv_path = output_path.join("fm-size-dbs.csv");
    let content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("fmsize_ver"));
    assert!(content.contains("fmdevtool_ver"));
    assert!(content.contains("0.1.0"));
    assert!(content.contains("21.1.1"));
}

#[test]
fn test_write_field_sizes_csv() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    
    let databases = vec![
        DatabaseFile {
            path: "/path/to/db1.fmp12".to_string(),
            name: "db1".to_string(),
            size_bytes: 1024 * 1024,
            account_name: None,
            password: None,
            ear_key: None,
            tables: vec![
                Table {
                    name: "Table1".to_string(),
                    size_bytes: 512 * 1024,
                    fields: vec![
                        Field {
                            name: "Field1".to_string(),
                            field_size_bytes: 100 * 1024,
                            value_index_size_bytes: 50 * 1024,
                            word_index_size_bytes: 25 * 1024,
                        },
                        Field {
                            name: "Field2".to_string(),
                            field_size_bytes: 200 * 1024,
                            value_index_size_bytes: 100 * 1024,
                            word_index_size_bytes: 50 * 1024,
                        },
                    ],
                },
            ],
        },
    ];
    
    let durations = Vec::new();
    
    let csv_config = csv_writer::CsvWriterConfig {
        output_path,
        fmdevtool_ver: "21.1.1",
        fmsize_version: "0.1.0",
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &durations,
        Local::now(),
        Local::now(),
    ).unwrap();
    
    let csv_path = output_path.join("fm-size-fields.csv");
    assert!(csv_path.exists());
    
    let content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("Field1"));
    assert!(content.contains("Field2"));
    assert!(content.contains("Table1"));
    assert!(content.contains("db1"));
}

#[test]
fn test_write_durations_csv() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    
    let databases = Vec::new();
    let durations = vec![
        fm_size::data_models::CommandDuration {
            thread_id: 1,
            db: "db1".to_string(),
            table: Some("Table1".to_string()),
            command: "test command".to_string(),
            query_target: "tables".to_string(),
            duration: 1.5,
            fmdevtool_ver: "21.1.1".to_string(),
            start: Local::now(),
            end: Local::now(),
        },
    ];
    
    let csv_config = csv_writer::CsvWriterConfig {
        output_path,
        fmdevtool_ver: "21.1.1",
        fmsize_version: "0.1.0",
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &durations,
        Local::now(),
        Local::now(),
    ).unwrap();
    
    let csv_path = output_path.join("fm-size-durations.csv");
    assert!(csv_path.exists());
    
    let content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("db1"));
    assert!(content.contains("Table1"));
    assert!(content.contains("tables"));
}

#[test]
fn test_bytes_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    
    let databases = vec![
        DatabaseFile {
            path: "/path/to/db1.fmp12".to_string(),
            name: "db1".to_string(),
            size_bytes: 1024,
            account_name: None,
            password: None,
            ear_key: None,
            tables: Vec::new(),
        },
    ];
    
    let durations = Vec::new();
    
    let csv_config = csv_writer::CsvWriterConfig {
        output_path,
        fmdevtool_ver: "21.1.1",
        fmsize_version: "0.1.0",
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &durations,
        Local::now(),
        Local::now(),
    ).unwrap();
    
    let csv_path = output_path.join("fm-size-dbs.csv");
    let content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("bytes"));
    assert!(content.contains("1024"));
}

#[test]
fn test_empty_databases() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();
    
    let databases = Vec::new();
    let durations = Vec::new();
    
    // Should not panic with empty databases
    let csv_config = csv_writer::CsvWriterConfig {
        output_path,
        fmdevtool_ver: "21.1.1",
        fmsize_version: "0.1.0",
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &durations,
        Local::now(),
        Local::now(),
    ).unwrap();
    
    // Files should still be created
    assert!(output_path.join("fm-size-dbs.csv").exists());
    assert!(output_path.join("fm-size-fields.csv").exists());
    assert!(output_path.join("fm-size-durations.csv").exists());
}

