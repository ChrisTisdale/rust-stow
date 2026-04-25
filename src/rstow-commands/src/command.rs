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

use crate::stow_data::StowFilter;
use crate::{CommandError, CommandOperation, RestowData, StowData, UnstowData};
use grep::matcher::Matcher;
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, trace, warn};

/// Represents a structure that encapsulates data and an associated operation related to command processing.
///
/// This generic structure provides a way to associate a dataset (`TData`) with a specific operation
/// (`TOperation`) that processes an iterator of results (`TIter`). The `PhantomData` marker is used
/// to indicate that this structure logically depends on `TIter` without actually owning any instances of it.
///
/// # Type Parameters
/// - `TData`: The type of the data associated with the command.
/// - `TIter`: An iterator type that yields `Result<PathBuf, CommandError>` items. This represents
///   the source of paths and potential errors during command execution.
/// - `TOperation`: A type that implements the `CommandOperation<TIter>` trait. This defines the
///   operation associated with processing the command's data.
pub struct CommandData<
    TData,
    TIter: Iterator<Item = Result<PathBuf, CommandError>>,
    TOperation: CommandOperation<TIter>,
> {
    pub(crate) data: TData,
    pub(crate) operation: TOperation,
    pub(crate) _marker: std::marker::PhantomData<TIter>,
}

/// Represents a set of commands for managing stowing operations with associated data.
///
/// The `Command` enum encapsulates three different operations: `Stow`, `Unstow`, and `Restow`.
/// Each operation is parameterized over custom iterator and command types, enabling flexibility
/// and extensibility for different use cases.
///
/// # Type Parameters
///
/// - `TIter`: A type implementing the `Iterator` trait, where each item is a `Result<PathBuf, CommandError>`.
///   This parameter defines the iterator responsible for providing results of path operations.
/// - `TCommand`: A type implementing the `CommandOperation` trait, representing the command behavior.
///
/// # Variants
///
/// * `Stow`:
///     - Contains `CommandData` specialized with `StowData`.
///     - Represents an operation to "stow" (move or organize) specific resources associated with paths.
///
/// * `Unstow`:
///     - Contains `CommandData` specialized with `UnstowData`.
///     - Represents an operation to "unstow" (revert or remove organization) specific resources tied to paths.
///
/// * `Restow`:
///     - Contains `CommandData` specialized with `RestowData`.
///     - Represents an operation to "restow" (reorganize or refresh organization) specific resources associated with paths.
///
/// # Example
/// ```
/// use std::error::Error;
/// use rstow_commands::{Command, CommandBuilder, CommandOperationImpl, CommandError, StowData, StowOptions};
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let directory = std::env::current_dir()?;
///     let parent = directory.parent().map(|p| p.to_path_buf());
///     if let Some(parent) = parent {
///         let builder = CommandBuilder::<CommandOperationImpl>::new()
///             .with_directory(directory)
///             .with_target(parent)
///             .stow();
///         let command = builder.build()?;
///         println!("Built command: {command}");
///     }
///
///     return Ok(());
/// }
/// ```
pub enum Command<TIter: Iterator<Item = Result<PathBuf, CommandError>>, TCommand: CommandOperation<TIter>> {
    Stow(CommandData<StowData, TIter, TCommand>),
    Unstow(CommandData<UnstowData, TIter, TCommand>),
    Restow(CommandData<RestowData, TIter, TCommand>),
}

impl<TIter: Iterator<Item = Result<PathBuf, CommandError>>, TCommand: CommandOperation<TIter>> Display
    for Command<TIter, TCommand>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stow(_) => write!(f, "Stow"),
            Self::Unstow(_) => write!(f, "UnStow"),
            Self::Restow(_) => write!(f, "ReStow"),
        }
    }
}

impl<TIter: Iterator<Item = Result<PathBuf, CommandError>>, TCommand: CommandOperation<TIter>>
    Command<TIter, TCommand>
{
    /// Execute the command.
    ///
    /// # Arguments
    /// * `self`: The command to execute.
    ///
    /// returns: Result<(), `CommandError`>
    /// The result of the command execution.
    ///
    /// # Errors
    /// * `CommandError::InvalidStowDirectory` - Returned when the stowed directory does not exist or is not a directory
    /// * `CommandError::InvalidTargetDirectory` - Returned when the target directory does not exist or is not a directory
    /// * `CommandError::DirectoryEntryAlreadyExists` - Returned when a directory entry already exists and the command is configured to adopt it
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use rstow_commands::{Command, CommandBuilder, CommandOperationImpl, CommandError, StowData, StowOptions};
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let directory = std::env::current_dir()?;
    ///     let parent = directory.parent().map(|p| p.to_path_buf());
    ///     if let Some(parent) = parent {
    ///         let builder = CommandBuilder::<CommandOperationImpl>::new()
    ///             .with_directory(directory)
    ///             .with_target(parent)
    ///             .stow();
    ///         let command = builder.build()?;
    ///         command.execute()?;
    ///     }
    ///
    ///     return Ok(());
    /// }
    /// ```
    pub fn execute(self) -> Result<(), CommandError> {
        match self {
            Self::Stow(mut a) => Self::process_stow(&a.data, &mut a.operation),
            Self::Unstow(mut a) => Self::process_unstow(&a.data, &mut a.operation),
            Self::Restow(mut a) => Self::process_restow(&a.data, &mut a.operation),
        }
    }

    fn process_stow(args: &StowData, operation: &mut TCommand) -> Result<(), CommandError> {
        info!(
            "Stowing files for {} to {}",
            args.directory.display(),
            args.target.display()
        );

        if args.directory == args.target {
            error!("Stow directory cannot be the same as the target directory");
            return Err(CommandError::InvalidStowDirectory(
                args.directory.display().to_string(),
            ));
        }

        if !operation.is_directory(&args.target) {
            error!("Target directory does not exist or is not a directory");
            return Err(CommandError::InvalidTargetDirectory(
                args.target.display().to_string(),
            ));
        }

        if !operation.is_directory(&args.directory) {
            error!("Stow directory does not exist or is not a directory");
            return Err(CommandError::StowDirectoryNotFound(
                args.directory.display().to_string(),
            ));
        }

        Self::process_directory_entry(&args.directory, args, operation)?;
        Ok(())
    }

    fn is_ignored(entry_path: &Path, filter: &StowFilter) -> bool {
        trace!(
            "Checking if ignore file matches entry: {}",
            entry_path.display()
        );

        for matcher in &filter.ignored {
            if let Some(name) = entry_path.as_os_str().to_str() {
                if matcher.is_match(name.as_bytes()).unwrap_or(false) {
                    let overridden = filter
                        .overrides
                        .iter()
                        .any(|o| o.is_match(name.as_bytes()).unwrap_or(false));

                    trace!(
                        "Ignoring entry: {}.  Overriding: {}",
                        entry_path.display(),
                        overridden
                    );

                    return !overridden;
                }
            } else {
                warn!(
                    "Failed to get file name for entry: {}",
                    entry_path.display()
                );
            }
        }

        false
    }

    fn process_directory_entry(entry: &Path, args: &StowData, operation: &mut TCommand) -> Result<(), CommandError> {
        trace!("Processing directory entry: {}", entry.display());
        for entry in operation.read_directory(entry)? {
            match entry {
                Ok(e) => Self::stow_item(&e, args, operation)?,
                Err(e) => warn!("Failed to read directory entry: {e}"),
            }
        }

        Ok(())
    }

    fn stow_item(item: &Path, args: &StowData, operation: &mut TCommand) -> Result<(), CommandError> {
        trace!("Reviewing directory entry: {}", item.display());
        if Self::is_ignored(item, &args.options.filter) {
            debug!("Ignoring item: {}", item.display());
            return Ok(());
        }

        let updated_root = item
            .strip_prefix(&args.directory)
            .map(|p| Path::new("/").join(p));

        if let Ok(updated_root) = updated_root
            && Self::is_ignored(&updated_root, &args.options.filter)
        {
            debug!("Ignoring item: {}", item.display());
            return Ok(());
        }

        if operation.is_symlink(item) {
            error!(
                "Stow directory contains a symbolic link: {}",
                item.display()
            );

            return Err(CommandError::InvalidStowItem(item.display().to_string()));
        }

        let no_folding = operation.is_directory(item) && args.options.no_folding;
        let file_name = Self::get_file_name(item, args)?;
        let full_path = args.target.join(file_name);
        trace!(
            "Stowing directory entry: {}.  With no folding: {}",
            item.display(),
            no_folding
        );

        if no_folding && !operation.exists(&full_path) {
            return Self::process_fold(args, item, &full_path, operation);
        }

        if operation.exists(&full_path) {
            return Self::handle_existing_item(item, args, &full_path, operation);
        }

        operation.link_item(item, &full_path)?;
        Ok(())
    }

    fn get_file_name(item: &Path, args: &StowData) -> Result<OsString, CommandError> {
        let file_name = item.file_name().map_or_else(
            || Err(CommandError::InvalidStowItem(item.display().to_string())),
            Ok,
        )?;

        if let Some(prefix) = args.options.dot_file_prefix.as_ref()
            && let Some(name) = file_name.to_str()
            && name.starts_with(prefix)
        {
            let updated = ".".to_string() + name.trim_start_matches(prefix);
            trace!("Updating file name: {name} to {updated}");
            return Ok(OsString::from(&updated));
        }

        Ok(file_name.to_owned())
    }

    fn process_fold(
        args: &StowData,
        entry_path: &Path,
        full_path: &Path,
        operation: &mut TCommand,
    ) -> Result<(), CommandError> {
        info!("Creating directory: {}", full_path.display());
        operation.create_directory(full_path)?;
        Self::process_directory_entry(
            entry_path,
            &StowData {
                directory: args.directory.clone(),
                target: full_path.to_path_buf(),
                options: args.options.clone(),
            },
            operation,
        )
    }

    fn handle_existing_item(
        item: &Path,
        args: &StowData,
        full_path: &Path,
        operation: &mut TCommand,
    ) -> Result<(), CommandError> {
        if operation.is_symlink(full_path) && operation.read_link(full_path)? == item {
            info!("Skipping existing symlink: {}", full_path.display());
            Ok(())
        } else if operation.is_directory(item) {
            info!(
                "Directory already exists traversing its children.  Stowing children of: {}",
                full_path.display()
            );

            Self::process_directory_entry(
                item,
                &StowData {
                    directory: args.directory.clone(),
                    target: full_path.to_path_buf(),
                    options: args.options.clone(),
                },
                operation,
            )?;

            Ok(())
        } else {
            warn!("File already exists: {}", full_path.display());

            Err(CommandError::DirectoryEntryAlreadyExists(
                item.file_name()
                    .unwrap_or_else(|| OsStr::new("Unknown Name"))
                    .to_string_lossy()
                    .to_string(),
            ))
        }
    }

    fn process_unstow(args: &UnstowData, operation: &mut TCommand) -> Result<(), CommandError> {
        info!("Unstowing files");
        Self::unstow_directory_filter(&args.target, args, operation, |path| path == args.directory)?;
        Ok(())
    }

    fn unstow_directory_filter<P>(
        target: &Path,
        args: &UnstowData,
        operation: &mut TCommand,
        mut skip: P,
    ) -> Result<(), CommandError>
    where
        P: FnMut(&Path) -> bool,
    {
        for entry in operation.read_directory(target)? {
            match entry {
                Ok(e) => {
                    trace!("Reviewing directory entry: {}", e.display());
                    if skip(&e) {
                        info!("Skipping target directory: {}", e.display());
                        continue;
                    }

                    Self::cleanup_symlink(args, &e, operation)?;
                }
                Err(e) => warn!("Failed to read directory entry: {e}"),
            }
        }
        Ok(())
    }

    fn unstow_directory(target: &Path, args: &UnstowData, operation: &mut TCommand) -> Result<(), CommandError> {
        for entry in operation.read_directory(target)? {
            match entry {
                Ok(e) => {
                    trace!("Reviewing directory entry: {}", e.display());
                    Self::cleanup_symlink(args, &e, operation)?;
                }
                Err(e) => warn!("Failed to read directory entry: {e}"),
            }
        }
        Ok(())
    }

    fn cleanup_symlink(args: &UnstowData, entry_path: &Path, operation: &mut TCommand) -> Result<(), CommandError> {
        if operation.is_symlink(entry_path)
            && operation
                .read_link(entry_path)
                .is_ok_and(|p| p.starts_with(&args.directory))
        {
            operation.remove_item(entry_path)?;
        } else if operation.is_directory(entry_path) {
            Self::unstow_directory(entry_path, args, operation)?;
        }

        Ok(())
    }

    fn process_restow(args: &RestowData, operation: &mut TCommand) -> Result<(), CommandError> {
        info!("Restowing files");
        Self::process_unstow(args.as_ref(), operation)?;
        Self::process_stow(args.as_ref(), operation)?;
        Ok(())
    }
}
