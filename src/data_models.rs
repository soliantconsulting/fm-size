use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DatabaseFile {
    pub path: String,
    pub name: String, // without .fmp12 extension
    pub size_bytes: u64,
    pub account_name: Option<String>,
    pub password: Option<String>,
    pub ear_key: Option<String>,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub size_bytes: u64,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_size_bytes: u64,
    pub value_index_size_bytes: u64,
    pub word_index_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct CommandDuration {
    pub thread_id: usize,
    pub db: String,
    pub table: Option<String>,
    pub command: String,
    pub query_target: String,
    pub duration: f64,
    pub fmdevtool_ver: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub fmdevtool_path: Option<String>,
    pub databases: Vec<DatabaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path_to_db: String,
    #[serde(default)]
    pub account_name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub ear_key: Option<String>,
}
