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

use std::env;
use std::path::PathBuf;

const APP_NAME: &str = "rstow";

#[cfg(target_os = "macos")]
const DEFAULT_CONFIG_PATH: &str = "~/Library/Application Support/";

#[cfg(target_os = "linux")]
const DEFAULT_CONFIG_PATH: &str = "~/.config/";

#[cfg(target_os = "windows")]
const DEFAULT_CONFIG_PATH: &str = "~\\AppData\\Roaming";

#[cfg(target_os = "macos")]
const DEFAULT_LOG_PATH: &str = "~/Library/Application Support/";

#[cfg(target_os = "linux")]
const DEFAULT_LOG_PATH: &str = "~/.local/share/";

#[cfg(target_os = "windows")]
const DEFAULT_LOG_PATH: &str = "~\\AppData\\Local";

pub struct AppDirectories {
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppDirectories {
    pub fn load_directories() -> Self {
        let config_dir = Self::get_config_directory();
        let log_dir = Self::get_log_directory();
        Self {
            config_dir,
            log_dir,
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn get_config_directory() -> PathBuf {
        env::var_os("XDG_CONFIG_HOME")
            .map_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH), |s| PathBuf::from(&s))
            .join(APP_NAME)
    }

    #[cfg(target_os = "windows")]
    fn get_config_directory() -> PathBuf {
        env::var_os("APPDATA")
            .map_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH), |v| PathBuf::from(&v))
            .join(APP_NAME)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn get_log_directory() -> PathBuf {
        env::var_os("XDG_DATA_HOME")
            .map_or_else(|| PathBuf::from(DEFAULT_LOG_PATH), |s| PathBuf::from(&s))
            .join(APP_NAME)
    }

    #[cfg(target_os = "windows")]
    fn get_log_directory() -> PathBuf {
        env::var_os("LOCALAPPDATA")
            .map_or_else(|| PathBuf::from(DEFAULT_LOG_PATH), |v| PathBuf::from(&v))
            .join(APP_NAME)
    }
}
