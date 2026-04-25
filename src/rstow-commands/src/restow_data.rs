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

use crate::{StowData, StowOptions, UnstowData};
use std::path::PathBuf;

pub struct RestowData {
    pub(crate) unstow_data: UnstowData,
    pub(crate) stow_data: StowData,
}

impl From<RestowData> for UnstowData {
    fn from(restow_data: RestowData) -> Self {
        Self::new(
            restow_data.stow_data.target,
            restow_data.stow_data.directory,
        )
    }
}

impl AsRef<UnstowData> for RestowData {
    fn as_ref(&self) -> &UnstowData {
        &self.unstow_data
    }
}

impl AsRef<StowData> for RestowData {
    fn as_ref(&self) -> &StowData {
        &self.stow_data
    }
}

impl RestowData {
    #[must_use]
    pub fn new(target: PathBuf, directory: PathBuf, options: StowOptions) -> Self {
        Self {
            unstow_data: UnstowData::new(target.clone(), directory.clone()),
            stow_data: StowData::new(target, directory, options),
        }
    }
}
