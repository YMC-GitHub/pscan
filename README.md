# PScan - Process Scanning and Analysis Tool

## Project Description

PScan is a cross-platform command-line tool for process filtering and analysis with advanced window detection capabilities. It provides multiple output formats and flexible filtering options to help developers and system administrators quickly locate and analyze system processes.

## Project Value and Significance

### 🎯 Core Problems Solved
- **Quick Process Location**: Rapidly find target processes among numerous system processes
- **Window Association Analysis**: Identify which processes have graphical interface windows
- **Multi-format Output**: Support for table, JSON, YAML, CSV, and other formats for easy integration with other tools
- **Cross-platform Compatibility**: Runs on both Windows and Unix-like systems

### 💡 Application Scenarios
- Finding specific processes when debugging applications
- System resource monitoring and analysis
- Automation script integration
- Process management tool development

## Features

### 🔍 Filtering Capabilities
- **PID Filter**: Exact filtering by process ID
- **Name Filter**: Fuzzy matching by process name (case-insensitive)
- **Title Filter**: Fuzzy matching by window title (case-insensitive)
- **Window Status Filter**: Filter processes with or without windows

### 📊 Output Formats
- **Table Format**: Easy-to-read tabular output
- **JSON Format**: Easy parsing by programs
- **YAML Format**: Well-structured configuration format
- **CSV Format**: Spreadsheet-compatible format
- **Simple Format**: Concise single-line output
- **Detailed Format**: Complete process information display

### 🖥️ Platform Support
- **Windows**: Full window detection support
- **Linux/macOS**: Basic process information support (window detection as placeholder)

## Directory Structure

```
pscan/
├── Cargo.toml          # Project configuration and dependency management
├── Cargo.lock          # Dependency version locking
├── README.md           # Project documentation (Chinese)
├── README.en.md        # Project documentation (English)
└── src/
    ├── main.rs         # Main program entry
    └── output.rs       # Output format processing module
```

## Installation Guide

### Prerequisites
- Rust programming environment (version 1.70.0 or higher)
- Cargo package manager

### Installation Steps

1. **Install from Source**
```bash
# Clone the project
git clone https://github.com/ymc-github/pscan.git
cd pscan

# Build and install
cargo install --path .
```

2. **Install from Crates.io** (after publication)
```bash
cargo install pscan
```

3. **Run Locally**
```bash
# Run in development mode
cargo run -- --help

# Run in release mode
cargo run --release -- --help
```

## Usage Examples

### Basic Usage

```bash
# View all processes (table format)
pscan

# View detailed process information
pscan --verbose

# Filter by process name
pscan --name "chrome"

# Filter by window title
pscan --title "Visual Studio"

# Show only processes with windows
pscan --has-window
```

### Filtering Examples

```bash
# Exact search by PID
pscan --pid 1234

# Combined filtering: processes with name containing "code" and having windows
pscan --name "code" --has-window

# Show processes without windows
pscan --no-window
```

### Output Format Examples

```bash
# JSON format output
pscan --format json

# YAML format output
pscan --format yaml

# CSV format output (suitable for Excel import)
pscan --format csv

# Simple format output
pscan --format simple

# Detailed format output
pscan --format detailed
```

### Advanced Usage

```bash
# Combine filtering and output formats
pscan --name "firefox" --has-window --format json

# Table output in verbose mode
pscan --verbose --format table

# Pipeline processing: find Chrome processes and count them
pscan --name "chrome" --format simple | wc -l
```

## Output Description

### Table Columns
- **PID**: Process ID
- **Name**: Process name
- **Title**: Window title or fallback title
- **Memory**: Memory usage (MB)
- **Window**: Whether has window (shown only in verbose mode)

### Memory Units
- Memory usage automatically converted to MB
- Raw memory usage in bytes (shown in detailed output)

## Development Guide

### Project Architecture
- **main.rs**: Handles command-line parsing, process collection, and filtering logic
- **output.rs**: Handles all output format rendering and display

### Extension Development
To add new output formats, add corresponding handling in the `OutputFormat` enum and `display_processes` function in `output.rs`.

## Troubleshooting

### Common Issues

1. **Window Detection Not Working (Non-Windows Systems)**
   - Currently, window detection is fully supported only on Windows systems
   - Other systems will display warning messages

2. **Permission Issues**
   - Administrator privileges may be required on some systems to access all process information

3. **Incomplete Process Information**
   - Some processes may not provide complete command-line or path information

## Author Information

**Developer**: YeMiancheng  
**Email**: ymc.github@gmail.com  
**GitHub**: [ymc-github](https://github.com/ymc-github)

## License Information

This project is dual-licensed under:
- MIT License
- Apache License 2.0

You may choose either license according to your needs.

## Contributing

Issues and Pull Requests are welcome to help improve this project!

## Changelog

### v0.1.0
- Initial version release
- Basic process filtering functionality
- Multi-format output support
- Windows window detection feature