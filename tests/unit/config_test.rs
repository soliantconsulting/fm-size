use fm_size::config::{Args, ProcessedConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_from_args_with_cli_args() {
    let temp_dir = TempDir::new().unwrap();
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();
    
    // Test without account_name to avoid prompting
    let args = Args {
        fmdevtool_path: Some(tool_path.to_string_lossy().to_string()),
        db_file_paths: Some(vec!["/path/to/db.fmp12".to_string()]),
        db_filter: None,
        config_path: None,
        output_file_path: None,
        max_concurrent: 2,
        recurse: false,
    };

    let config = ProcessedConfig::from_args(args).unwrap();
    assert_eq!(config.fmdevtool_path, tool_path.to_string_lossy());
    assert_eq!(config.databases.len(), 1);
    assert_eq!(config.databases[0].path, "/path/to/db.fmp12");
    assert_eq!(config.databases[0].account_name, None);
    assert_eq!(config.max_concurrent, 2);
    assert!(!config.config_loaded);
}

#[test]
fn test_from_args_with_config_file() {
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
            }},
            {{
                "path_to_db": "/path/to/db2.fmp12",
                "account_name": null,
                "password": null,
                "ear_key": "key123"
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
    assert_eq!(config.databases.len(), 2);
    assert_eq!(config.databases[0].path, "/path/to/db1.fmp12");
    assert_eq!(config.databases[0].account_name, Some("admin".to_string()));
    assert_eq!(config.databases[1].ear_key, Some("key123".to_string()));
    assert!(config.config_loaded);
}

#[test]
fn test_from_args_missing_fmdevtool_path() {
    let args = Args {
        fmdevtool_path: None,
        db_file_paths: Some(vec!["/path/to/db.fmp12".to_string()]),
        db_filter: None,
        config_path: None,
        output_file_path: None,
        max_concurrent: 1,
        recurse: false,
    };

    let result = ProcessedConfig::from_args(args);
    // If default path exists, it should succeed; otherwise it should fail
    match result {
        Ok(config) => {
            // Default path exists - verify it's using the default path
            let default_path = ProcessedConfig::get_default_fmdevtool_path();
            if let Some(path) = default_path {
                assert_eq!(config.fmdevtool_path, path);
            }
        }
        Err(e) => {
            // Default path doesn't exist - verify error message
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("fmdevtool_path is required") 
                || error_msg.contains("FMDeveloperTool not found")
                || error_msg.contains("default location")
            );
        }
    }
}

#[test]
fn test_from_args_no_databases() {
    let temp_dir = TempDir::new().unwrap();
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();
    
    let args = Args {
        fmdevtool_path: Some(tool_path.to_string_lossy().to_string()),
        db_file_paths: None,
        db_filter: None,
        config_path: None,
        output_file_path: None,
        max_concurrent: 1,
        recurse: false,
    };

    let result = ProcessedConfig::from_args(args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No database files specified"));
}

#[test]
fn test_from_args_invalid_max_concurrent() {
    let temp_dir = TempDir::new().unwrap();
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();
    
    let args = Args {
        fmdevtool_path: Some(tool_path.to_string_lossy().to_string()),
        db_file_paths: Some(vec!["/path/to/db.fmp12".to_string()]),
        db_filter: None,
        config_path: None,
        output_file_path: None,
        max_concurrent: 0,
        recurse: false,
    };

    let result = ProcessedConfig::from_args(args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("max_concurrent must be greater than 0"));
}


#[test]
fn test_from_args_db_filter() {
    let temp_dir = TempDir::new().unwrap();
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();
    
    let args = Args {
        fmdevtool_path: Some(tool_path.to_string_lossy().to_string()),
        db_file_paths: Some(vec!["/path/to/dir".to_string()]),
        account_name: None,
        db_filter: Some("db1.fmp12,db2.fmp12".to_string()),
        config_path: None,
        output_file_path: None,
        max_concurrent: 1,
        recurse: false,
    };

    let config = ProcessedConfig::from_args(args).unwrap();
    assert_eq!(config.db_filter, Some(vec!["db1.fmp12".to_string(), "db2.fmp12".to_string()]));
}

#[test]
fn test_from_args_db_filter_without_extension() {
    let temp_dir = TempDir::new().unwrap();
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();
    
    let args = Args {
        fmdevtool_path: Some(tool_path.to_string_lossy().to_string()),
        db_file_paths: Some(vec!["/path/to/dir".to_string()]),
        account_name: None,
        db_filter: Some("db1,db2".to_string()),
        config_path: None,
        output_file_path: None,
        max_concurrent: 1,
        recurse: false,
    };

    let config = ProcessedConfig::from_args(args).unwrap();
    assert_eq!(config.db_filter, Some(vec!["db1.fmp12".to_string(), "db2.fmp12".to_string()]));
}

#[test]
fn test_from_args_output_path_explicit() {
    let temp_dir = TempDir::new().unwrap();
    let tool_path = temp_dir.path().join("FMDeveloperTool");
    fs::write(&tool_path, "fake tool").unwrap();
    
    let args = Args {
        fmdevtool_path: Some(tool_path.to_string_lossy().to_string()),
        db_file_paths: Some(vec!["/path/to/db.fmp12".to_string()]),
        db_filter: None,
        config_path: None,
        output_file_path: Some("/custom/output".to_string()),
        max_concurrent: 1,
        recurse: false,
    };

    let config = ProcessedConfig::from_args(args).unwrap();
    assert_eq!(config.output_file_path.to_string_lossy(), "/custom/output");
}

#[test]
fn test_load_config_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    fs::write(&config_file, "invalid json").unwrap();
    
    let result = ProcessedConfig::load_config(&config_file.to_string_lossy());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse config file"));
}

#[test]
fn test_load_config_missing_file() {
    let result = ProcessedConfig::load_config("/nonexistent/path/config.json");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read config file"));
}

