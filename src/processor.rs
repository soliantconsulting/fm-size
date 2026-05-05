use crate::config::ProcessedConfig;
use crate::csv_writer;
use crate::data_models::{CommandDuration, DatabaseFile};
use crate::file_finder;
use crate::fmdevtool::FMDeveloperTool;
use crate::logger::Logger;
use anyhow::Context;
use chrono::Local;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

pub async fn process(config: ProcessedConfig) -> anyhow::Result<()> {
    let overall_start_instant = Instant::now();
    let overall_start = Local::now();

    // Initialize FMDeveloperTool
    let tool = Arc::new(
        FMDeveloperTool::new(config.fmdevtool_path.clone())
            .context("Failed to initialize FMDeveloperTool")?,
    );
    let fmdevtool_ver = tool.version().to_string();
    let version_supported = tool.is_version_supported();

    // Initialize logger
    let log_path = config.output_file_path.join("fm-size.log");
    let logger = Arc::new(
        Logger::new(&log_path)
            .with_context(|| format!("Failed to create logger: {}", log_path.display()))?,
    );

    // Output version as first line
    logger.log(
        Some(overall_start),
        "system",
        None,
        None,
        &format!("fm-size version {}", env!("CARGO_PKG_VERSION")),
    )?;

    logger.write_blank_line()?;
    logger.write_blank_line()?;
    logger.log(
        Some(overall_start),
        "system",
        None,
        None,
        &format!("FMDeveloperTool version {}", fmdevtool_ver),
    )?;

    // Warn if version is not supported
    if !version_supported {
        let supported_versions = FMDeveloperTool::get_supported_versions_string();
        let warning = format!("‼️ Warning: fm-size has NOT been tested with FMDeveloperTool version {}. The results might not be correct. Supported versions include: {}.", fmdevtool_ver, supported_versions);
        logger.log_now("system", None, None, &warning)?;
    }

    // Find all database files
    let mut db_paths = Vec::new();
    if config.config_loaded {
        // Files come from config
        for db_info in &config.databases {
            let path = Path::new(&db_info.path);
            if !path.exists() {
                logger.log_error(&format!("File not found in config: {}", db_info.path))?;
                continue;
            }
            let size = fs::metadata(path)?.len();
            db_paths.push((path.to_path_buf(), size, db_info.clone()));
        }
    } else {
        // Find files from paths
        let paths: Vec<String> = config.databases.iter().map(|d| d.path.clone()).collect();
        let found_files =
            file_finder::find_fmp12_files(&paths, config.recurse, config.db_filter.as_deref())?;

        for (path, size) in found_files {
            // Use credentials from first database (they should all be the same from CLI)
            let db_info = config.databases.first().cloned().unwrap();
            db_paths.push((path, size, db_info));
        }
    }

    // Sort by size (largest first)
    db_paths.sort_by(|(_, a, _), (_, b, _)| b.cmp(a));

    logger.log_now(
        "system",
        None,
        None,
        &format!("Found {} database files to process", db_paths.len()),
    )?;

    // Pre-check all files (fail early if any are locked)
    if !db_paths.is_empty() {
        logger.log_now("system", None, None, "Pre-checking file access...")?;

        let check_semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        let mut check_handles = Vec::new();

        for (thread_id, (path, _, db_info)) in db_paths.iter().enumerate() {
            let thread_id = thread_id + 1; // Start from 1, not 0
            let semaphore_clone = check_semaphore.clone();
            let permit = semaphore_clone.acquire_owned().await?;
            let tool_clone = tool.clone();
            let logger_clone = logger.clone();
            let path_str = path.to_string_lossy().to_string();
            let account_name = db_info.account_name.clone();
            let password = db_info.password.clone();
            let ear_key = db_info.ear_key.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                tokio::task::spawn_blocking(move || {
                    tool_clone.check_file_access(
                        &path_str,
                        account_name.as_deref(),
                        password.as_deref(),
                        ear_key.as_deref(),
                        thread_id,
                        &logger_clone,
                    )
                })
                .await
            });

            check_handles.push((path.to_string_lossy().to_string(), handle));
        }

        // Collect results and fail early if any check fails
        for (path_str, handle) in check_handles {
            match handle.await {
                Ok(Ok(Ok(()))) => {} // Success
                Ok(Ok(Err(e))) => {
                    anyhow::bail!("Failed to access {}: {}", path_str, e);
                }
                Ok(Err(e)) => {
                    anyhow::bail!("Task panicked for {}: {:?}", path_str, e);
                }
                Err(e) => {
                    anyhow::bail!("Failed to spawn blocking task for {}: {:?}", path_str, e);
                }
            }
        }

        logger.log_now("system", None, None, "🚀 All files accessible")?;
        logger.write_blank_line()?;
    }

    // Process files in parallel
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
    let mut handles = Vec::new();

    for (thread_id, (path, _, db_info)) in db_paths.into_iter().enumerate() {
        let thread_id = thread_id + 1; // Start from 1, not 0
        let semaphore_clone = semaphore.clone();
        let permit = semaphore_clone.acquire_owned().await?;
        let tool_clone = tool.clone();
        let logger_clone = logger.clone();
        let db_info_clone = db_info.clone();
        let path_str = path.to_string_lossy().to_string();
        let path_str_for_handle = path_str.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            process_database(
                &tool_clone,
                &path_str_for_handle,
                db_info_clone,
                thread_id,
                &logger_clone,
            )
            .await
        });

        handles.push((path_str, handle));
    }

    // Collect results
    let mut databases = Vec::new();
    let mut all_durations = Vec::new();

    for (path_str, handle) in handles {
        match handle.await {
            Ok(Ok((db, durations))) => {
                databases.push(db);
                all_durations.extend(durations);
            }
            Ok(Err(e)) => {
                anyhow::bail!("Failed to process {}: {}", path_str, e);
            }
            Err(e) => {
                anyhow::bail!("Task panicked for {}: {:?}", path_str, e);
            }
        }
    }

    // Write CSV files
    let overall_end = Local::now();
    let fmsize_version = env!("CARGO_PKG_VERSION");
    let csv_config = csv_writer::CsvWriterConfig {
        output_path: &config.output_file_path,
        fmdevtool_ver: &fmdevtool_ver,
        fmsize_version,
    };
    csv_writer::write_csv_files(
        &csv_config,
        &databases,
        &all_durations,
        overall_start,
        overall_end,
    )?;

    let overall_duration = overall_start_instant.elapsed().as_secs();

    // Warn again if version is not supported (before "Processing complete")
    if !version_supported {
        let supported_versions = FMDeveloperTool::get_supported_versions_string();
        let warning = format!("‼️ Warning: fm-size has NOT been tested with FMDeveloperTool version {}. The results might not be correct. Supported versions include: {}.", fmdevtool_ver, supported_versions);
        logger.log_now("system", None, None, &warning)?;
    }

    logger.log_now(
        "system",
        None,
        None,
        &format!("Processing complete in {} seconds", overall_duration),
    )?;
    logger.write_blank_line()?;

    Ok(())
}

async fn process_database(
    tool: &FMDeveloperTool,
    file_path: &str,
    db_info: crate::config::DatabaseInfo,
    thread_id: usize,
    logger: &Logger,
) -> anyhow::Result<(DatabaseFile, Vec<CommandDuration>)> {
    let start_time = Instant::now();
    let file_path_buf = Path::new(file_path);
    let db_name = file_path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    logger.log_now(
        &db_name,
        None,
        Some(thread_id),
        &format!("Starting processing: {}", file_path),
    )?;

    let mut durations = Vec::new();

    // Get file size
    let file_size = fs::metadata(file_path_buf)?.len();

    // Get table list
    let table_names = tool.get_tables(
        file_path,
        db_info.account_name.as_deref(),
        db_info.password.as_deref(),
        db_info.ear_key.as_deref(),
        thread_id,
        logger,
    )?;

    // Create table objects and get field and index sizes for each
    let mut tables = Vec::new();
    for table_name in table_names {
        let mut table = crate::data_models::Table {
            name: table_name.clone(),
            size_bytes: 0,
            fields: Vec::new(),
        };
        // Get field sizes
        let field_params = crate::fmdevtool::FieldIndexQueryParams {
            file_path,
            table_name: &table.name,
            account_name: db_info.account_name.as_deref(),
            password: db_info.password.as_deref(),
            ear_key: db_info.ear_key.as_deref(),
            thread_id,
            logger,
        };
        let (mut fields, duration) = tool.get_field_sizes(&field_params)?;
        durations.push(duration);

        // Get index sizes
        let index_params = crate::fmdevtool::FieldIndexQueryParams {
            file_path,
            table_name: &table.name,
            account_name: db_info.account_name.as_deref(),
            password: db_info.password.as_deref(),
            ear_key: db_info.ear_key.as_deref(),
            thread_id,
            logger,
        };
        let (index_fields, duration) = tool.get_index_sizes(&index_params)?;
        durations.push(duration);

        // Merge index data into fields
        let mut index_map: std::collections::HashMap<String, (u64, u64)> = index_fields
            .into_iter()
            .map(|f| {
                (
                    f.name.clone(),
                    (f.value_index_size_bytes, f.word_index_size_bytes),
                )
            })
            .collect();

        for field in &mut fields {
            if let Some((value_idx, word_idx)) = index_map.remove(&field.name) {
                field.value_index_size_bytes = value_idx;
                field.word_index_size_bytes = word_idx;
            }
        }

        // Log any unmatched index fields
        if !index_map.is_empty() {
            let unmatched: Vec<String> = index_map.keys().cloned().collect();
            logger.log_now(
                &db_name,
                Some(&table.name),
                Some(thread_id),
                &format!(
                    "⚠️ Warning: Found {} index field(s) that don't match any field names: {}",
                    unmatched.len(),
                    unmatched.join(", ")
                ),
            )?;
        }

        table.fields = fields;
        tables.push(table);
    }

    let time_to_process = start_time.elapsed().as_secs_f64();

    let database = DatabaseFile {
        path: file_path.to_string(),
        name: db_name,
        size_bytes: file_size,
        account_name: db_info.account_name,
        password: db_info.password,
        ear_key: db_info.ear_key,
        tables,
    };

    logger.log_now(
        &database.name,
        None,
        Some(thread_id),
        &format!("Completed processing in {:.2} seconds", time_to_process),
    )?;

    Ok((database, durations))
}
