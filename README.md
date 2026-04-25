# Rust Stow

A Rust implementation of the GNU stow utility for managing dotfiles.

## Overview

`rust-stow` (or `rstow`) is a symlink farm manager which takes separate packages of software and/or data located in
separate directories on the filesystem and makes them appear to be installed in the same place. It is primarily used for
managing dotfiles, allowing you to keep your configuration files in a central repository while symlinking them to their
expected locations (e.g., in your home directory).

### Features

- **Stow**: Create symlinks for a package.
- **Delete**: Remove symlinks for a package.
- **Restow**: Refresh symlinks (delete followed by stow).
- **Simulation Mode**: See what would happen without making any changes.
- **Folding/Unfolding**: Supports directory folding similar to GNU Stow (can be disabled).
- **Ignore/Override**: Regex-based ignore and override patterns.
- **Customizable**: Configurable via TOML and ignore files.

## Requirements

- **Rust**: Stable (2024 edition)
- **Cargo**: Package manager for Rust

## Setup and Installation

### From Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ChrisTisdale/rust-stow.git
   cd rust-stow
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. The binary will be available at `target/release/rstow`.

### Running with Cargo

You can also run the application directly using Cargo:

```bash
cargo run --package rstow -- [OPTIONS] [COMMAND]
```

## Usage

```bash
rstow [OPTIONS] [COMMAND]
```

### Commands

- `stow <PACKAGE>`: Stow the specified package.
- `delete <PACKAGE>`: Unstow the specified package.
- `restow <PACKAGE>`: Restow the specified package.

### Global Options

- `-n`, `--simulate`, `--no`: Do not perform any actions. Show what would happen.
- `-d`, `--directory`, `--dir <DIRECTORY>`: The directory to operate on. Defaults to the current working directory.
- `-t`, `--target <DIRECTORY>`: The target directory to operate on. Defaults to the parent directory of the stow
  directory.
- `-l`, `--log-level <LEVEL>`: Log level (`Trace`, `Debug`, `Info`, `Warn`, `Error`, `Off`).
- `-c`, `--config <FILE>`: Path to the configuration TOML file. Defaults to `.rstow.toml`.
- `--no-folding`: Disable folding of newly stowed directories.
- `-i`, `--ignore <REGEX>`: File or directory to ignore (supports regex).
- `-o`, `--override <REGEX>`: File or directory to override (supports regex).
- `--dotfiles`: Enable special handling for dotfiles. Replaces the 'dot-' prefix with a period (.). For example, '
  dot-bashrc' becomes '.bashrc'.

## Configuration

### `.rstow.toml`

`rstow` can be configured using a `.rstow.toml` file. It looks for this file in the current working directory or in the
user's configuration directory (`$XDG_CONFIG_HOME/rstow/.rstow.toml`).

Example `.rstow.toml`:

```toml
version = 1

[ignored]
file = ".rstow-ignore"
comment = "#"

[logging]
level = "Info"
# logging_path = "path/to/logs"
# rotation = "Daily" # "Daily" or "Hourly"
# max_log_files = 30
# color_support = true
```

### Ignore Files

By default, `rstow` looks for a `.rstow-ignore` file to determine which files should be skipped during stowing. This
file supports regex patterns and comments starting with `#`.

## Scripts

The project uses `xtask` for custom automation.

- **Build and Distribute**:
  ```bash
  cargo run --package xtask -- dist
  ```
  This command builds the application in release mode and generates man pages in `target/dist`.

## Environment Variables

- `XDG_CONFIG_HOME`: Used to locate configuration files (e.g., `$XDG_CONFIG_HOME/rstow/.rstow.toml`).
- `XDG_DATA_HOME`: Used for application data and logs.
- `RSTOW_LOG`: Can be used to override the log level (standard `tracing` / `log` environment variable support might be
  available, TODO: verify implementation).

## Tests

Run the test suite using Cargo:

```bash
cargo test
```

## Project Structure

This repository is organized as a Rust workspace:

- `src/rstow`: The main CLI application entry point.
- `src/rstow-args`: Command-line argument parsing and CLI logic (using `clap`).
- `src/rstow-commands`: Core logic for stow, unstow, and restow operations.
- `src/rstow-config`: Configuration loading and management (using `serde` and `toml`).
- `xtask`: Custom build and distribution tasks (e.g., man page generation).

## License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public
License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later
version.

See [LICENSE.md](LICENSE.md) for details.
