use crate::data_models::{CommandDuration, Field, Table};
use crate::logger::Logger;
use anyhow::Context;
use chrono::Local;
use encoding_rs::{UTF_16LE, UTF_8};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::time::Instant;

const CHECK_TABLE_NAME: &str = "__checking_if_file_can_be_opened__";

/// Parameters for querying field and index sizes
pub struct FieldIndexQueryParams<'a> {
    pub file_path: &'a str,
    pub table_name: &'a str,
    pub account_name: Option<&'a str>,
    pub password: Option<&'a str>,
    pub ear_key: Option<&'a str>,
    pub thread_id: usize,
    pub logger: &'a Logger,
}

pub struct FMDeveloperTool {
    path: String,
    version: String,
}

impl FMDeveloperTool {
    const MIN_SUPPORTED_VERSION: &str = "21.1.1";
    const MAX_SUPPORTED_VERSION: &str = "26.0.1";

    fn min_supported_version_number() -> u64 {
        // Handles versions in any format: 21, 21.1, 21.1.1, 21.1.1.4
        Self::version_to_number(Self::MIN_SUPPORTED_VERSION)
    }

    fn max_supported_version_number() -> u64 {
        // For max version, we need to allow any build number
        // Handles versions in any format: 26, 26.0, 26.0.1, 26.0.1.4
        // Always sets build to 9999 to allow any build number
        let parts: Vec<&str> = Self::MAX_SUPPORTED_VERSION.split('.').collect();
        let major: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let build: u64 = 9999; // Always use 9999 to allow any build number

        // Format: {major}{minor:04d}{patch:04d}{build:04d}
        major * 1_000_000_000_000 + minor * 100_000_000 + patch * 10_000 + build
    }

    pub fn new(path: String) -> anyhow::Result<Self> {
        let version = Self::get_version(&path)?;
        Ok(Self { path, version })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn version_to_number(version_str: &str) -> u64 {
        let parts: Vec<&str> = version_str.split('.').collect();

        let major: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let build: u64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

        // Format: {major}{minor:04d}{patch:04d}{build:04d}
        // Example: 21.1.1   -> 2100010010000
        // Example: 21.1.1.4 -> 2100010010004
        major * 1_000_000_000_000 + minor * 100_000_000 + patch * 10_000 + build
    }

    pub fn is_version_supported(&self) -> bool {
        Self::check_version_supported(&self.version)
    }

    pub fn get_supported_versions_string() -> String {
        format!(
            "{} through {}",
            Self::MIN_SUPPORTED_VERSION,
            Self::MAX_SUPPORTED_VERSION
        )
    }

    /// Check if version is supported. Public for testing.
    pub fn check_version_supported(version_str: &str) -> bool {
        let version_num = Self::version_to_number(version_str);
        let min_version = Self::min_supported_version_number();
        let max_version = Self::max_supported_version_number();

        // Supported range: 21.1.1.x through 26.0.1.x
        version_num >= min_version && version_num <= max_version
    }

    fn get_version(path: &str) -> anyhow::Result<String> {
        let output = Command::new(path)
            .arg("--version")
            .output()
            .with_context(|| format!("Failed to execute FMDeveloperTool at: {}", path))?;

        if !output.status.success() {
            anyhow::bail!("FMDeveloperTool --version failed");
        }

        let version = String::from_utf8(output.stdout)
            .context("Failed to parse FMDeveloperTool version")?
            .trim()
            .to_string();

        Ok(version)
    }

    pub fn check_file_access(
        &self,
        file_path: &str,
        account_name: Option<&str>,
        password: Option<&str>,
        ear_key: Option<&str>,
        thread_id: usize,
        logger: &Logger,
    ) -> anyhow::Result<()> {
        let mut cmd = Command::new(&self.path);
        cmd.arg("--sortBySize")
            .arg(file_path)
            .arg(account_name.unwrap_or(""))
            .arg(password.unwrap_or(""))
            .arg("-csv_format")
            .arg("-target_tablename")
            .arg(CHECK_TABLE_NAME);

        if let Some(key) = ear_key {
            cmd.arg("-encryption_key").arg(key);
        }

        // Log the command before execution
        let db_name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let output = cmd.output().context("Failed to execute FMDeveloperTool")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);

        // Filter out the "Table not found" message and replace with our custom message
        let check_table_pattern = CHECK_TABLE_NAME;
        let filtered_output = output_str
            .lines()
            .filter(|line| {
                !(line.contains("Table not found") && line.contains(check_table_pattern))
            })
            .collect::<Vec<_>>()
            .join("\n");
        let filtered_error = error_str
            .lines()
            .filter(|line| {
                !(line.contains("Table not found") && line.contains(check_table_pattern))
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Log filtered output
        if !filtered_output.trim().is_empty() {
            logger.log_now(db_name, None, Some(thread_id), &filtered_output)?;
        }
        if !filtered_error.trim().is_empty() {
            logger.log_now(db_name, None, Some(thread_id), &filtered_error)?;
        }

        if !output.status.success() {
            let error_msg = if error_str.contains("File already open")
                || error_str.contains("File permission problem")
            {
                format!("❌ File is locked: {}", file_path)
            } else {
                format!("❌ Failed to access file: {}\n{}", file_path, error_str)
            };
            anyhow::bail!(error_msg);
        }

        // Check for "Table not found" which is expected for our test table
        // If found, log our custom success message
        if output_str.contains("Table not found") || error_str.contains("Table not found") {
            logger.log_now(
                db_name,
                None,
                Some(thread_id),
                "✅ Confirmed file is not locked and can be opened",
            )?;
        }

        Ok(())
    }

    pub fn get_tables(
        &self,
        file_path: &str,
        account_name: Option<&str>,
        password: Option<&str>,
        ear_key: Option<&str>,
        thread_id: usize,
        logger: &Logger,
    ) -> anyhow::Result<Vec<String>> {
        let db_name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        logger.log_now(
            db_name,
            None,
            Some(thread_id),
            &format!("Getting table list from XML for: {}", file_path),
        )?;

        // Create temporary XML file (persists for debugging)
        let temp_dir = env::temp_dir();
        let xml_filename = format!("fm-size-{}-{}.xml", db_name, thread_id);
        let xml_path = temp_dir.join(xml_filename);

        let mut cmd = Command::new(&self.path);
        cmd.arg("--saveAsXML")
            .arg(file_path)
            .arg(account_name.unwrap_or(""))
            .arg(password.unwrap_or(""))
            .arg("-target_filename")
            .arg(&xml_path)
            .arg("-f");

        if let Some(key) = ear_key {
            cmd.arg("-encryption_key").arg(key);
        }

        // Log the command before execution
        let command = format!(
            "{} --saveAsXML {} {} {} -target_filename {} -f {}",
            self.path,
            file_path,
            account_name.unwrap_or(""),
            "REDACTED",
            xml_path.display(),
            if ear_key.is_some() {
                " -encryption_key REDACTED"
            } else {
                ""
            }
        );
        logger.log_now(
            db_name,
            None,
            Some(thread_id),
            &format!("Executing: {}", command),
        )?;

        let output = cmd.output().context("Failed to execute FMDeveloperTool")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);

        if !output_str.is_empty() {
            logger.log_now(db_name, None, Some(thread_id), &output_str)?;
        }
        if !error_str.is_empty() {
            logger.log_now(db_name, None, Some(thread_id), &error_str)?;
        }

        if !output.status.success() {
            anyhow::bail!("Failed to generate XML file: {}", error_str);
        }

        // Parse XML to extract table names
        let table_names = Self::parse_table_names_from_xml(&xml_path)
            .with_context(|| format!("Failed to parse XML file: {}", xml_path.display()))?;

        logger.log_now(
            db_name,
            None,
            Some(thread_id),
            &format!("Found {} table(s) in XML", table_names.len()),
        )?;

        Ok(table_names)
    }

    pub fn get_field_sizes(
        &self,
        params: &FieldIndexQueryParams,
    ) -> anyhow::Result<(Vec<Field>, CommandDuration)> {
        let start_instant = Instant::now();
        let start_timestamp = Local::now();
        let db_name = std::path::Path::new(params.file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        params.logger.log(
            Some(start_timestamp),
            db_name,
            Some(params.table_name),
            Some(params.thread_id),
            &format!("Getting field sizes for: {}", params.file_path),
        )?;

        let mut cmd = Command::new(&self.path);
        cmd.arg("--sortBySize")
            .arg(params.file_path)
            .arg(params.account_name.unwrap_or(""))
            .arg(params.password.unwrap_or(""))
            .arg("-csv_format")
            .arg("-target_tablename")
            .arg(params.table_name);

        if let Some(key) = params.ear_key {
            cmd.arg("-encryption_key").arg(key);
        }

        // Log the command before execution
        let command = format!(
            "{} --sortBySize {} {} {} -csv_format -target_tablename {}{}",
            self.path,
            params.file_path,
            params.account_name.unwrap_or(""),
            "REDACTED",
            params.table_name,
            if params.ear_key.is_some() {
                " -encryption_key REDACTED"
            } else {
                ""
            }
        );
        params.logger.log_now(
            db_name,
            Some(params.table_name),
            Some(params.thread_id),
            &format!("Executing: {}", command),
        )?;

        let output = cmd.output().context("Failed to execute FMDeveloperTool")?;
        let end_timestamp = Local::now();
        let duration = start_instant.elapsed().as_secs_f64();

        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);

        params.logger.log(
            Some(end_timestamp),
            db_name,
            Some(params.table_name),
            Some(params.thread_id),
            &output_str,
        )?;
        if !error_str.is_empty() {
            params.logger.log_now(
                db_name,
                Some(params.table_name),
                Some(params.thread_id),
                &error_str,
            )?;
        }

        if !output.status.success() {
            anyhow::bail!("Failed to get field sizes: {}", error_str);
        }

        let fields = Self::parse_field_sizes_csv_impl(&output_str)?;

        let duration_record = CommandDuration {
            thread_id: params.thread_id,
            db: db_name.to_string(),
            table: Some(params.table_name.to_string()),
            command,
            query_target: "fields".to_string(),
            duration,
            fmdevtool_ver: self.version.clone(),
            start: start_timestamp,
            end: end_timestamp,
        };

        Ok((fields, duration_record))
    }

    pub fn get_index_sizes(
        &self,
        params: &FieldIndexQueryParams,
    ) -> anyhow::Result<(Vec<Field>, CommandDuration)> {
        let start_instant = Instant::now();
        let start_timestamp = Local::now();
        let db_name = std::path::Path::new(params.file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        params.logger.log(
            Some(start_timestamp),
            db_name,
            Some(params.table_name),
            Some(params.thread_id),
            &format!("Getting index sizes for: {}", params.file_path),
        )?;

        let mut cmd = Command::new(&self.path);
        cmd.arg("--sortBySize")
            .arg(params.file_path)
            .arg(params.account_name.unwrap_or(""))
            .arg(params.password.unwrap_or(""))
            .arg("-csv_format")
            .arg("-target_tablename")
            .arg(params.table_name)
            .arg("-query_index");

        if let Some(key) = params.ear_key {
            cmd.arg("-encryption_key").arg(key);
        }

        // Log the command before execution
        let command = format!(
            "{} --sortBySize {} {} {} -csv_format -target_tablename {} -query_index{}",
            self.path,
            params.file_path,
            params.account_name.unwrap_or(""),
            "REDACTED",
            params.table_name,
            if params.ear_key.is_some() {
                " -encryption_key REDACTED"
            } else {
                ""
            }
        );
        params.logger.log_now(
            db_name,
            Some(params.table_name),
            Some(params.thread_id),
            &format!("Executing: {}", command),
        )?;

        let output = cmd.output().context("Failed to execute FMDeveloperTool")?;
        let end_timestamp = Local::now();
        let duration = start_instant.elapsed().as_secs_f64();

        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);

        params.logger.log(
            Some(end_timestamp),
            db_name,
            Some(params.table_name),
            Some(params.thread_id),
            &output_str,
        )?;
        if !error_str.is_empty() {
            params.logger.log_now(
                db_name,
                Some(params.table_name),
                Some(params.thread_id),
                &error_str,
            )?;
        }

        if !output.status.success() {
            anyhow::bail!("Failed to get index sizes: {}", error_str);
        }

        let fields = Self::parse_index_sizes_csv_impl(&output_str)?;

        let duration_record = CommandDuration {
            thread_id: params.thread_id,
            db: db_name.to_string(),
            table: Some(params.table_name.to_string()),
            command,
            query_target: "indexes".to_string(),
            duration,
            fmdevtool_ver: self.version.clone(),
            start: start_timestamp,
            end: end_timestamp,
        };

        Ok((fields, duration_record))
    }

    /// Filter CSV lines to remove noise. Public for testing.
    pub fn filter_csv_lines(csv: &str) -> String {
        Self::filter_csv_lines_impl(csv)
    }

    fn filter_csv_lines_impl(csv: &str) -> String {
        csv.lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Include lines that:
                // 1. Start with a quoted field (e.g., "FieldName" or "TableName")
                // 2. Are header lines (contain comma and look like CSV headers)
                // 3. Are empty (will be handled by CSV parser)
                trimmed.is_empty()
                    || trimmed.starts_with('"')
                    || (trimmed.contains(',')
                        && !trimmed.contains("doesn't")
                        && !trimmed.contains("Table not found"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse table sizes from CSV. Public for testing.
    pub fn parse_table_sizes_csv(csv: &str) -> anyhow::Result<Vec<Table>> {
        Self::parse_table_sizes_csv_impl(csv)
    }

    fn parse_table_sizes_csv_impl(csv: &str) -> anyhow::Result<Vec<Table>> {
        let filtered_csv = Self::filter_csv_lines_impl(csv);
        let mut reader = csv::Reader::from_reader(filtered_csv.as_bytes());
        let mut tables = Vec::new();

        for result in reader.records() {
            let record = result.context("Failed to parse CSV record")?;
            if record.len() < 3 {
                continue;
            }

            let table_name = record.get(0).unwrap_or("").trim_matches('"').to_string();
            let size_str = record.get(1).unwrap_or("0").trim_matches('"');
            let size = size_str.parse::<u64>().unwrap_or(0);

            tables.push(Table {
                name: table_name,
                size_bytes: size,
                fields: Vec::new(),
            });
        }

        Ok(tables)
    }

    fn parse_table_names_from_xml(xml_path: &std::path::Path) -> anyhow::Result<Vec<String>> {
        // Read the entire file as bytes
        let mut file = File::open(xml_path)
            .with_context(|| format!("Failed to open XML file: {}", xml_path.display()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .with_context(|| format!("Failed to read XML file: {}", xml_path.display()))?;

        // Check for UTF-16LE BOM (0xFF 0xFE) and convert to UTF-8
        let (utf8_content, _, _) = if buffer.starts_with(&[0xFF, 0xFE]) {
            // UTF-16LE with BOM
            UTF_16LE.decode(&buffer[2..])
        } else if buffer.len() >= 2 && buffer[0] != 0x3C {
            // Might be UTF-16LE without BOM (check if first byte isn't '<' which would be 0x3C in UTF-8)
            UTF_16LE.decode(&buffer)
        } else {
            // Assume UTF-8
            UTF_8.decode(&buffer)
        };

        let utf8_string = utf8_content.into_owned();

        // Parse the UTF-8 string with quick-xml
        let mut reader = Reader::from_str(&utf8_string);
        reader.trim_text(true);

        let mut table_names = Vec::new();
        let mut buf = Vec::new();
        let mut in_base_table_catalog = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"BaseTableCatalog" => {
                            in_base_table_catalog = true;
                        }
                        b"BaseTable" if in_base_table_catalog => {
                            // Extract name attribute
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"name" {
                                    match attr.unescape_value() {
                                        Ok(name_bytes) => {
                                            let name_str = name_bytes.to_string();
                                            if !name_str.is_empty() {
                                                table_names.push(name_str);
                                            }
                                        }
                                        Err(_) => {
                                            // Fallback to raw value
                                            let raw_value = attr.value.as_ref();
                                            let name_str =
                                                String::from_utf8_lossy(raw_value).to_string();
                                            if !name_str.is_empty() {
                                                table_names.push(name_str);
                                            }
                                        }
                                    }
                                    break; // Found name attribute, no need to check others
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().as_ref() == b"BaseTableCatalog" {
                        in_base_table_catalog = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    anyhow::bail!("XML parsing error: {}", e);
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(table_names)
    }

    #[allow(dead_code)]
    fn parse_table_names_csv_impl(csv: &str) -> anyhow::Result<Vec<String>> {
        let filtered_csv = Self::filter_csv_lines_impl(csv);
        let mut reader = csv::Reader::from_reader(filtered_csv.as_bytes());
        let mut table_names = Vec::new();

        for result in reader.records() {
            let record = result.context("Failed to parse CSV record")?;
            if record.is_empty() {
                continue;
            }

            let table_name = record.get(0).unwrap_or("").trim_matches('"').to_string();
            if !table_name.is_empty() {
                table_names.push(table_name);
            }
        }

        Ok(table_names)
    }

    /// Parse field sizes from CSV. Public for testing.
    pub fn parse_field_sizes_csv(csv: &str) -> anyhow::Result<Vec<Field>> {
        Self::parse_field_sizes_csv_impl(csv)
    }

    fn parse_field_sizes_csv_impl(csv: &str) -> anyhow::Result<Vec<Field>> {
        let filtered_csv = Self::filter_csv_lines_impl(csv);
        let mut reader = csv::Reader::from_reader(filtered_csv.as_bytes());
        let mut fields = Vec::new();

        for result in reader.records() {
            let record = result.context("Failed to parse CSV record")?;
            if record.len() < 3 {
                continue;
            }

            let field_name = record.get(0).unwrap_or("").trim_matches('"').to_string();
            let size_str = record.get(1).unwrap_or("0").trim_matches('"');
            let size = size_str.parse::<u64>().unwrap_or(0);

            fields.push(Field {
                name: field_name,
                field_size_bytes: size,
                value_index_size_bytes: 0,
                word_index_size_bytes: 0,
            });
        }

        Ok(fields)
    }

    /// Parse index sizes from CSV. Public for testing.
    pub fn parse_index_sizes_csv(csv: &str) -> anyhow::Result<Vec<Field>> {
        Self::parse_index_sizes_csv_impl(csv)
    }

    fn parse_index_sizes_csv_impl(csv: &str) -> anyhow::Result<Vec<Field>> {
        let filtered_csv = Self::filter_csv_lines_impl(csv);
        let mut reader = csv::Reader::from_reader(filtered_csv.as_bytes());
        let mut fields = Vec::new();

        for result in reader.records() {
            let record = result.context("Failed to parse CSV record")?;
            if record.len() < 5 {
                continue;
            }

            let field_name = record.get(0).unwrap_or("").trim_matches('"').to_string();
            let value_size_str = record.get(1).unwrap_or("0").trim_matches('"');
            let value_size = value_size_str.parse::<u64>().unwrap_or(0);
            let word_size_str = record.get(3).unwrap_or("0").trim_matches('"');
            let word_size = word_size_str.parse::<u64>().unwrap_or(0);

            fields.push(Field {
                name: field_name,
                field_size_bytes: 0, // Index sizes don't have field size
                value_index_size_bytes: value_size,
                word_index_size_bytes: word_size,
            });
        }

        Ok(fields)
    }
}
