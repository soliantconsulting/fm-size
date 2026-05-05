# fm-size

Version 0.2.0

The Claris FileMaker CLI developer tool (FMDeveloperTool) can be used to perform several actions, including seeing how tables, fields, and indexes contribute to the size of a FileMaker database file.

However, the tool must be run multiple times to acquire a complete picture.

fm-size is a wrapper for FMDeveloperTool that runs all the commands necessary to get a comprehensive file size view.

Multiple FileMaker database files can be targeted in parallel.

## Quick Start

### Option 1: Config file (recommended for multiple databases)

Create a `config.json` file listing your databases and credentials:

```json
{
  "fmdevtool_path": "/path/to/FMDeveloperTool",
  "databases": [
    {
      "path_to_db": "/path/to/database1.fmp12",
      "account_name": "admin",
      "password": "password",
      "ear_key": null
    },
    {
      "path_to_db": "/path/to/database2.fmp12",
      "account_name": null,
      "password": null,
      "ear_key": "encryption-key-here"
    }
  ]
}
```

Then run:

```bash
fm-size -c config.json
```

Add `-m` to process multiple databases in parallel:

```bash
fm-size -c config.json -m 4
```

### Option 2: Command line (single database or one-off run)

Provide the database path directly. If FMDeveloperTool is at its default location you don't need `-t`:

```bash
fm-size -i /path/to/database.fmp12
```

You will be prompted for account name, password, and encryption key (press Enter to skip any that aren't needed). This keeps credentials out of your command history.

With a custom FMDeveloperTool path:

```bash
fm-size -t /path/to/FMDeveloperTool -i /path/to/database.fmp12
```

## Output

The tool generates three CSV files in the output directory:

1. **fm-size-dbs.csv**: Overall database file sizes with processing times
2. **fm-size-fields.csv**: Size breakdown by field for each database
3. **fm-size-durations.csv**: Detailed timing information for each command executed

All sizes are reported in bytes. The CSV files always include `fmsize_ver` and `fmdevtool_ver` columns showing the versions used.

Output defaults to the directory of the first input file. Override with `-o`:

```bash
fm-size -c config.json -o /path/to/output/
```

## All Options

### Input

- `-i, --db_file_paths <PATHS>...`: Paths to search for `.fmp12` files. Can be directories or individual `.fmp12` files
- `-c, --config_path <PATH>`: Path to JSON config file. When provided, credentials and database paths are read from the file — no prompts are shown. Overrides `-i`, `-t`, and authentication options.

### FMDeveloperTool

- `-t, --fmdevtool_path <PATH>`: Path to FMDeveloperTool executable. Defaults to the platform-standard location:
  - macOS: `/Library/FileMaker Server/Database Server/bin/FMDeveloperTool`
  - Windows: `C:\Program Files\FileMaker\FileMaker Server\Database Server\FMDeveloperTool.exe`
  - Linux: `/opt/FileMaker/FileMaker Server/Database Server/bin/FMDeveloperTool`

### Authentication (command-line mode only)

When not using `--config_path`, you will be prompted for:
- **Account name**: visible input (press Enter to skip)
- **Password**: concealed input (press Enter to skip)
- **Encryption key**: concealed input for EAR databases (press Enter to skip)

### Processing

- `-f, --db_filter <FILTER>`: Comma-separated list of `.fmp12` filenames to process; others are skipped
- `-r, --recurse`: Recursively search subdirectories for `.fmp12` files
- `-m, --max_concurrent <NUMBER>`: Maximum number of databases to process in parallel (default: 1)

### Output directory

- `-o, --output_file_path <PATH>`: Directory for output CSV files (defaults to directory of first input path)

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021)

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

The compiled binary will be at `target/release/fm-size` (or `target/debug/fm-size` for debug builds).

### Install

```bash
cargo install --path .
```

This installs the `fm-size` binary to your Cargo bin directory (typically `~/.cargo/bin/`).

### Cross-platform builds

The project includes a `Makefile` with targets for cross-compilation:

```bash
make build-linux      # Linux x86_64
make build-windows    # Windows x86_64
make build-mac-intel  # macOS Intel (x86_64)
make build-mac-m1     # macOS Apple Silicon (aarch64)
make build-all        # All of the above
```

Cross-compilation requires the corresponding Rust target to be installed, e.g.:

```bash
rustup target add x86_64-unknown-linux-gnu
```

### Development

```bash
make check   # Run formatter, linter, and tests
make fmt     # Format code
make clippy  # Run linter
make test    # Run tests
```
