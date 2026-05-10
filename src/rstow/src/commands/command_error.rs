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

use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CommandError {
    #[error(transparent)]
    InvalidPath(#[from] std::io::Error),
    #[error("Invalid target directory: {0}.  The target directory must exist and be a directory.")]
    InvalidTargetDirectory(String),
    #[error("Invalid stow directory: {0}.  The stow directory must exist and be a directory.")]
    StowDirectoryNotFound(String),
    #[error("Invalid stow directory: {0}.  It must not be the same as the target directory.")]
    InvalidStowDirectory(String),
    #[error("Directory Entry Already Exists: {0}")]
    DirectoryEntryAlreadyExists(String),
    #[error(
        "The stow directory contains an invalid item: {0}.  It must be a file or directory and not a symbolic link."
    )]
    InvalidStowItem(String),
}
