use anyhow::Context;
use clap::{CommandFactory, Parser};
use fm_size::config::{Args, ProcessedConfig};
use fm_size::processor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Show help if no arguments provided
    if std::env::args().count() == 1 {
        Args::command().print_help()?;
        return Ok(());
    }

    let args = Args::parse();
    let config = ProcessedConfig::from_args(args)?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&config.output_file_path).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            config.output_file_path.display()
        )
    })?;

    processor::process(config).await
}
