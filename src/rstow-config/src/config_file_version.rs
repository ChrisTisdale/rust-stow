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

use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

impl Serialize for ConfigFileVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::V1 => serializer.serialize_i64(1),
        }
    }
}

impl<'de> Deserialize<'de> for ConfigFileVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = i64::deserialize(deserializer)?;
        match version {
            1 => Ok(Self::V1),
            _ => Err(serde::de::Error::custom(format!(
                "Unsupported config file version: {version}"
            ))),
        }
    }
}
