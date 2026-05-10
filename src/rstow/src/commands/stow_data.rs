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

use grep::pcre2::{RegexMatcher, RegexMatcherBuilder};
use std::fmt::Display;
use std::path::PathBuf;
use tracing::{debug, trace, warn};

#[derive(Clone, Default)]
pub struct StowFilter {
    pub(crate) ignored: Vec<RegexMatcher>,
    pub(crate) overrides: Vec<RegexMatcher>,
}

#[derive(Clone, Default)]
pub struct StowOptions {
    pub(crate) filter: StowFilter,
    pub(crate) dot_file_prefix: Option<String>,
    pub(crate) no_folding: bool,
}

pub struct StowData {
    pub(crate) target: PathBuf,
    pub(crate) directory: PathBuf,
    pub(crate) options: StowOptions,
}

impl StowOptions {
    #[must_use]
    pub fn new<T: AsRef<str> + Display, I: Iterator<Item = T>, O: Iterator<Item = T>>(
        dot_file_prefix: Option<String>,
        no_folding: bool,
        ignored: I,
        overrides: O,
    ) -> Self {
        trace!("Creating stow options");
        debug!("Creating ignore matches");
        let ignored = Self::create_matches(ignored);
        debug!("Creating override matches");
        let overrides = Self::create_matches(overrides);
        Self {
            no_folding,
            dot_file_prefix,
            filter: StowFilter { ignored, overrides },
        }
    }

    fn create_matches<T: AsRef<str> + Display, I: Iterator<Item = T>>(ignored: I) -> Vec<RegexMatcher> {
        debug!("Creating matches items");
        ignored
            .filter_map(|item| {
                trace!("Adding matched item: {item}");
                match RegexMatcherBuilder::new().build(item.as_ref()) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        warn!("Failed to create file matcher: {e}");
                        None
                    }
                }
            })
            .collect()
    }
}

impl StowData {
    #[must_use]
    pub const fn new(target: PathBuf, directory: PathBuf, options: StowOptions) -> Self {
        Self {
            target,
            directory,
            options,
        }
    }
}
