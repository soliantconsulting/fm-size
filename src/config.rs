use crate::data_models::Config;
use anyhow::Context;
use clap::Parser;
use rpassword::prompt_password;
use serde_json;
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "fm-size")]
#[command(about = "Analyze FileMaker database file sizes")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    #[arg(
        short = 't',
        long = "fmdevtool_path",
        help = "Path to FMDeveloperTool (optional - will use default location if not specified)"
    )]
    pub fmdevtool_path: Option<String>,

    #[arg(
        short = 'i',
        long = "db_file_paths",
        help = "Paths to search for .fmp12 files (space-separated values). Can be directories or .fmp12 files"
    )]
    pub db_file_paths: Option<Vec<String>>,

    #[arg(
        short = 'r',
        long = "recurse",
        help = "Recursively search subdirectories for .fmp12 files"
    )]
    pub recurse: bool,

    #[arg(
        short = 'f',
        long = "db_filter",
        help = "Comma-separated list of .fmp12 files to process. Other files found will be skipped. Don't need to include the .fmp12 extension."
    )]
    pub db_filter: Option<String>,

    #[arg(
        short = 'c',
        long = "config_path",
        help = "Path to JSON config file (optional). If provided, overrides: fmdevtool_path (optional in config), db_file_paths, account_name, password, ear_key, db_filter. Use this json structure: { \"fmdevtool_path\": string|null, \"databases\": [ { \"path_to_db\": string, \"account_name\": string|null, \"password\": string|null, \"ear_key\": string|null } ] }"
    )]
    pub config_path: Option<String>,

    #[arg(
        short = 'o',
        long = "output_file_path",
        help = "Directory for output CSV files [default: current working directory]"
    )]
    pub output_file_path: Option<String>,

    #[arg(
        short = 'm',
        long = "max_concurrent",
        default_value = "1",
        help = "Maximum number of database files to process in parallel"
    )]
    pub max_concurrent: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessedConfig {
    pub fmdevtool_path: String,
    pub databases: Vec<DatabaseInfo>,
    pub output_file_path: PathBuf,
    pub max_concurrent: usize,
    pub recurse: bool,
    pub db_filter: Option<Vec<String>>,
    pub config_loaded: bool,
}

#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub path: String,
    pub account_name: Option<String>,
    pub password: Option<String>,
    pub ear_key: Option<String>,
}

impl ProcessedConfig {
    /// Get the default FMDeveloperTool path based on the operating system
    /// Public for testing
    pub fn get_default_fmdevtool_path() -> Option<String> {
        let path = match std::env::consts::OS {
            "windows" => {
                r"C:\Program Files\FileMaker\FileMaker Server\Database Server\FMDeveloperTool.exe"
            }
            "macos" => "/Library/FileMaker Server/Database Server/bin/FMDeveloperTool",
            "linux" => "/opt/FileMaker/FileMaker Server/Database Server/bin/FMDeveloperTool",
            _ => return None,
        };

        // Check if the file exists
        if PathBuf::from(path).exists() {
            Some(path.to_string())
        } else {
            None
        }
    }

    /// Resolve FMDeveloperTool path - if it's just a filename, try to find it in PATH
    fn resolve_fmdevtool_path(path: &str) -> anyhow::Result<String> {
        let path_buf = PathBuf::from(path);

        // If it's already an absolute path or contains directory separators, use it as-is
        if path_buf.is_absolute() || path.contains('/') || path.contains('\\') {
            return Ok(path.to_string());
        }

        // It's just a filename, try to find it in PATH
        // First try using `which` command (Unix-like systems)
        #[cfg(unix)]
        {
            if let Ok(output) = Command::new("which").arg(path).output() {
                if output.status.success() {
                    let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !resolved.is_empty() && PathBuf::from(&resolved).exists() {
                        return Ok(resolved);
                    }
                }
            }
        }

        // On Windows, try `where` command
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("where").arg(path).output() {
                if output.status.success() {
                    let resolved = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    if !resolved.is_empty() && PathBuf::from(&resolved).exists() {
                        return Ok(resolved);
                    }
                }
            }
        }

        // If not found in PATH, return the original path (will fail validation later)
        Ok(path.to_string())
    }

    pub fn from_args(args: Args) -> anyhow::Result<Self> {
        let mut fmdevtool_path = args.fmdevtool_path;
        let mut db_file_paths = args.db_file_paths;
        let mut db_filter = args.db_filter.map(|s| {
            s.split(',')
                .map(|x| {
                    let trimmed = x.trim().to_string();
                    if trimmed.is_empty() {
                        String::new()
                    } else if trimmed.ends_with(".fmp12") {
                        trimmed
                    } else {
                        format!("{}.fmp12", trimmed)
                    }
                })
                .filter(|x| !x.is_empty())
                .collect()
        });

        // Load config file if provided
        let mut databases = Vec::new();
        let (account_name, password, encryption_key) = if let Some(config_path) = &args.config_path
        {
            let config = Self::load_config_impl(config_path)?;
            // Only override if config provides a value
            if let Some(path) = config.fmdevtool_path {
                fmdevtool_path = Some(path);
            }
            db_file_paths = None; // Will be populated from config databases
            db_filter = None;

            for db_config in config.databases {
                databases.push(DatabaseInfo {
                    path: db_config.path_to_db,
                    account_name: db_config.account_name,
                    password: db_config.password,
                    ear_key: db_config.ear_key,
                });
            }
            (None, None, None) // Config file provides auth per database
        } else {
            // If not using config file, prompt for account_name, password, and encryption_key
            // Skip prompting if stdin is not a TTY (e.g., in tests or non-interactive environments)
            // In test environments, stdin might be a TTY but we still don't want to prompt
            // Check if we're in a test by looking at the executable name or environment
            let exe_name = std::env::args().next().unwrap_or_default().to_lowercase();
            let is_test = exe_name.contains("test")
                || exe_name.contains("config_unit_test")
                || exe_name.contains("config_test")
                || std::env::var("FM_SIZE_SKIP_PROMPTS").is_ok();

            // Only prompt if stdin is a TTY AND we're not in a test
            if std::io::stdin().is_terminal() && !is_test {
                let account_name = Self::prompt_account_name()?;
                let (password, encryption_key) = if account_name.is_some() {
                    let pwd = Self::prompt_password()?;
                    let key = Self::prompt_encryption_key()?;
                    (
                        if pwd.is_empty() { None } else { Some(pwd) },
                        if key.is_empty() { None } else { Some(key) },
                    )
                } else {
                    (None, None)
                };
                (account_name, password, encryption_key)
            } else {
                // Not a TTY or in test environment, skip prompting
                (None, None, None)
            }
        };

        // If no config was loaded, use CLI args
        if databases.is_empty() {
            if let Some(paths) = db_file_paths {
                for path in paths {
                    databases.push(DatabaseInfo {
                        path,
                        account_name: account_name.clone(),
                        password: password.clone(),
                        ear_key: encryption_key.clone(),
                    });
                }
            }
        }

        // If fmdevtool_path is still None, try default location
        let fmdevtool_path = if let Some(path) = fmdevtool_path {
            path
        } else {
            Self::get_default_fmdevtool_path().ok_or_else(|| {
                anyhow::anyhow!(
                    "fmdevtool_path is required. Either provide --fmdevtool_path, include it in config file, or install FileMaker Server at the default location:\n  Windows: C:\\Program Files\\FileMaker\\FileMaker Server\\Database Server\\FMDeveloperTool.exe\n  macOS: /Library/FileMaker Server/Database Server/bin/FMDeveloperTool\n  Linux: /opt/FileMaker/FileMaker Server/Database Server/bin/FMDeveloperTool"
                )
            })?
        };

        // Resolve the path - if it's just a filename, try to find it in PATH
        let fmdevtool_path = Self::resolve_fmdevtool_path(&fmdevtool_path)?;

        // Validate that the path exists
        if !PathBuf::from(&fmdevtool_path).exists() {
            anyhow::bail!("FMDeveloperTool not found at: {}", fmdevtool_path);
        }

        if databases.is_empty() {
            anyhow::bail!("No database files specified");
        }

        // Validate max_concurrent
        if args.max_concurrent == 0 {
            anyhow::bail!("max_concurrent must be greater than 0");
        }

        // Determine output path
        let output_file_path = if let Some(path) = args.output_file_path {
            PathBuf::from(path)
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        let config_loaded = args.config_path.is_some();

        Ok(Self {
            fmdevtool_path,
            databases,
            output_file_path,
            max_concurrent: args.max_concurrent,
            recurse: args.recurse,
            db_filter,
            config_loaded,
        })
    }

    /// Load config from file. Public for testing.
    pub fn load_config(path: &str) -> anyhow::Result<Config> {
        Self::load_config_impl(path)
    }

    fn load_config_impl(path: &str) -> anyhow::Result<Config> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;
        Ok(config)
    }

    /// Prompt for password with concealed input
    fn prompt_password() -> anyhow::Result<String> {
        prompt_password("Enter database password (press Enter to skip): ")
            .map_err(|e| anyhow::anyhow!("Failed to read password: {}", e))
    }

    /// Prompt for encryption key with concealed input
    fn prompt_encryption_key() -> anyhow::Result<String> {
        prompt_password("Enter EAR (encryption at rest) key. Ignored for databases that don't use EAR (press Enter to skip): ")
            .map_err(|e| anyhow::anyhow!("Failed to read encryption key: {}", e))
    }

    /// Prompt for account name with visible input
    fn prompt_account_name() -> anyhow::Result<Option<String>> {
        print!("Enter account name for database authentication (press Enter to skip): ");
        io::stdout()
            .flush()
            .map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| anyhow::anyhow!("Failed to read account name: {}", e))?;

        let trimmed = input.trim().to_string();
        Ok(if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        })
    }
}
