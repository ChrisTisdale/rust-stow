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
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Ignored {
    #[serde(default = "default_ignored")]
    pub file: PathBuf,
    #[serde(default = "default_comment")]
    pub comment: char,
}

impl Display for Ignored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ignored {{ file: {}, comment: '{}' }}",
            self.file.display(),
            self.comment
        )
    }
}

impl Default for Ignored {
    fn default() -> Self {
        Self {
            file: default_ignored(),
            comment: default_comment(),
        }
    }
}

const fn default_comment() -> char {
    '#'
}

fn default_ignored() -> PathBuf {
    PathBuf::from(".rstow-ignore")
}
