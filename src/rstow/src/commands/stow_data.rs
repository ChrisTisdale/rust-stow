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
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use tracing::{debug, instrument, trace, warn};

#[derive(Clone, Default)]
pub struct StowFilter {
    pub(crate) ignored: Vec<RegexMatcher>,
    pub(crate) overrides: Vec<RegexMatcher>,
}

impl Debug for StowFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StowFilter")
            .field("ignored", &self.ignored.len())
            .field("overrides", &self.overrides.len())
            .finish()
    }
}

#[derive(Clone, Default, Debug)]
pub struct StowOptions {
    pub(crate) filter: StowFilter,
    pub(crate) dot_file_prefix: Option<String>,
    pub(crate) no_folding: bool,
}

#[derive(Debug, Clone)]
pub struct StowData {
    pub(crate) target: PathBuf,
    pub(crate) directory: PathBuf,
    pub(crate) options: StowOptions,
}

impl StowOptions {
    #[must_use]
    #[instrument(level = "trace", skip(ignored, overrides))]
    pub fn new<T: AsRef<str> + Display + Debug, I: Iterator<Item = T>, O: Iterator<Item = T>>(
        dot_file_prefix: Option<String>,
        no_folding: bool,
        ignored: I,
        overrides: O,
    ) -> Self {
        trace!("Creating stow options");
        debug!("Creating ignore matches");
        let ignored = ignored.filter_map(Self::build_matcher).collect();
        debug!("Creating override matches");
        let overrides = overrides.filter_map(Self::build_matcher).collect();
        Self {
            no_folding,
            dot_file_prefix,
            filter: StowFilter { ignored, overrides },
        }
    }

    #[instrument(level = "trace")]
    fn build_matcher<T: AsRef<str> + Display + Debug>(item: T) -> Option<RegexMatcher> {
        debug!("Adding matched item: {item}");
        match RegexMatcherBuilder::new().build(item.as_ref()) {
            Ok(m) => Some(m),
            Err(e) => {
                warn!("Failed to create file matcher: {e}");
                None
            }
        }
    }
}

impl StowData {
    #[must_use]
    #[instrument(level = "trace")]
    pub fn new(target: PathBuf, directory: PathBuf, options: StowOptions) -> Self {
        Self {
            target,
            directory,
            options,
        }
    }

    #[must_use]
    pub fn clone_with_target(&self, target: PathBuf) -> Self {
        Self {
            target,
            directory: self.directory.clone(),
            options: self.options.clone(),
        }
    }
}
