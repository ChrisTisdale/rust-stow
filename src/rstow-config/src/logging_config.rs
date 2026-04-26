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

use crate::LevelError;
use crate::rotation_error::RotationError;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use tracing_appender::rolling::Rotation;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(i64)]
pub enum LoggingLevel {
    Off = 0,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LoggingLevel {
    type Err = LevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "off" | "Off" | "OFF" => Ok(Self::Off),
            "trace" | "Trace" | "TRACE" => Ok(Self::Trace),
            "debug" | "Debug" | "DEBUG" => Ok(Self::Debug),
            "info" | "Info" | "INFO" => Ok(Self::Info),
            "warn" | "Warn" | "WARN" => Ok(Self::Warn),
            "error" | "Error" | "ERROR" => Ok(Self::Error),
            _ => Err(LevelError::InvalidLevelString(s.to_string())),
        }
    }
}

impl TryFrom<i64> for LoggingLevel {
    type Error = LevelError;

    fn try_from(value: i64) -> Result<Self, LevelError> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Trace),
            2 => Ok(Self::Debug),
            3 => Ok(Self::Info),
            4 => Ok(Self::Warn),
            5 => Ok(Self::Error),
            _ => Err(LevelError::InvalidLevel(value)),
        }
    }
}

impl Serialize for LoggingLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Off => "off",
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        })
    }
}

impl<'de> Deserialize<'de> for LoggingLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LoggingLevelVisitor;

        impl Visitor<'_> for LoggingLevelVisitor {
            type Value = LoggingLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("off or trace or debug or info or warn or error")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(de::Error::custom)
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse().map_err(de::Error::custom)
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(LoggingLevelVisitor)
    }
}

impl Display for LoggingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Trace => write!(f, "Trace"),
            Self::Debug => write!(f, "Debug"),
            Self::Info => write!(f, "Info"),
            Self::Warn => write!(f, "Warn"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl From<LevelFilter> for LoggingLevel {
    fn from(value: LevelFilter) -> Self {
        match value {
            LevelFilter::OFF => Self::Off,
            LevelFilter::TRACE => Self::Trace,
            LevelFilter::DEBUG => Self::Debug,
            LevelFilter::INFO => Self::Info,
            LevelFilter::WARN => Self::Warn,
            LevelFilter::ERROR => Self::Error,
        }
    }
}

impl From<LoggingLevel> for LevelFilter {
    fn from(value: LoggingLevel) -> Self {
        match value {
            LoggingLevel::Off => Self::OFF,
            LoggingLevel::Trace => Self::TRACE,
            LoggingLevel::Debug => Self::DEBUG,
            LoggingLevel::Info => Self::INFO,
            LoggingLevel::Warn => Self::WARN,
            LoggingLevel::Error => Self::ERROR,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(i64)]
pub enum RotationType {
    Hourly = 1,
    #[default]
    Daily,
}

impl From<RotationType> for Rotation {
    fn from(value: RotationType) -> Self {
        match value {
            RotationType::Hourly => Self::HOURLY,
            RotationType::Daily => Self::DAILY,
        }
    }
}

impl FromStr for RotationType {
    type Err = RotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hourly" | "Hourly" | "HOURLY" => Ok(Self::Hourly),
            "daily" | "Daily" | "DAILY" => Ok(Self::Daily),
            _ => Err(RotationError::InvalidRotationTypeString(s.to_string())),
        }
    }
}

impl TryFrom<i64> for RotationType {
    type Error = RotationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Hourly),
            2 => Ok(Self::Daily),
            _ => Err(RotationError::InvalidRotationType(value)),
        }
    }
}

impl Serialize for RotationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
        })
    }
}

impl<'de> Deserialize<'de> for RotationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RotationTypeVisitor;

        impl Visitor<'_> for RotationTypeVisitor {
            type Value = RotationType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("hourly or daily")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(de::Error::custom)
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse().map_err(de::Error::custom)
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_string(RotationTypeVisitor)
    }
}

impl Display for RotationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hourly => write!(f, "Hourly"),
            Self::Daily => write!(f, "Daily"),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: Option<LoggingLevel>,
    pub file: Option<PathBuf>,
    pub logging_path: Option<PathBuf>,
    pub rotation: Option<RotationType>,
    pub max_log_files: Option<usize>,
    pub color_support: Option<bool>,
}

impl Display for LoggingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoggingConfig {{ level: {:?}, file: {:?}, logging_path: {:?}, rotation: {:?}, max_log_files: {:?}, color_support: {:?} }}",
            self.level, self.file, self.logging_path, self.rotation, self.max_log_files, self.color_support
        )
    }
}
