/*
 * rust-stow
 * Copyright (C) 2026 Chris Tisdale
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use crate::{CliArgs, CliError};
use clap::builder::Styles;
use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, ValueHint};
use rstow_commands::{CommandBuilder, CommandOperationImpl};
use rstow_config::{AppConfiguration, DEFAULT_CONFIG_FILE, path_resolver};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tracing::level_filters::LevelFilter;

const STYLES: Styles = Styles::styled();
const APP_NAME: &str = "rstow";

#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq, Hash)]
enum ProcessCommands {
    Stow,
    Delete,
    Restow,
}

impl Display for ProcessCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stow => f.write_str("Stow"),
            Self::Delete => f.write_str("Delete"),
            Self::Restow => f.write_str("Restow"),
        }
    }
}

#[derive(Args)]
struct ProcessCommandArgs {
    #[arg(
        short = 's',
        long = "stow",
        help = "Stow packages into the target directory, creating symbolic links for each file. This is the default operation if no command is specified.",
        conflicts_with = "delete",
        conflicts_with = "restow"
    )]
    stow: bool,
    #[arg(
        short = 'D',
        long = "delete",
        alias = "unstow",
        visible_alias = "unstow",
        help = "Remove symbolic links from the target directory that belong to the specified packages. This is useful for cleaning up after a package is no longer needed.",
        conflicts_with = "stow",
        conflicts_with = "restow"
    )]
    delete: bool,
    #[arg(
        short = 'R',
        long = "restow",
        help = "Restow packages by first removing their existing symbolic links and then re-stowing them. This is equivalent to running 'delete' followed by 'stow', and is useful for updating links after package contents change.",
        conflicts_with = "stow",
        conflicts_with = "delete"
    )]
    restow: bool,
}

impl From<ProcessCommandArgs> for ProcessCommands {
    fn from(args: ProcessCommandArgs) -> Self {
        if args.delete {
            Self::Delete
        } else if args.restow {
            Self::Restow
        } else {
            Self::Stow
        }
    }
}

#[derive(Args)]
struct GlobalArgs {
    #[arg(
        short = 'n',
        long = "simulate",
        alias = "no",
        visible_alias = "no",
        global = true,
        help = "Perform a dry run of the operation. This will display the actions that would be taken without making any actual changes to the filesystem."
    )]
    simulate: bool,
    #[arg(
        short = 'd',
        long = "directory",
        alias = "dir",
        global = true,
        visible_alias = "dir",
        help = "Specify the source directory (stow directory) containing the packages to be managed. If not provided, it defaults to the current working directory.",
        value_name = "DIRECTORY",
        value_hint = ValueHint::DirPath
    )]
    directory: Option<PathBuf>,
    #[arg(
        short = 't',
        long = "target",
        global = true,
        help = "Specify the target directory where symbolic links will be created. By default, this is the parent of the source directory.",
        value_name = "DIRECTORY",
        value_hint = ValueHint::DirPath
    )]
    target: Option<PathBuf>,
    #[arg(
        short = 'l',
        long = "log-level",
        global = true,
        help = "Set the application logging level. Supported levels are: Trace, Debug, Info, Warn, Error, or Off. This is primarily used for troubleshooting and debugging.",
        value_name = "LEVEL"
    )]
    log_level: Option<LevelFilter>,
    #[arg(
        short = 'c',
        long = "config",
        global = true,
        help = "Path to a custom configuration file. If not specified, rstow looks for a '.stow.toml' file in the current working directory.",
        value_name = "FILE",
        value_hint = ValueHint::FilePath
    )]
    config_file: Option<PathBuf>,
    #[arg(
        long = "no-folding",
        global = true,
        help = "Disable directory folding during stowing and refolding during deletion. Folding is a technique where a single symbolic link to a directory is used instead of individual links for each file within that directory."
    )]
    no_folding: bool,
    #[arg(
        short = 'i',
        long = "ignore",
        global = true,
        help = "Specify a file path or a regular expression pattern to exclude specific files or directories from being processed.",
        value_name = "PATTERN"
    )]
    ignored: Vec<String>,
    #[arg(
        short = 'o',
        long = "override",
        global = true,
        help = "Specify a file path or a regular expression pattern for files or directories that should be forcefully stowed, even if they would otherwise be ignored or causing conflicts.",
        value_name = "PATTERN"
    )]
    overrides: Vec<String>,
    #[arg(
        long = "dotfiles",
        global = true,
        help = "Enable special handling for dotfiles by automatically renaming files with a specific prefix. For example, using the default 'dot-' prefix, a file named 'dot-bashrc' will be stowed as '.bashrc' in the target directory.",
        value_name = "PREFIX",
        default_missing_value = "dot-",
        num_args = 0..=1
    )]
    dotfiles: Option<String>,
}

#[derive(Parser)]
#[command(version, name = APP_NAME, about, author, propagate_version = true, styles = STYLES, help_template = "\
{before-help}{name} {version}: {author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
#[clap(rename_all = "snake_case")]
pub struct CommandLineProcessor {
    #[clap(flatten)]
    process_command: ProcessCommandArgs,
    #[clap(flatten)]
    global_args: GlobalArgs,
}

impl CommandLineProcessor {
    /// Parses and processes command-line arguments to configure the CLI application.
    ///
    /// This function performs the following steps:
    /// 1. Parses the command-line arguments into a `CliArgs` structure using `try_parse()`.
    /// 2. Resolves the `directory` argument, which is mandatory, and throws an error if it is missing.
    /// 3. Determines the configuration file path from the provided `config_file` argument, or defaults to `DEFAULT_CONFIG_FILE` under the resolved directory.
    /// 4. Checks if the configuration file exists and sets it up accordingly. If it doesn't exist, an optional `None` is used.
    /// 5. Loads the application configuration using `AppConfiguration::load_configuration`, which also incorporates ignored items.
    /// 6. Sets up the logger using the logging level specified by the global arguments.
    /// 7. Resolves the `target` argument, falling back to a default target if none is provided.
    /// 8. Configures the command builder with the parsed `directory` and `target`.
    ///     - If the `simulate` mode is enabled in the global arguments, simulation behavior is applied to the builder.
    ///     - Otherwise, the real command execution mode is applied.
    /// 9. Constructs the specific command to execute based on the parsed subcommand:
    ///     - `ProcessCommands::Stow`: Constructs a "stow" command with ignored items and folding settings.
    ///     - `ProcessCommands::Delete`: Constructs an "unstow" command.
    ///     - `ProcessCommands::Restow`: Constructs a "restow" command with ignored items and folding settings.
    /// 10. Wraps the constructed command along with the logger guard into a `CliArgs` structure and returns it.
    ///
    /// # Errors
    /// - Returns a `CliError` if any mandatory argument is missing or if there are issues resolving paths, loading configuration, or building commands.
    ///
    /// # Returns
    /// On successful processing of the CLI arguments, a `Result<CliArgs<CommandOperationImpl>, CliError>` structure is returned, containing:
    /// - The prepared command to execute.
    /// - A guard for resource cleanup (e.g., the logger setup).
    ///
    /// # Example
    /// ```
    /// use rstow_args::{CliArgs, CommandLineProcessor};
    ///
    /// match CommandLineProcessor::get_cli_args() {
    ///     Ok(cli_args) => {
    ///         println!("Command setup successfully: {}", cli_args.command);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to parse command-line arguments: {e}");
    ///     }
    /// }
    /// ```
    pub fn get_cli_args() -> Result<CliArgs<CommandOperationImpl>, CliError> {
        let cli_args = Self::try_parse()?;
        let global = cli_args.global_args;
        let process_command = cli_args.process_command.into();
        let directory = global.directory.map_or_else(
            || Err(Self::command().error(ErrorKind::MissingRequiredArgument, "DIRECTORY is required."))?,
            |d| path_resolver::resolve_path(&d).map_err(CliError::from),
        )?;

        let config_file = match global.config_file {
            Some(c) => Self::resolve_config_file(&c)?,
            None => directory.join(DEFAULT_CONFIG_FILE),
        };

        let ignored = global.ignored.into_iter().collect();
        let config_file = if fs::exists(&config_file).unwrap_or(false) {
            Some(&config_file)
        } else {
            None
        };

        let app_config = AppConfiguration::load_configuration(config_file.map(PathBuf::as_path), &directory, ignored)?;
        let guard = app_config.setup_logger(global.log_level)?;

        let target = global
            .target
            .as_deref()
            .map_or_else(Self::get_default_target, |p| {
                path_resolver::resolve_path(p).map_err(CliError::from)
            })?;

        let mut builder = CommandBuilder::new()
            .with_directory(directory)
            .with_target(target);

        builder = if global.simulate {
            builder.simulate()
        } else {
            builder.command()
        };

        let overrides = global.overrides.into_iter().collect();
        let command = match process_command {
            ProcessCommands::Stow => builder
                .stow()
                .with_ignored(app_config.ignored)
                .with_no_folding(global.no_folding)
                .with_dot_file_prefix(global.dotfiles)
                .with_overrides(overrides)
                .build(),
            ProcessCommands::Delete => builder.unstow().build(),
            ProcessCommands::Restow => builder
                .restow()
                .with_ignored(app_config.ignored)
                .with_no_folding(global.no_folding)
                .with_dot_file_prefix(global.dotfiles)
                .with_overrides(overrides)
                .build(),
        }?;

        Ok(CliArgs {
            command,
            _guard: guard,
        })
    }

    fn resolve_config_file(path: &Path) -> Result<PathBuf, CliError> {
        fs::metadata(path)
            .map(|m| m.is_file())
            .map_err(|_| CliError::InvalidConfigurationFile(path.display().to_string()))
            .and_then(|is_file| {
                if is_file {
                    Ok(path.to_path_buf())
                } else {
                    Err(CliError::InvalidConfigurationFile(
                        path.display().to_string(),
                    ))
                }
            })
    }

    fn get_default_target() -> Result<PathBuf, CliError> {
        let current_dir = env::current_dir()?;
        current_dir.parent().map_or_else(
            || Err(CliError::InvalidTargetDirectory),
            |p| Ok(p.to_path_buf()),
        )
    }
}
