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

use crate::CommandError;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};
use std::{fs, os};
use tracing::{debug, info, warn};

pub trait CommandOperation<T: Iterator<Item = Result<PathBuf, CommandError>>> {
    /// Creates a symbolic link from the `source` path to the `target` path.
    ///
    /// # Parameters
    /// - `target`: A reference to the path where the symbolic link will be created.
    /// - `source`: A reference to the path that the symbolic link will point to.
    ///
    /// # Returns
    /// - `Ok(())`: If the symbolic link is successfully created.
    /// - `Err(CommandError)`: If there is an error during the symbolic link creation process.
    ///
    /// # Errors
    /// This method returns a `CommandError` if:
    /// - The `target` or `source` path is invalid.
    /// - The underlying operating system fails to create the symbolic link.
    ///
    /// # Platform Compatibility
    /// - This method relies on the operating system's ability to create symbolic links,
    ///   which may have restrictions or require specific permissions on some platforms.
    ///   For example, on Windows, symbolic links require administrative privileges.
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let mut instance = CommandOperationImpl::default();
    /// let result = instance.link_item(
    ///     Path::new("/path/to/target"),
    ///     Path::new("/path/to/source")
    /// );
    ///
    /// match result {
    ///     Ok(_) => println!("Symbolic link created successfully."),
    ///     Err(e) => eprintln!("Failed to create symbolic link: {:?}", e),
    /// }
    /// ```
    fn link_item(&mut self, target: &Path, source: &Path) -> Result<(), CommandError>;

    /// Removes a symbolic link or junction point at the specified target path.
    ///
    /// # Parameters
    /// - `target`: A reference to a [`Path`] indicating the target of the link to be removed.
    ///
    /// # Returns
    /// - `Ok(())`: If the link was successfully removed.
    /// - `Err(CommandError)`: If an error occurs during the removal process. This could happen due to
    ///   insufficient permissions, the target not being a link, or other system-level issues.
    ///
    /// # Errors
    /// - Returns a `CommandError` if:
    ///   - The target path does not exist.
    ///   - The target is not a symbolic link or junction.
    ///   - There are not enough permissions to remove the link.
    ///   - Any other I/O or system-level error occurs during the operation.
    ///
    /// # Remarks
    /// This function operates on symbolic links and junction points. It does not follow the link or
    /// delete any file or directory that the link points to. Ensure the target path exists and is
    /// a valid link before calling this function.
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let mut instance = CommandOperationImpl::default();
    /// let link_path = Path::new("/path/to/symlink");
    /// match instance.remove_link(link_path) {
    ///     Ok(_) => println!("Link removed successfully."),
    ///     Err(err) => eprintln!("Failed to remove link: {}", err),
    /// }
    /// ```
    fn remove_link(&mut self, target: &Path) -> Result<(), CommandError>;

    /// Removes a specified item from a given path.
    ///
    /// This method attempts to remove the item located at the provided `target` path.
    /// If the operation is successful, it returns `Ok(())`. If an error occurs during
    /// the removal process, it will return a `CommandError` with details of the failure.
    ///
    /// # Parameters
    /// - `target`: A reference to a [`Path`] that specifies the location of the item to be removed.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(CommandError)` if there is an issue removing the item.
    ///
    /// # Errors
    /// This method will return a [`CommandError`] in scenarios such as:
    /// - The path does not exist.
    /// - Insufficient permissions to remove the item.
    /// - The target is in use and cannot be removed.
    /// - Any other I/O-related errors.
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let mut instance = CommandOperationImpl::default();
    /// let path = Path::new("/path/to/item");
    /// match instance.remove_item(&path) {
    ///     Ok(_) => println!("Item was successfully removed."),
    ///     Err(e) => eprintln!("Failed to remove item: {:?}", e),
    /// }
    /// ```
    fn remove_item(&mut self, target: &Path) -> Result<(), CommandError>;

    /// Creates a new directory at the specified target path.
    ///
    /// # Arguments
    /// * `target` - A reference to the [`Path`] where the directory should be created.
    ///
    /// # Returns
    /// * `Ok(())` if the directory was successfully created.
    /// * `Err(CommandError)` if there was an error during the directory creation process.
    ///
    /// # Errors
    /// This function will return a `CommandError` in the following scenarios:
    /// - If the target path is invalid.
    /// - If the directory already exists and cannot be replaced (depending on the implementation).
    /// - If there are not enough permissions to create the directory at the target location.
    /// - If there is another I/O-related error during the creation process.
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let mut instance = CommandOperationImpl::default();
    /// let path = Path::new("/some/directory");
    /// match instance.create_directory(path) {
    ///     Ok(_) => println!("Directory created successfully!"),
    ///     Err(e) => eprintln!("Failed to create directory: {:?}", e),
    /// }
    /// ```
    fn create_directory(&mut self, target: &Path) -> Result<(), CommandError>;

    /// Checks if the given path corresponds to a directory.
    ///
    /// This function takes a reference to a `Path` and determines whether the
    /// path represents a directory on the filesystem. If the metadata for the
    /// given path cannot be retrieved (e.g., the path does not exist or there
    /// are insufficient permissions), the function will return `false`.
    ///
    /// # Arguments
    ///
    /// * `target` - A reference to a `Path` representing the file system entry
    ///   to check.
    ///
    /// # Returns
    ///
    /// * `true` if the path corresponds to a directory.
    /// * `false` if the path is not a directory, does not exist, or if an error
    ///   occurs while accessing the path's metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use std::fs;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let instance = CommandOperationImpl::default();
    /// let path = Path::new("./example_directory");
    /// let is_dir = instance.is_directory(&path);
    /// println!("Is it a directory? {}", is_dir);
    /// ```
    fn is_directory(&self, target: &Path) -> bool;

    /// Checks whether the given path refers to a file.
    ///
    /// # Parameters
    /// - `target`: A reference to a `Path` that represents the filesystem path to be checked.
    ///
    /// # Returns
    /// - `true` if the specified path exists and points to a regular file.
    /// - `false` if the path does not exist, cannot be accessed, or does not refer to a file.
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let instance = CommandOperationImpl::default();
    /// let file_path = Path::new("example.txt");
    /// let result = instance.is_file(file_path);
    /// println!("Is file: {}", result);
    /// ```
    fn is_file(&self, target: &Path) -> bool;

    /// Checks if the given path is a symbolic link.
    ///
    /// # Parameters
    /// - `target`: A reference to a `Path` that represents the target to be checked.
    ///
    /// # Returns
    /// - `true` if the `target` is a symbolic link.
    /// - `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let instance = CommandOperationImpl::default();
    /// let path = Path::new("/some/path");
    /// let is_link = instance.is_symlink(&path);
    /// println!("Is symlink: {}", is_link);
    /// ```
    ///
    /// # Notes
    /// - This function uses the `is_symlink` method provided by the `Path` type
    ///   under the hood to determine if the path is a symbolic link.
    fn is_symlink(&self, target: &Path) -> bool;

    /// Reads the symbolic link at the specified target path and returns the resolved path.
    ///
    /// This function attempts to read the symbolic link located at the given `target` path and
    /// resolves it to the actual path it points to. If the operation is successful, it returns
    /// the resolved path as a `PathBuf`. If an error occurs while reading the symbolic link,
    /// the function returns a `CommandError`.
    ///
    /// # Arguments
    ///
    /// * `target` - A reference to a `Path` representing the location of the symbolic link to read.
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, CommandError>` - On success, the resolved path is returned as a `PathBuf`.
    ///   On failure, an error of type `CommandError` is returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The specified `target` does not exist or is not a symbolic link.
    /// * There are not enough permissions to read the symbolic link.
    /// * Other I/O errors occur during the operation.
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let instance = CommandOperationImpl::default();
    /// let file_path = Path::new("example.txt");
    /// let result = instance.read_link(file_path);
    /// match result {
    ///     Ok(path) => println!("Resolved path: {}", path.display()),
    ///     Err(e) => println!("Error reading link: {}", e),
    /// }
    /// ```
    fn read_link(&self, target: &Path) -> Result<PathBuf, CommandError>;

    /// Checks if a given file path or directory exists.
    ///
    /// # Arguments
    ///
    /// * `target` - A reference to a `Path` that specifies the file or directory to check.
    ///
    /// # Returns
    ///
    /// * `true` if the specified path exists in the filesystem.
    /// * `false` if the specified path does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let instance = CommandOperationImpl::default();
    /// let path = Path::new("/some/file/path");
    /// let result = instance.exists(&path);
    /// println!("Does the path exist? {}", result);
    /// ```
    fn exists(&self, target: &Path) -> bool;

    /// Reads the contents of a directory at the specified path.
    ///
    /// # Parameters
    /// - `target`: A reference to a [`Path`] specifying the directory to read.
    ///
    /// # Returns
    /// - `Ok(ReadDir)`: An iterator over the entries within the directory if successful.
    /// - `Err(CommandError)`: An error if the directory cannot be read.
    ///
    /// # Errors
    /// This function will return an error in the following cases:
    /// - The provided path does not exist.
    /// - The path is not a directory.
    /// - Insufficient permissions to read the directory.
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandOperation, CommandOperationImpl};
    ///
    /// let instance = CommandOperationImpl::default();
    /// let path = Path::new("/some/directory");
    /// match instance.read_directory(path) {
    ///     Ok(entries) => {
    ///         for entry in entries {
    ///             println!("{:?}", entry);
    ///         }
    ///     },
    ///     Err(e) => println!("Failed to read directory: {}", e),
    /// }
    /// ```
    fn read_directory(&self, target: &Path) -> Result<T, CommandError>;
}

/// Represents the implementation type of a command operation.
///
/// This enum has two variants:
///
/// - `Default`: The default implementation type, used when no specific operation is specified.
/// - `Simulated`: Represents a simulated implementation type, often used for testing or scenarios where the operation doesn't interact with a real system.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum CommandOperationImpl {
    #[default]
    Default,
    Simulated(SimulatedData),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LinkedDirectory {
    path: PathBuf,
    link_path: PathBuf,
}

/// A structure representing simulated data for tracking created directories and symbolic links.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SimulatedData {
    created_directories: Vec<PathBuf>,
    created_links: Vec<LinkedDirectory>,
}

/// A struct that provides an iterator for reading entries in a directory.
///
/// The `DirectoryReader` struct wraps a `ReadDir` instance, which is returned
/// by the standard library's `std::fs::read_dir` function. It allows for
/// iterating over the entries of a directory and accessing metadata such as
/// file names and file types.
pub struct DirectoryReader {
    read_dir: ReadDir,
}

impl From<ReadDir> for DirectoryReader {
    fn from(read_dir: ReadDir) -> Self {
        Self { read_dir }
    }
}

impl Iterator for DirectoryReader {
    type Item = Result<PathBuf, CommandError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_dir
            .next()
            .map(|res| res.map_err(Into::into).map(|entry| entry.path()))
    }
}

impl CommandOperation<DirectoryReader> for CommandOperationImpl {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn link_item(&mut self, item: &Path, target: &Path) -> Result<(), CommandError> {
        match self {
            Self::Default => {
                info!("Linking {} {}", item.display(), target.display());
                os::unix::fs::symlink(item, target)?;
            }
            Self::Simulated(data) => {
                data.created_links.push(LinkedDirectory {
                    path: item.to_path_buf(),
                    link_path: target.to_path_buf(),
                });

                println!("LINK: {} => {}", item.display(), target.display());
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn link_item(&mut self, item: &Path, target: &Path) -> Result<(), CommandError> {
        match self {
            Self::Default => {
                info!("Linking {} {}", item.display(), target.display());
                if self.is_directory(item) {
                    os::windows::fs::symlink_dir(item, target)?;
                } else {
                    os::windows::fs::symlink_file(item, target)?;
                }
            }
            Self::Simulated(data) => {
                data.created_links.push(LinkedDirectory {
                    path: item.to_path_buf(),
                    link_path: target.to_path_buf(),
                });

                println!("LINK: {} => {}", item.display(), target.display());
            }
        }

        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn remove_link(&mut self, entry_path: &Path) -> Result<(), CommandError> {
        if !self.is_symlink(entry_path) {
            warn!("Not a symlink: {}", entry_path.display());
            return Ok(());
        }

        match self {
            Self::Default => {
                debug!("Deleting symlink: {}", entry_path.display());
                fs::remove_file(entry_path)?;
            }
            Self::Simulated(_) => {
                println!("Will delete symlink: {}", entry_path.display());
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn remove_link(&mut self, entry_path: &Path) -> Result<(), CommandError> {
        if !self.is_symlink(entry_path) {
            warn!("Not a symlink: {}", entry_path.display());
            return Ok(());
        }

        match self {
            Self::Default => {
                debug!("Deleting symlink: {}", entry_path.display());
                if self.is_directory(entry_path) {
                    fs::remove_dir(entry_path)?;
                } else {
                    fs::remove_file(entry_path)?;
                }
            }
            Self::Simulated(_) => {
                println!("Will delete symlink: {}", entry_path.display());
            }
        }

        Ok(())
    }

    fn remove_item(&mut self, target: &Path) -> Result<(), CommandError> {
        match self {
            Self::Default => {
                debug!("Deleting file: {}", target.display());
                fs::remove_file(target)?;
            }
            Self::Simulated(_) => println!("RM: {}", target.display()),
        }

        Ok(())
    }

    fn create_directory(&mut self, target: &Path) -> Result<(), CommandError> {
        match self {
            Self::Default => {
                debug!("Creating directory: {}", target.display());
                fs::create_dir_all(target)?;
            }
            Self::Simulated(data) => {
                data.created_directories.push(target.to_owned());
                println!("MKDIR: {}", target.display());
            }
        }

        Ok(())
    }

    fn is_directory(&self, target: &Path) -> bool {
        if fs::metadata(target).is_ok_and(|meta| meta.is_dir()) {
            return true;
        }

        match self {
            Self::Default => false,
            Self::Simulated(data) => data.created_directories.contains(&target.to_path_buf()),
        }
    }

    fn is_file(&self, target: &Path) -> bool {
        fs::metadata(target).is_ok_and(|meta| meta.is_file())
    }

    fn is_symlink(&self, target: &Path) -> bool {
        fs::symlink_metadata(target).is_ok_and(|meta| meta.is_symlink())
    }

    fn read_link(&self, target: &Path) -> Result<PathBuf, CommandError> {
        target.read_link().map_err(Into::into)
    }

    fn exists(&self, target: &Path) -> bool {
        if fs::exists(target).unwrap_or(false) {
            return true;
        }

        match self {
            Self::Default => false,
            Self::Simulated(data) => {
                data.created_directories.contains(&target.to_path_buf())
                    || data
                        .created_links
                        .iter()
                        .any(|link| link.link_path == target)
            }
        }
    }

    fn read_directory(&self, target: &Path) -> Result<DirectoryReader, CommandError> {
        fs::read_dir(target).map_err(Into::into).map(Into::into)
    }
}
