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

mod app_configuration;
mod app_directories;
mod config;
mod config_error;
mod config_file_version;
mod ignored;
mod logging_config;
mod logging_error;

mod level_error;
mod overrides;
pub mod path_resolver;
mod rotation_error;
mod version_error;

pub use app_configuration::{AppConfiguration, DEFAULT_CONFIG_FILE};
pub use config_error::ConfigError;
pub use config_file_version::ConfigFileVersion;
pub use level_error::LevelError;
pub use logging_error::LoggingError;
pub use rotation_error::RotationError;
pub use version_error::VersionError;

pub(crate) use app_directories::AppDirectories;
pub(crate) use config::Config;
pub(crate) use ignored::Ignored;
pub(crate) use logging_config::LoggingConfig;
pub(crate) use overrides::Overrides;
