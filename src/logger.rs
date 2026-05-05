use chrono::{DateTime, Local};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub struct Logger {
    log_file: Mutex<std::fs::File>,
}

impl Logger {
    pub fn new(log_path: &std::path::Path) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .with_context(|| format!("Failed to create log file: {}", log_path.display()))?;

        Ok(Self {
            log_file: Mutex::new(file),
        })
    }

    pub fn log(
        &self,
        timestamp: Option<DateTime<Local>>,
        db: &str,
        table: Option<&str>,
        thread_id: Option<usize>,
        line: &str,
    ) -> anyhow::Result<()> {
        let timestamp = timestamp.unwrap_or_else(Local::now);
        let timestamp_str = timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        let thread_prefix = if let Some(tid) = thread_id {
            format!("[DB{}] ", tid)
        } else {
            String::new()
        };

        let prefix = if db == "system" {
            // Omit [system] prefix for system messages
            format!("[{}] {}", timestamp_str, thread_prefix)
        } else if let Some(tbl) = table {
            format!("[{}] {}[{} → {}] ", timestamp_str, thread_prefix, db, tbl)
        } else {
            format!("[{}] {}[{}] ", timestamp_str, thread_prefix, db)
        };

        let full_line = format!("{}{}", prefix, line);

        // Write to stdout
        println!("{}", full_line);

        // Write to log file
        let mut file = self.log_file.lock().unwrap();
        writeln!(file, "{}", full_line)?;

        Ok(())
    }

    // Convenience method that always uses current time
    pub fn log_now(
        &self,
        db: &str,
        table: Option<&str>,
        thread_id: Option<usize>,
        line: &str,
    ) -> anyhow::Result<()> {
        self.log(None, db, table, thread_id, line)
    }

    pub fn log_error(&self, message: &str) -> anyhow::Result<()> {
        eprintln!("{}", message);
        let mut file = self.log_file.lock().unwrap();
        writeln!(file, "❌ {}", message)?;
        Ok(())
    }

    pub fn write_blank_line(&self) -> anyhow::Result<()> {
        let mut file = self.log_file.lock().unwrap();
        writeln!(file)?;
        Ok(())
    }
}

use anyhow::Context;
