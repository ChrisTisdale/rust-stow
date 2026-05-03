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

use crate::{Config, ConfigError, LoggingError};
use std::collections::HashSet;
use std::fmt::Display;
use std::io::stderr;
use std::path::{Path, PathBuf};
use std::{env, fs};
use supports_color::Stream;
use tracing::subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::fmt::format::{Compact, DefaultFields, Format};

pub const DEFAULT_CONFIG_FILE: &str = ".rstow.toml";

const DEFAULT_LOG_FILE: &str = "rstow.log";

const DEFAULT_IGNORE: &[&str] = &[
    "RCS",
    ".+,v",
    "CVS",
    r"\.\#.+",
    r"\.cvsignore",
    r"\.svn",
    "_darcs",
    r"\.hg",
    r"\.git",
    r"\.gitignore",
    r"\.gitmodules",
    r"\.jj",
    ".+~",
    r"\#.*\#",
    "^/README.*",
    "^/LICENSE.*",
    "^/COPYING",
    "^/.DS_Store",
];

pub struct AppConfiguration {
    config: Config,
    pub ignored: HashSet<String>,
    pub overrides: HashSet<String>,
}

impl Display for AppConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AppConfiguration {{ config: {}, ignored: {:?} }}",
            self.config, self.ignored
        )
    }
}

impl AppConfiguration {
    /// Load the configuration from the provided configuration file
    ///
    /// # Arguments
    ///
    /// * `config_file`: The path to the configuration file
    /// * `ignored`: The set of ignored patterns
    ///
    /// returns: Result<`AppConfiguration`, `ConfigError`>
    ///
    /// # Errors
    /// * `ConfigError::ConfigError` - Returned when the configuration file cannot be read
    /// * `ConfigError::TomlError` - Returned when the configuration file is not a valid Toml File
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::error::Error;
    /// use rstow_config::{AppConfiguration, ConfigError};
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     use std::env;
    /// let configuration = AppConfiguration::load_configuration(None, &env::current_dir()?, HashSet::new(), HashSet::new())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn load_configuration(
        config_file: Option<&Path>,
        search_path: &Path,
        mut ignored: HashSet<String>,
        mut overrides: HashSet<String>,
    ) -> Result<Self, ConfigError> {
        let mut config = Config::from_file(config_file)?;
        if config.ignored.file.is_relative() {
            config.ignored.file = search_path.join(config.ignored.file);
        }

        if let Some(logging_path) = &config.logging.logging_path
            && logging_path.is_relative()
        {
            config.logging.logging_path = Some(search_path.join(logging_path));
        }

        ignored.extend(Self::read_ignore_file(&config, config_file)?);
        overrides.extend(Self::read_override_file(&config)?);
        Ok(Self {
            config,
            ignored,
            overrides,
        })
    }

    /// Setting up logging for the application using the provided configuration
    ///
    /// # Arguments
    ///
    /// * `override_level`: The level to override the configuration level with
    ///
    /// returns: Result<Option<WorkerGuard>, `LoggingError`>
    /// The guard for the log file, if any, is returned
    ///
    /// # Errors
    /// * `LoggingError::LoggingError` - Returned when the logger cannot be set up
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::error::Error;
    /// use rstow_config::{AppConfiguration, LoggingError};
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     use std::env;
    /// let configuration = AppConfiguration::load_configuration(None, &env::current_dir()?, HashSet::new(), HashSet::new())?;
    ///     configuration.setup_logger(None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn setup_logger(&self, override_level: Option<LevelFilter>) -> Result<Option<WorkerGuard>, LoggingError> {
        let config_level = override_level.unwrap_or_else(|| {
            self.config
                .logging
                .level
                .map_or(LevelFilter::WARN, Into::into)
        });

        if config_level == LevelFilter::OFF {
            return Ok(None);
        }

        self.config
            .logging
            .file
            .as_ref()
            .and_then(|d| self.get_rolling_appender(d))
            .map(tracing_appender::non_blocking)
            .map_or_else(
                || {
                    subscriber::set_global_default(
                        self.get_default_trace_builder(config_level)
                            .with_writer(stderr)
                            .finish(),
                    )?;
                    Ok(None)
                },
                |(appender, guard)| {
                    let subscriber = self
                        .get_default_trace_builder(config_level)
                        .with_ansi(false)
                        .with_writer(appender)
                        .finish();
                    subscriber::set_global_default(subscriber)?;
                    Ok(Some(guard))
                },
            )
    }

    fn build_file_pattern(path: Option<&Path>) -> Option<String> {
        path.and_then(|p| p.file_name())
            .and_then(|p| p.to_str())
            .map(|p| format!("^/{p}"))
    }

    fn read_ignore_file(config: &Config, config_file: Option<&Path>) -> Result<HashSet<String>, ConfigError> {
        let mut files = HashSet::new();
        if let Some(file_string) = Self::build_file_pattern(Some(config.ignored.file.as_path())) {
            files.insert(file_string);
        }

        if let Some(file_string) = Self::build_file_pattern(config_file) {
            files.insert(file_string);
        }

        if !fs::exists(config.ignored.file.as_path()).unwrap_or(false) {
            files.extend(DEFAULT_IGNORE.iter().map(ToString::to_string));
            return Ok(files);
        }

        Self::read_ignore_or_override_file(config.ignored.file.as_path(), config.ignored.comment, files)
    }

    fn read_override_file(config: &Config) -> Result<HashSet<String>, ConfigError> {
        let files = HashSet::new();
        if !fs::exists(config.overrides.file.as_path()).unwrap_or(false) {
            return Ok(files);
        }

        Self::read_ignore_or_override_file(
            config.overrides.file.as_path(),
            config.overrides.comment,
            files,
        )
    }

    fn read_ignore_or_override_file(
        file: &Path,
        comment: char,
        mut files: HashSet<String>,
    ) -> Result<HashSet<String>, ConfigError> {
        let content = fs::read_to_string(file)?;
        let items = content
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with(comment))
            .map(|line| Self::parse_line(comment, line))
            .filter(|line| !line.is_empty())
            .map(ToString::to_string);

        files.extend(items);
        Ok(files)
    }

    fn parse_line(comment: char, line: &str) -> &str {
        let mut has_escaped_backslash = false;
        for (i, c) in line.char_indices() {
            if !has_escaped_backslash && c == comment {
                return line[..i].trim();
            }

            has_escaped_backslash = c == '\\' && !has_escaped_backslash;
        }

        line.trim()
    }

    fn get_log_path(root: &Path, dir: &Path) -> PathBuf {
        if dir.is_absolute() {
            dir.to_owned()
        } else {
            root.join(dir)
        }
    }

    fn create_directory_if_necessary(dir: &Path) -> Result<(), std::io::Error> {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        Ok(())
    }

    fn get_rolling_appender(&self, path: &Path) -> Option<RollingFileAppender> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(DEFAULT_LOG_FILE)
            .to_string();

        let current = env::current_dir();
        match current {
            Ok(current) => {
                let root = self.config.logging.logging_path.as_ref().map(|p| {
                    if p.is_absolute() {
                        p.to_owned()
                    } else {
                        current.join(p)
                    }
                });

                root.and_then(|dir| {
                    path.parent()
                        .map(|d| Self::get_log_path(&dir, d))
                        .and_then(|d| Self::create_directory_if_necessary(&d).ok().map(|()| d))
                        .and_then(|n| n.to_str().map(ToString::to_string))
                        .or_else(|| dir.to_str().map(ToString::to_string))
                })
                .map_or_else(|| None, |root| self.map_file_appender(file_name, root))
            }
            Err(..) => None,
        }
    }

    fn map_file_appender(&self, file_name: String, root: String) -> Option<RollingFileAppender> {
        RollingFileAppender::builder()
            .rotation(
                self.config
                    .logging
                    .rotation
                    .map_or(Rotation::DAILY, Into::into),
            )
            .max_log_files(self.config.logging.max_log_files.unwrap_or(5))
            .filename_prefix(file_name)
            .build(root)
            .ok()
    }

    fn get_default_trace_builder(&self, log_level: LevelFilter) -> SubscriberBuilder<DefaultFields, Format<Compact>> {
        tracing_subscriber::fmt()
            .compact()
            .with_level(true)
            .with_max_level(log_level)
            .with_file(true)
            .log_internal_errors(true)
            .with_line_number(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(true)
            .with_ansi(
                self.config.logging.color_support.unwrap_or(true) && supports_color::on(Stream::Stderr).is_some(),
            )
    }
}
