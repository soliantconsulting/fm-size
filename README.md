# fm-size

Version 0.2.0

The Claris FileMaker CLI developer tool (FMDeveloperTool) can be used to perform several actions, including seeing how tables, fields, and indexes contribute to the size of a FileMaker database file.

However, the tool must be run multiple times to acquire a complete picture.

fm-size is a wrapper for FMDeveloperTool that runs all the commands necessary to get a comprehensive file size view.

Multiple FileMaker database files can be targeted in parallel.

## Usage

### Basic Usage

The minimum required arguments are the path(s) to FileMaker database files. If FMDeveloperTool is installed at the default location, you don't need to specify `-t/--fmdevtool_path`:

```bash
fm-size -i /path/to/database.fmp12
```

Or with a custom FMDeveloperTool path:

```bash
fm-size -t /path/to/FMDeveloperTool -i /path/to/database.fmp12
```

With config file, specifying 3 concurrent threads:

```bash
fm-size -c path/to/config.json -m 3
```

### Command-Line Options

#### Required (unless using config file)

- `-i, --db_file_paths <PATHS>...`: Paths to search for `.fmp12` files. Can be directories or individual `.fmp12` files

#### Optional

- `-t, --fmdevtool_path <PATH>`: Path to FMDeveloperTool executable (optional - will use default location if not specified)
  - Windows: `C:\Program Files\FileMaker\FileMaker Server\Database Server\FMDeveloperTool.exe`
  - macOS: `/Library/FileMaker Server/Database Server/bin/FMDeveloperTool`
  - Linux: `/opt/FileMaker/FileMaker Server/Database Server/bin/FMDeveloperTool`

#### Authentication (optional)

When not using `--config_path`, you will be prompted for authentication credentials:
- **Account name**: Prompted with visible input (press Enter to skip if not needed)
- **Password**: Prompted with concealed input (press Enter to skip if not needed)
- **Encryption key**: Prompted with concealed input for EAR (encryption at rest) databases (press Enter to skip if not needed)

This prevents secrets from appearing in command history or process lists.

**Note**: If using `--config_path`, account_name, password, and encryption_key should be specified in the config file instead.

#### Processing Options

- `-f, --db_filter <FILTER>`: Comma-separated list of `.fmp12` files to process. Other files found will be skipped
- `-r, --recurse`: Recursively search subdirectories for `.fmp12` files
- `-m, --max_concurrent <NUMBER>`: Maximum number of database files to process in parallel (default: 1)

#### Output Options

- `-o, --output_file_path <PATH>`: Directory for output CSV files (defaults to first path from `--db_file_paths`)

#### Configuration File

- `-c, --config_path <PATH>`: Path to JSON config file. If provided, overrides: `fmdevtool_path` (optional in config), `db_file_paths`, `account_name`, `password`, `ear_key`, `db_filter`. When using a config file, authentication credentials are read from the file and no prompts are shown.

### Configuration File Format

When using `-c/--config_path`, provide a JSON file with the following structure:

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

### Examples

#### Process a single database

```bash
fm-size -t /path/to/FMDeveloperTool \
        -i /path/to/MyDatabase.fmp12
# You will be prompted for account name, password, and encryption key
```

#### Process multiple databases in a directory using up to 4 concurrent threads

```bash
fm-size -t /path/to/FMDeveloperTool \
        -i /path/to/databases/ \
        -m 4 \
        -r
# You will be prompted for account name, password, and encryption key
```

#### Process specific databases with filtering

```bash
fm-size -t /path/to/FMDeveloperTool \
        -i /path/to/databases/ \
        -f "Database1,Database2"
# You will be prompted for account name, password, and encryption key
```

#### Use a configuration file

```bash
fm-size -c config.json -o /path/to/output/
```

## Output

The tool generates three CSV files in the output directory:

1. **fm-size-dbs.csv**: Overall database file sizes with processing times
2. **fm-size-fields.csv**: Size breakdown by field for each database
3. **fm-size-durations.csv**: Detailed timing information for each command executed

All sizes are reported in bytes. The CSV files always include `fmsize_ver` and `fmdevtool_ver` columns showing the versions used.
