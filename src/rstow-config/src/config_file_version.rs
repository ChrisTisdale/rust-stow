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

use crate::version_error::VersionError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
pub enum ConfigFileVersion {
    #[default]
    V1 = 1,
}

impl Display for ConfigFileVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
        }
    }
}

impl TryFrom<i64> for ConfigFileVersion {
    type Error = VersionError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::V1),
            _ => Err(VersionError::UnsupportedVersion(value)),
        }
    }
}

impl FromStr for ConfigFileVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "v1" | "V1" => Ok(Self::V1),
            _ => Err(VersionError::UnsupportedVersionString(s.to_string())),
        }
    }
}

impl Serialize for ConfigFileVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(*self as i64)
    }
}

impl<'de> Deserialize<'de> for ConfigFileVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ConfigFileVersionVisitor;

        impl serde::de::Visitor<'_> for ConfigFileVersionVisitor {
            type Value = ConfigFileVersion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("1 or v1 or V1")
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(serde::de::Error::custom)
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse().map_err(serde::de::Error::custom)
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(ConfigFileVersionVisitor)
    }
}
