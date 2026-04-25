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

use crate::ConfigError;
use std::env;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::path;

/// Resolves a given path to an absolute path, expanding the home directory symbol (`~`) if present.
///
/// # Platform Support
/// This function only compiles on Windows due to the `#[cfg(target_os = "windows")]` attribute.
///
/// # Arguments
/// * `path` - A reference to a `Path` object representing a relative or tilde-prefixed path.
///
/// # Returns
/// * `Ok(PathBuf)` - The absolute path corresponding to the input path if resolution is successful.
/// * `Err(ConfigError)` - An error if the path cannot be resolved, either due to:
///   - Failure to retrieve the user's home directory.
///   - Failure to compute an absolute path.
///
/// # Behavior
/// - If the path does **not** start with `~`, it simply returns the absolute path of the input using `path::absolute`.
/// - If the path **does** start with `~`, it expands it to the user's home directory (`env::home_dir`)
///   and appends the remaining path components, then computes and returns the absolute path.
///
/// # Errors
/// This function returns a `ConfigError::UnableToResolveDirectory` error in cases where:
/// - The user's home directory cannot be determined.
/// - An absolute path cannot be computed for the resolved path.
///
/// # Example
/// ```
/// use std::path::Path;
/// use rstow_config::path_resolver;
///
/// let path = Path::new("~/Documents/config.txt");
/// match path_resolver::resolve_path(&path) {
///     Ok(resolved_path) => println!("Resolved path: {}", resolved_path.display()),
///     Err(e) => eprintln!("Failed to resolve path: {}", e),
/// }
/// ```
#[cfg(target_os = "windows")]
pub fn resolve_path(path: &Path) -> Result<PathBuf, ConfigError> {
    if !path.starts_with("~") {
        let path = path::absolute(&path)?;
        return Ok(path);
    }

    let home_dir = env::home_dir().ok_or_else(|| ConfigError::UnableToResolveDirectory(path.display().to_string()))?;
    let path = path.strip_prefix("~")?;
    let resolved_path = home_dir.join(path);
    Ok(path::absolute(&resolved_path)?)
}

/// Resolves a given file path, expanding the home directory (`~`) if necessary,
/// and returning a canonicalized absolute path.
///
/// This function is only available on non-Windows target operating systems,
/// as indicated by the `#[cfg(not(target_os = "windows"))]` attribute.
///
/// If the `path` does not start with the `~` character (home directory shorthand),
/// the function directly attempts to canonicalize the path using the filesystem.
///
/// If the `path` starts with `~`, the function resolves the user's home directory
/// using `env::home_dir`. It then joins the resolved home directory with the rest
/// of the path (after stripping the `~` prefix), and finally canonicalizes it.
///
/// # Parameters
/// - `path`: A reference to a `Path` representing the input path to be resolved.
///
/// # Returns
/// - `Ok(PathBuf)`: The resolved absolute path as a `PathBuf`.
/// - `Err(ConfigError)`: Returns a `ConfigError` in the following cases:
///     - The function is unable to retrieve the user’s home directory.
///     - The provided path cannot be canonicalized.
///     - The path after prefix stripping cannot be joined or further resolved.
///
/// # Errors
/// - `ConfigError::UnableToResolveDirectory`: Thrown when the home directory cannot be resolved.
/// - Any `std::io::Error` occurring during path canonicalization will also be propagated
///   as part of the `Result`.
///
/// # Example
/// ```rust
/// use std::path::Path;
/// use rstow_config::path_resolver;
///
/// let input_path = Path::new("~/example/path");
/// match path_resolver::resolve_path(input_path) {
///     Ok(resolved_path) => println!("Resolved Path: {}", resolved_path.display()),
///     Err(error) => eprintln!("Error resolving path: {:?}", error),
/// }
/// ```
#[cfg(not(target_os = "windows"))]
pub fn resolve_path(path: &Path) -> Result<PathBuf, ConfigError> {
    if !path.starts_with("~") {
        let path = path.canonicalize()?;
        return Ok(path);
    }

    let home_dir = env::home_dir().ok_or_else(|| ConfigError::UnableToResolveDirectory(path.display().to_string()))?;
    let path = path.strip_prefix("~")?;
    let resolved_path = home_dir.join(path);
    Ok(resolved_path.canonicalize()?)
}
