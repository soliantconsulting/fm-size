use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_fmp12_files(
    paths: &[String],
    recurse: bool,
    db_filter: Option<&[String]>,
) -> anyhow::Result<Vec<(PathBuf, u64)>> {
    let mut files = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);
        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path_str);
        }

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("fmp12") {
            let size = fs::metadata(path)
                .with_context(|| format!("Failed to get metadata for {}", path_str))?
                .len();
            files.push((path.to_path_buf(), size));
        } else if path.is_dir() {
            find_in_directory(path, recurse, &mut files)?;
        }
    }

    // Apply filter if provided
    if let Some(filter) = db_filter {
        let filter_set: std::collections::HashSet<&str> =
            filter.iter().map(|s| s.as_str()).collect();
        files.retain(|(p, _)| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| filter_set.contains(n))
                .unwrap_or(false)
        });
    }

    // Sort by size (largest first)
    files.sort_by(|(_, a), (_, b)| b.cmp(a));

    Ok(files)
}

fn find_in_directory(
    dir: &Path,
    recurse: bool,
    files: &mut Vec<(PathBuf, u64)>,
) -> anyhow::Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if path.extension().and_then(|s| s.to_str()) == Some("fmp12") {
                let size = fs::metadata(&path)
                    .with_context(|| format!("Failed to get metadata for {}", path.display()))?
                    .len();
                files.push((path, size));
            }
        } else if path.is_dir() && recurse {
            find_in_directory(&path, recurse, files)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        let result = find_fmp12_files(&paths, false, None).unwrap();

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
        let result = find_fmp12_files(&paths, false, Some(&filter)).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].0.ends_with("test1.fmp12"));
    }
}
