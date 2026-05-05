use crate::data_models::{CommandDuration, DatabaseFile};
use anyhow::Context;
use chrono::{DateTime, Local};
use csv::Writer;
use std::path::Path;

/// Configuration for CSV writing operations
pub struct CsvWriterConfig<'a> {
    pub output_path: &'a Path,
    pub fmdevtool_ver: &'a str,
    pub fmsize_version: &'a str,
}

pub fn write_csv_files(
    config: &CsvWriterConfig,
    databases: &[DatabaseFile],
    durations: &[CommandDuration],
    overall_start: DateTime<Local>,
    overall_end: DateTime<Local>,
) -> anyhow::Result<()> {
    // Write fm-size-dbs.csv
    write_file_sizes(config, databases)?;

    // Write fm-size-fields.csv
    write_field_sizes(config, databases)?;

    // Write fm-size-durations.csv
    write_durations(config, durations, overall_start, overall_end)?;

    Ok(())
}

fn write_file_sizes(config: &CsvWriterConfig, databases: &[DatabaseFile]) -> anyhow::Result<()> {
    let file_path = config.output_path.join("fm-size-dbs.csv");
    let mut wtr = Writer::from_path(&file_path)
        .with_context(|| format!("Failed to create file: {}", file_path.display()))?;

    // Build header
    let header = vec!["#", "db", "bytes", "fmsize_ver", "fmdevtool_ver"];
    wtr.write_record(&header)?;

    let mut db_num = 0;

    for db in databases {
        db_num += 1;

        let db_num_str = db_num.to_string();
        let bytes_str = db.size_bytes.to_string();

        // Build row
        let row: Vec<&str> = vec![
            &db_num_str,
            &db.name,
            &bytes_str,
            config.fmsize_version,
            config.fmdevtool_ver,
        ];
        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    Ok(())
}

fn write_field_sizes(config: &CsvWriterConfig, databases: &[DatabaseFile]) -> anyhow::Result<()> {
    let file_path = config.output_path.join("fm-size-fields.csv");
    // Delete existing file to avoid conflicts
    let _ = std::fs::remove_file(&file_path);
    let mut wtr = Writer::from_path(&file_path)
        .with_context(|| format!("Failed to create file: {}", file_path.display()))?;

    // Build header
    let header = vec![
        "#",
        "db",
        "#",
        "table",
        "#",
        "field",
        "field_bytes",
        "value_index_bytes",
        "word_index_bytes",
        "fmsize_ver",
        "fmdevtool_ver",
    ];
    let expected_field_count = header.len();
    wtr.write_record(&header)?;

    let mut db_num = 0;
    for db in databases {
        db_num += 1;

        let mut table_num = 0;

        for table in &db.tables {
            // Only process tables that have fields
            if table.fields.is_empty() {
                continue;
            }

            table_num += 1;

            let mut field_num = 0;
            for field in &table.fields {
                field_num += 1;

                // Build row - store all formatted strings first
                let db_num_str = db_num.to_string();
                let table_num_str = table_num.to_string();
                let field_num_str = field_num.to_string();
                let field_bytes_str = field.field_size_bytes.to_string();
                let value_index_bytes_str = field.value_index_size_bytes.to_string();
                let word_index_bytes_str = field.word_index_size_bytes.to_string();

                let row: Vec<&str> = vec![
                    &db_num_str,
                    &db.name,
                    &table_num_str,
                    &table.name,
                    &field_num_str,
                    &field.name,
                    &field_bytes_str,
                    &value_index_bytes_str,
                    &word_index_bytes_str,
                    config.fmsize_version,
                    config.fmdevtool_ver,
                ];
                // Ensure row has correct number of fields
                if row.len() != expected_field_count {
                    anyhow::bail!(
                        "Field row has {} fields but header has {} fields",
                        row.len(),
                        expected_field_count
                    );
                }
                wtr.write_record(&row)?;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}

fn write_durations(
    config: &CsvWriterConfig,
    durations: &[CommandDuration],
    _overall_start: DateTime<Local>,
    _overall_end: DateTime<Local>,
) -> anyhow::Result<()> {
    let file_path = config.output_path.join("fm-size-durations.csv");
    // Delete existing file to avoid conflicts
    let _ = std::fs::remove_file(&file_path);
    let mut wtr = Writer::from_path(&file_path)
        .with_context(|| format!("Failed to create file: {}", file_path.display()))?;
    let include_command = false;

    // Build header
    let mut header = vec![
        "thread_id",
        "db",
        "table",
        "query_target",
        "duration_in_sec",
    ];
    if include_command {
        header.push("command");
    }
    header.push("fmsize_ver");
    header.push("fmdevtool_ver");
    let expected_field_count = header.len();
    wtr.write_record(&header)?;

    for duration in durations {
        // Build row
        let table_str = duration.table.clone().unwrap_or_default();
        let duration_str = format!("{:.2}", duration.duration);
        let mut row: Vec<String> = vec![
            duration.thread_id.to_string(),
            duration.db.clone(),
            table_str,
            duration.query_target.clone(),
            duration_str.clone(),
        ];
        if include_command {
            row.push(duration.command.clone());
        }
        row.push(config.fmsize_version.to_string());
        row.push(config.fmdevtool_ver.to_string());
        // Convert Vec<String> to Vec<&str> for CSV writer
        let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        // Ensure row has correct number of fields
        if row_refs.len() != expected_field_count {
            anyhow::bail!("Duration row has {} fields but header has {} fields. db={}, table={:?}, query_target={}", row_refs.len(), expected_field_count, duration.db, duration.table, duration.query_target);
        }
        wtr.write_record(&row_refs)
            .with_context(|| format!("Failed to write duration record for db={}, table={:?}, query_target={}, row_len={}, expected={}", duration.db, duration.table, duration.query_target, row_refs.len(), expected_field_count))?;
    }

    wtr.flush()?;
    Ok(())
}
