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

use crate::{
    AppDirectories, ConfigError, ConfigFileVersion, DEFAULT_CONFIG_FILE, Ignored, LoggingConfig, path_resolver,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::Path;
use std::{env, fs};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub version: ConfigFileVersion,
    #[serde(default)]
    pub ignored: Ignored,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {{ version: {}, ignored: {}, logging: {} }}",
            self.version, self.ignored, self.logging
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: ConfigFileVersion::V1,
            ignored: Ignored::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    pub fn from_file(file_path: Option<&Path>) -> Result<Self, ConfigError> {
        let app_directories = AppDirectories::load_directories();
        if let Some(file_path) = file_path {
            return Self::read_config_file(file_path, app_directories);
        }

        let config_file = app_directories.config_dir.join(DEFAULT_CONFIG_FILE);
        if fs::exists(&config_file).unwrap_or(false) {
            return Self::read_config_file(&config_file, app_directories);
        }

        Ok(Self::default())
    }

    fn read_config_file(file_path: &Path, app_directories: AppDirectories) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(file_path)?;
        let mut config = toml::from_str::<Self>(&content)?;
        let home_dir = env::home_dir().ok_or(ConfigError::UnableToFindHomeDirectory)?;
        config.ignored.file = path_resolver::resolve_path(&config.ignored.file)?;
        if config.ignored.file.is_relative() {
            let parent_dir = file_path.parent().unwrap_or(home_dir.as_path());
            config.ignored.file = parent_dir.join(config.ignored.file);
        }

        if let Some(file) = config.ignored.file.to_str() {
            let path = Path::new(file);
            config.ignored.file = path_resolver::resolve_path(path)?;
        }

        config.logging.logging_path = if let Some(path) = config.logging.logging_path {
            Some(path_resolver::resolve_path(&path)?)
        } else {
            Some(app_directories.log_dir)
        };

        if let Some(path) = &config.logging.logging_path
            && path.is_relative()
        {
            let parent_dir = file_path.parent().unwrap_or(home_dir.as_path());
            config.logging.logging_path = Some(parent_dir.join(path));
        }

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logging_config::{LoggingLevel, RotationType};
    use std::path::PathBuf;

    #[test]
    fn toml_version_1_deserialization() {
        let config_content = r#"
        version = 1

        [ignored]
        file = "ignored_files.txt"
        comment = 'c'

        [logging]
        level = "info"
        file = "temp.log"
        logging_path = "log_dir"
        rotation = "daily"
        max_log_files = 10
        color_support = true
        "#;

        let config: Config = toml::from_str(config_content).expect("Failed to parse TOML");
        assert_eq!(config.ignored.file, Path::new("ignored_files.txt"));
        assert_eq!(config.ignored.comment, 'c');
        assert_eq!(config.version, ConfigFileVersion::V1);
        assert_eq!(config.logging.file, Some(PathBuf::from("temp.log")));
        assert_eq!(config.logging.logging_path, Some(PathBuf::from("log_dir")));
        assert_eq!(config.logging.rotation, Some(RotationType::Daily));
        assert_eq!(config.logging.max_log_files, Some(10));
        assert_eq!(config.logging.color_support, Some(true));
        assert_eq!(config.logging.level, Some(LoggingLevel::Info));
    }

    #[test]
    fn toml_version_1_deserialization_string_version() {
        let allowed_versions = vec!["1", "v1", "V1"];
        for version in allowed_versions {
            let config_content = format!(
                r#"
            version = "{version}"
            "#
            );

            let config: Config = toml::from_str(&config_content).expect("Failed to parse TOML");
            assert_eq!(config.version, ConfigFileVersion::V1);
        }
    }

    #[test]
    fn toml_version_1_everything_is_optional() {
        let config_content = "
        version = 1
        ";

        let default_config = Config::default();
        let config: Config = toml::from_str(config_content).expect("Failed to parse TOML");
        assert_eq!(config.version, ConfigFileVersion::V1);
        assert_eq!(
            config.logging.logging_path,
            default_config.logging.logging_path
        );
        assert_eq!(config.logging.rotation, default_config.logging.rotation);
        assert_eq!(
            config.logging.max_log_files,
            default_config.logging.max_log_files
        );
        assert_eq!(
            config.logging.color_support,
            default_config.logging.color_support
        );
        assert_eq!(config.logging.level, default_config.logging.level);
        assert_eq!(config.ignored.file, default_config.ignored.file);
        assert_eq!(config.ignored.comment, default_config.ignored.comment);
    }

    #[test]
    fn toml_version_1_ignores_logging_level_case() {
        let allowed_levels = vec![
            LoggingLevel::Off,
            LoggingLevel::Trace,
            LoggingLevel::Debug,
            LoggingLevel::Info,
            LoggingLevel::Warn,
            LoggingLevel::Error,
        ];

        for level in allowed_levels {
            let config_content = format!(
                r#"
            version = 1

            [logging]
            level = "{level}"
            "#
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.level, Some(level));

            let config_content = format!(
                r#"
            version = 1

            [logging]
            level = "{}"
            "#,
                level.to_string().to_uppercase()
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.level, Some(level));

            let config_content = format!(
                r#"
            version = 1

            [logging]
            level = "{}"
            "#,
                level.to_string().to_lowercase()
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.level, Some(level));
        }
    }

    #[test]
    fn toml_version_1_logging_level_can_use_numeric_value() {
        let allowed_levels = vec![
            LoggingLevel::Off,
            LoggingLevel::Trace,
            LoggingLevel::Debug,
            LoggingLevel::Info,
            LoggingLevel::Warn,
            LoggingLevel::Error,
        ];

        for level in allowed_levels {
            let config_content = format!(
                r"
            version = 1

            [logging]
            level = {}
            ",
                level as i64
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.level, Some(level));
        }
    }

    #[test]
    fn toml_version_1_ignores_rotation_case() {
        let rotations_types = vec![RotationType::Hourly, RotationType::Daily];
        for rotation in rotations_types {
            let config_content = format!(
                r#"
            version = 1

            [logging]
            rotation = "{rotation}"
            "#
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.rotation, Some(rotation));

            let config_content = format!(
                r#"
            version = 1

            [logging]
            rotation = "{}"
            "#,
                rotation.to_string().to_uppercase()
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.rotation, Some(rotation));

            let config_content = format!(
                r#"
            version = 1

            [logging]
            rotation = "{}"
            "#,
                rotation.to_string().to_lowercase()
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.rotation, Some(rotation));
        }
    }

    #[test]
    fn toml_version_1_rotation_can_use_numeric_value() {
        let rotations_types = vec![RotationType::Hourly, RotationType::Daily];
        for rotation in rotations_types {
            let config_content = format!(
                r"
            version = 1

            [logging]
            rotation = {}
            ",
                rotation as i64
            );

            let config: Config = toml::from_str(config_content.as_str()).expect("Failed to parse TOML");
            assert_eq!(config.logging.rotation, Some(rotation));
        }
    }
}
