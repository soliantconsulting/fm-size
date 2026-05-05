use fm_size::file_finder;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_find_fmp12_files() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create some test files
    fs::write(dir_path.join("test1.fmp12"), "test").unwrap();
    fs::write(dir_path.join("test2.fmp12"), "test").unwrap();
    fs::write(dir_path.join("not_fmp12.txt"), "test").unwrap();

    let paths = vec![dir_path.to_string_lossy().to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, None).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|(p, _)| p.ends_with("test1.fmp12")));
    assert!(result.iter().any(|(p, _)| p.ends_with("test2.fmp12")));
}

#[test]
fn test_find_fmp12_files_with_filter() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    fs::write(dir_path.join("test1.fmp12"), "test").unwrap();
    fs::write(dir_path.join("test2.fmp12"), "test").unwrap();

    let paths = vec![dir_path.to_string_lossy().to_string()];
    let filter = vec!["test1.fmp12".to_string()];
    let result = file_finder::find_fmp12_files(&paths, false, Some(&filter)).unwrap();

    assert_eq!(result.len(), 1);
    assert!(result[0].0.ends_with("test1.fmp12"));
}

