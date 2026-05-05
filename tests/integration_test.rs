use fm_size::config::{Args, ProcessedConfig};
use fm_size::file_finder;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_file_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();

    let config_content = format!(
        r#"
    {{
        "fmdevtool_path": "{}",
        "databases": [
            {{
                "path_to_db": "/path/to/db1.fmp12",
                "account_name": "admin",
                "password": "password",
                "ear_key": null
            }}
        ]
    }}
    "#,
        tool_path.to_string_lossy().replace('\\', "\\\\")
    );

    fs::write(&config_file, config_content).unwrap();

    let args = Args {
        fmdevtool_path: None,
        db_file_paths: None,
        db_filter: None,
        config_path: Some(config_file.to_string_lossy().to_string()),
        output_file_path: None,
        max_concurrent: 1,
        recurse: false,
    };

    let config = ProcessedConfig::from_args(args).unwrap();
    assert_eq!(config.fmdevtool_path, tool_path.to_string_lossy());
    assert_eq!(config.databases.len(), 1);
    assert!(config.config_loaded);
}

#[test]
fn test_file_finder_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();

    // Create files
    fs::write(root.join("root.fmp12"), "test").unwrap();
    fs::write(subdir.join("sub.fmp12"), "test").unwrap();
    fs::write(root.join("not_fmp12.txt"), "test").unwrap();

    let paths = vec![root.to_string_lossy().to_string()];
    let result = file_finder::find_fmp12_files(&paths, true, None).unwrap();

    // Should find both .fmp12 files
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|(p, _)| p.ends_with("root.fmp12")));
    assert!(result.iter().any(|(p, _)| p.ends_with("sub.fmp12")));
}

#[test]
fn test_file_finder_non_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create directory structure
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();

    // Create files
    fs::write(root.join("root.fmp12"), "test").unwrap();
    fs::write(subdir.join("sub.fmp12"), "test").unwrap();

    let paths = vec![root.to_string_lossy().to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, None).unwrap();

    // Should only find root file (non-recursive)
    assert_eq!(result.len(), 1);
    assert!(result[0].0.ends_with("root.fmp12"));
}

#[test]
fn test_file_finder_with_filter() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(root.join("db1.fmp12"), "test").unwrap();
    fs::write(root.join("db2.fmp12"), "test").unwrap();
    fs::write(root.join("db3.fmp12"), "test").unwrap();

    let paths = vec![root.to_string_lossy().to_string()];
    let filter = vec!["db1.fmp12".to_string(), "db3.fmp12".to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, Some(&filter)).unwrap();

    // Should only find filtered files
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|(p, _)| p.ends_with("db1.fmp12")));
    assert!(result.iter().any(|(p, _)| p.ends_with("db3.fmp12")));
    assert!(!result.iter().any(|(p, _)| p.ends_with("db2.fmp12")));
}

#[test]
fn test_file_finder_with_direct_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let file_path = root.join("direct.fmp12");
    fs::write(&file_path, "test").unwrap();

    let paths = vec![file_path.to_string_lossy().to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, None).unwrap();

    // Should find the direct file
    assert_eq!(result.len(), 1);
    assert!(result[0].0.ends_with("direct.fmp12"));
}

#[test]
fn test_file_finder_nonexistent_path() {
    let paths = vec!["/nonexistent/path".to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, None);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Path does not exist"));
}

#[test]
fn test_config_file_with_multiple_databases() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();

    let config_content = format!(
        r#"
    {{
        "fmdevtool_path": "{}",
        "databases": [
            {{
                "path_to_db": "/path/to/db1.fmp12",
                "account_name": "admin1",
                "password": "pass1",
                "ear_key": null
            }},
            {{
                "path_to_db": "/path/to/db2.fmp12",
                "account_name": "admin2",
                "password": "pass2",
                "ear_key": "key123"
            }},
            {{
                "path_to_db": "/path/to/db3.fmp12",
                "account_name": null,
                "password": null,
                "ear_key": null
            }}
        ]
    }}
    "#,
        tool_path.to_string_lossy().replace('\\', "\\\\")
    );

    fs::write(&config_file, config_content).unwrap();

    let args = Args {
        fmdevtool_path: None,
        db_file_paths: None,
        db_filter: None,
        config_path: Some(config_file.to_string_lossy().to_string()),
        output_file_path: None,
        max_concurrent: 3,
        recurse: false,
    };

    let config = ProcessedConfig::from_args(args).unwrap();
    assert_eq!(config.databases.len(), 3);
    assert_eq!(config.databases[0].account_name, Some("admin1".to_string()));
    assert_eq!(config.databases[1].ear_key, Some("key123".to_string()));
    assert_eq!(config.databases[2].account_name, None);
    assert_eq!(config.max_concurrent, 3);
}

#[test]
fn test_config_file_invalid_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    // Missing required fields
    let config_content = r#"
    {
        "databases": []
    }
    "#;

    fs::write(&config_file, config_content).unwrap();

    let args = Args {
        fmdevtool_path: None,
        db_file_paths: None,
        db_filter: None,
        config_path: Some(config_file.to_string_lossy().to_string()),
        output_file_path: None,
        max_concurrent: 1,
        recurse: false,
    };

    let result = ProcessedConfig::from_args(args);
    // Should fail because fmdevtool_path is missing from config
    assert!(result.is_err());
}

#[test]
fn test_file_finder_sorted_by_size() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create files with different sizes
    fs::write(root.join("small.fmp12"), "x").unwrap(); // 1 byte
    fs::write(root.join("medium.fmp12"), "xxxxxx").unwrap(); // 6 bytes
    fs::write(root.join("large.fmp12"), "xxxxxxxxxxxx").unwrap(); // 12 bytes

    let paths = vec![root.to_string_lossy().to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, None).unwrap();

    // Should be sorted by size (largest first)
    assert_eq!(result.len(), 3);
    assert!(result[0].0.ends_with("large.fmp12"));
    assert!(result[1].0.ends_with("medium.fmp12"));
    assert!(result[2].0.ends_with("small.fmp12"));
}
