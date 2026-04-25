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

use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CliError {
    #[error(transparent)]
    LoggingError(#[from] rstow_config::LoggingError),
    #[error(transparent)]
    InvalidPath(#[from] std::io::Error),
    #[error(transparent)]
    CommandLineParsingError(#[from] clap::Error),
    #[error(transparent)]
    MatchingError(#[from] clap::parser::MatchesError),
    #[error(transparent)]
    InvalidConfigFile(#[from] rstow_config::ConfigError),
    #[error(transparent)]
    CommandError(#[from] rstow_commands::CommandError),
    #[error("Invalid target directory.  The target directory must exist and be a directory.")]
    InvalidTargetDirectory,
    #[error(transparent)]
    StripPrefixError(#[from] std::path::StripPrefixError),
    #[error("Invalid configuration file: {0}")]
    InvalidConfigurationFile(String),
    #[error(transparent)]
    CommandBuildError(#[from] rstow_commands::CommandBuildError),
}
