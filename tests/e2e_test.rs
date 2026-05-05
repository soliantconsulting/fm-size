use std::fs;
use std::path::Path;
use std::process::Command;

/// Normalize a CSV value by replacing variable fields with placeholders
fn normalize_csv_value(value: &str, column_name: &str) -> String {
    // Check if it's a duration field (duration_in_sec)
    if column_name == "duration_in_sec" && !value.is_empty() && value.parse::<f64>().is_ok() {
        return "<DURATION>".to_string();
    }

    // Check if it's a version column
    if column_name == "fmsize_ver" || column_name == "fmdevtool_ver" {
        // Version strings are normalized to placeholder
        if !value.is_empty() {
            return "<VERSION>".to_string();
        }
    }

    value.to_string()
}

/// Normalize a CSV row by replacing variable fields
fn normalize_csv_row(row: &[String], headers: &[String]) -> Vec<String> {
    row.iter()
        .enumerate()
        .map(|(idx, value)| {
            let column_name = headers.get(idx).map(|s| s.as_str()).unwrap_or("");
            normalize_csv_value(value, column_name)
        })
        .collect()
}

/// Compare two CSV files, normalizing variable fields
fn compare_csv_files(actual_path: &Path, expected_path: &Path) -> Result<(), String> {
    use csv::ReaderBuilder;

    let actual_content = fs::read_to_string(actual_path)
        .map_err(|e| format!("Failed to read actual file {:?}: {}", actual_path, e))?;
    let expected_content = fs::read_to_string(expected_path)
        .map_err(|e| format!("Failed to read expected file {:?}: {}", expected_path, e))?;

    let mut actual_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(actual_content.as_bytes());
    let mut expected_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(expected_content.as_bytes());

    let actual_headers: Vec<String> = actual_reader
        .headers()
        .map_err(|e| format!("Failed to read actual headers: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let expected_headers: Vec<String> = expected_reader
        .headers()
        .map_err(|e| format!("Failed to read expected headers: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if actual_headers != expected_headers {
        return Err(format!(
            "Headers don't match:\n  Actual: {:?}\n  Expected: {:?}",
            actual_headers, expected_headers
        ));
    }

    let mut actual_rows: Vec<Vec<String>> = Vec::new();
    for result in actual_reader.records() {
        let record = result.map_err(|e| format!("Failed to read actual record: {}", e))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        actual_rows.push(normalize_csv_row(&row, &actual_headers));
    }

    let mut expected_rows: Vec<Vec<String>> = Vec::new();
    for result in expected_reader.records() {
        let record = result.map_err(|e| format!("Failed to read expected record: {}", e))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        expected_rows.push(normalize_csv_row(&row, &expected_headers));
    }

    if actual_rows.len() != expected_rows.len() {
        return Err(format!(
            "Row count mismatch: actual has {} rows, expected has {} rows",
            actual_rows.len(),
            expected_rows.len()
        ));
    }

    for (idx, (actual_row, expected_row)) in
        actual_rows.iter().zip(expected_rows.iter()).enumerate()
    {
        if actual_row != expected_row {
            return Err(format!(
                "Row {} mismatch:\n  Actual: {:?}\n  Expected: {:?}",
                idx + 1,
                actual_row,
                expected_row
            ));
        }
    }

    Ok(())
}

/// End-to-end test: Exercises the complete application workflow by running the actual binary
/// with CLI arguments, similar to how a user would invoke it. This tests the full integration
/// of all components from command-line parsing through to CSV output generation.
///
/// This test requires FMDeveloperTool to be available. It will be skipped if the tool is not found.
#[test]
fn test_e2e_full_run_recursive_multithreaded_cli_args() {
    // Get the path to the binary
    // Note: hyphens in package names become underscores in env var names
    // Use env::var instead of env! because this is set at runtime by Cargo
    let binary_path = std::env::var("CARGO_BIN_EXE_fm_size")
        .unwrap_or_else(|_| "target/debug/fm-size".to_string());

    // Check if FMDeveloperTool is available at default location or use provided path
    // This test requires a real FMDeveloperTool to work properly
    let default_tool_path = fm_size::config::ProcessedConfig::get_default_fmdevtool_path();
    let tool_path = if let Some(path) = default_tool_path {
        if !std::path::Path::new(&path).exists() {
            eprintln!(
                "Skipping e2e test: FMDeveloperTool not found at default location: {}",
                path
            );
            return;
        }
        path
    } else {
        // Try to use "FMDeveloperTool" from PATH
        let which_output = Command::new("which").arg("FMDeveloperTool").output().ok();
        if let Some(output) = which_output {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                eprintln!("Skipping e2e test: FMDeveloperTool not found in PATH");
                return;
            }
        } else {
            eprintln!("Skipping e2e test: Could not check for FMDeveloperTool");
            return;
        }
    };

    // Create output directory named after the test in tests/output/
    let output_dir = Path::new("tests/output/full_run_recursive_multithreaded_cli_args");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Run the tool with config file to provide authentication credentials
    // The config file contains the account/password/ear_key for the test databases
    // We need to create a temporary config file with the correct tool path
    let config_path = Path::new("tests/fixtures/no_errors/config.json");
    if !config_path.exists() {
        eprintln!(
            "Skipping e2e test: Config file not found at: {}",
            config_path.display()
        );
        return;
    }

    // Read and update the config file with the actual tool path
    let config_content = fs::read_to_string(config_path).expect("Failed to read config file");
    let mut config_json: serde_json::Value =
        serde_json::from_str(&config_content).expect("Failed to parse config file");

    // Update fmdevtool_path in config to use the actual tool path
    config_json["fmdevtool_path"] = serde_json::Value::String(tool_path.clone());

    // Write updated config to a temp file in the output directory
    let temp_config = output_dir.join("temp_config.json");
    fs::write(
        &temp_config,
        serde_json::to_string_pretty(&config_json).unwrap(),
    )
    .expect("Failed to write temp config file");

    use std::io::Write;
    let mut child = Command::new(binary_path)
        .arg("--config_path")
        .arg(temp_config.to_string_lossy().as_ref())
        .arg("--output_file_path")
        .arg(output_dir.to_string_lossy().as_ref())
        .arg("--max_concurrent")
        .arg("3")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Write empty input (just newline) in case any prompts appear
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"\n").ok();
        stdin.flush().ok();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    // Check that the command succeeded
    assert!(
        output.status.success(),
        "Command failed with status: {}\nStdout: {}\nStderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Compare all output CSV files
    let csv_files = [
        "fm-size-dbs.csv",
        "fm-size-fields.csv",
        "fm-size-durations.csv",
    ];
    let expected_dir =
        Path::new("tests/e2e/expected_output/full_run_recursive_multithreaded_cli_args");

    for csv_file in &csv_files {
        let actual_path = output_dir.join(csv_file);
        let expected_path = expected_dir.join(csv_file);

        assert!(
            actual_path.exists(),
            "Expected output file {:?} does not exist",
            actual_path
        );

        assert!(
            expected_path.exists(),
            "Expected reference file {:?} does not exist",
            expected_path
        );

        compare_csv_files(&actual_path, &expected_path)
            .unwrap_or_else(|e| panic!("CSV comparison failed for {}: {}", csv_file, e));
    }
}
