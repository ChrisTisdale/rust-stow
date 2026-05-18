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

use crate::commands::stow_data::StowFilter;
use crate::commands::{CommandError, CommandOperation, RestowData, StowData, UnstowData};
use grep::matcher::Matcher;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, instrument, trace, warn};

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
///             .clone_with_target(parent)
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

impl<TIter: Iterator<Item = Result<PathBuf, CommandError>>, TCommand: CommandOperation<TIter>> Debug
    for Command<TIter, TCommand>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stow(d) => f.debug_struct("Stow").field("data", &d.data).finish(),
            Self::Unstow(d) => f.debug_struct("Unstow").field("data", &d.data).finish(),
            Self::Restow(d) => f.debug_struct("Restow").field("data", &d.data).finish(),
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
    ///             .clone_with_target(parent)
    ///             .stow();
    ///         let command = builder.build()?;
    ///         command.execute()?;
    ///     }
    ///
    ///     return Ok(());
    /// }
    /// ```
    #[instrument(level = "trace")]
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

    fn path_matches_filter<TMethod: Display + ?Sized, TMatcher: Matcher>(
        caller: &TMethod,
        entry_path: &Path,
        filters: &[TMatcher],
    ) -> bool {
        trace!(
            "{caller} - Checking if file matches entry: {}",
            entry_path.display()
        );

        if let Some(name) = entry_path.as_os_str().to_str() {
            for matcher in filters {
                if matcher.is_match(name.as_bytes()).unwrap_or(false) {
                    info!("{caller} - entry found: {}", entry_path.display());
                    return true;
                }
            }
        } else {
            warn!(
                "Failed to get file name for entry: {}",
                entry_path.display()
            );

            return false;
        }

        debug!(
            "{caller} - No matching entry found: {}",
            entry_path.display()
        );
        false
    }

    fn is_ignored(entry_path: &Path, filter: &StowFilter) -> bool {
        trace!(
            "Checking if item should be ignored: {}",
            entry_path.display()
        );

        Self::path_matches_filter("Ignored", entry_path, &filter.ignored)
    }

    fn should_override(entry_path: &Path, filter: &StowFilter) -> bool {
        trace!(
            "Checking if item should be overridden: {}",
            entry_path.display()
        );

        Self::path_matches_filter("Override", entry_path, &filter.overrides)
    }

    fn process_directory_entry(entry: &Path, args: &StowData, operation: &mut TCommand) -> Result<(), CommandError> {
        trace!("Processing directory entry: {}", entry.display());
        Self::process_directory(entry, args, operation, Self::stow_item)
    }

    fn process_directory<TData, F>(
        entry: &Path,
        args: &TData,
        operation: &mut TCommand,
        mut processor: F,
    ) -> Result<(), CommandError>
    where
        F: FnMut(&Path, &TData, &mut TCommand) -> Result<(), CommandError>,
    {
        for entry in operation.read_directory(entry)? {
            match entry {
                Ok(e) => processor(&e, args, operation)?,
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
        let file_name = Self::get_item_name(item, args.options.dot_file_prefix.as_ref())?;
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

    fn get_item_name(item: &Path, prefix: Option<&String>) -> Result<OsString, CommandError> {
        let file_name = item.file_name().map_or_else(
            || Err(CommandError::InvalidStowItem(item.display().to_string())),
            Ok,
        )?;

        if let Some(prefix) = prefix
            && let Some(name) = file_name.to_str()
            && let Some(stripped) = name.strip_prefix(prefix)
        {
            let updated = format!(".{stripped}");
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
            &args.clone_with_target(full_path.to_path_buf()),
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
        } else if operation.is_directory(item) && operation.is_directory(full_path) {
            info!(
                "Directory already exists traversing its children.  Stowing children of: {}",
                full_path.display()
            );

            Self::process_directory_entry(
                item,
                &args.clone_with_target(full_path.to_path_buf()),
                operation,
            )?;

            Ok(())
        } else if Self::should_override(full_path, &args.options.filter) && operation.is_file(full_path) {
            info!("Overriding existing file: {}", full_path.display());
            operation.remove_item(full_path)?;
            operation.link_item(item, full_path)?;

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
        info!(
            "Unstowing files from {} to {}",
            args.directory.display(),
            args.target.display()
        );

        Self::unstow_directory_entry(&args.directory, args, operation)?;
        Ok(())
    }

    fn unstow_directory_entry(entry: &Path, args: &UnstowData, operation: &mut TCommand) -> Result<(), CommandError> {
        Self::process_directory(entry, args, operation, Self::unstow_item)
    }

    fn unstow_item(item: &Path, args: &UnstowData, operation: &mut TCommand) -> Result<(), CommandError> {
        let item_name = Self::get_item_name(item, args.dot_file_prefix.as_ref())?;
        let full_path = args.target.join(item_name);

        if operation.is_symlink(&full_path) && operation.read_link(&full_path).is_ok_and(|p| p == item) {
            info!("Removing symlink: {}", full_path.display());
            operation.remove_link(&full_path)?;
            Self::cleanup_empty_parent(&full_path, &args.target, operation)?;
        } else if operation.is_directory(&full_path) && operation.is_directory(item) {
            Self::unstow_directory_entry(item, &args.clone_with_target(full_path.clone()), operation)?;

            if operation.is_directory(&full_path)
                && operation
                    .read_directory(&full_path)
                    .is_ok_and(|mut entries| entries.next().is_none())
            {
                info!("Removing empty directory: {}", full_path.display());
                operation.remove_item(&full_path)?;
                Self::cleanup_empty_parent(&full_path, &args.target, operation)?;
            }
        }

        Ok(())
    }

    fn cleanup_empty_parent(path: &Path, target_root: &Path, operation: &mut TCommand) -> Result<(), CommandError> {
        let mut current = path.parent();
        while let Some(parent) = current {
            if parent == target_root {
                break;
            }

            if operation.is_directory(parent)
                && operation
                    .read_directory(parent)
                    .is_ok_and(|mut entries| entries.next().is_none())
            {
                info!("Removing empty directory: {}", parent.display());
                operation.remove_item(parent)?;
                current = parent.parent();
            } else {
                break;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::commands::{CommandBuildError, CommandBuilder, CommandOperationImpl, DirectoryReader};
    use std::collections::HashSet;
    use std::error::Error;
    use std::path::PathBuf;
    use std::{env, fs};

    struct StowSetup {
        setup_path: PathBuf,
        directory: PathBuf,
    }

    impl StowSetup {
        fn new(test_name: &str) -> Result<Self, Box<dyn Error>> {
            Self::new_with_data(test_name, test_name)
        }

        fn new_with_data(scratch_name: &str, test_data_name: &str) -> Result<Self, Box<dyn Error>> {
            let project_root = env::var("CARGO_MANIFEST_DIR")?;
            let setup_path = PathBuf::from(&project_root)
                .join("test_data")
                .join("scratch")
                .join("stow")
                .join(scratch_name);

            let directory = PathBuf::from(project_root)
                .join("test_data")
                .join("stow_tests")
                .join(test_data_name);

            if !setup_path.exists() {
                fs::create_dir_all(&setup_path)?;
            }

            Ok(Self {
                setup_path,
                directory,
            })
        }

        fn default_builder(&self) -> CommandBuilder<CommandOperationImpl> {
            CommandBuilder::<CommandOperationImpl>::new()
                .with_target(self.setup_path.clone())
                .with_directory(self.directory.clone())
        }
    }

    impl Drop for StowSetup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.setup_path);
        }
    }

    fn validate_stow_result(path: &PathBuf, expected_files: &[PathBuf]) {
        let entries = fs::read_dir(path);
        assert!(entries.is_ok());
        for file in entries.unwrap() {
            assert!(file.is_ok());
            let file = file.unwrap();
            let path = file.path();
            if !path.is_symlink() && path.is_dir() {
                validate_stow_result(&file.path(), expected_files);
            } else {
                assert!(file.path().is_symlink());
                assert!(expected_files.contains(&file.path()));
            }
        }
    }

    #[test]
    fn existing_directory_test() {
        let setup = StowSetup::new("existing_directory_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let expected_files = [
            setup
                .setup_path
                .join("existing-directory")
                .join("new-file.txt"),
            setup.setup_path.join("linked-file.txt"),
            setup.setup_path.join("linked-directory"),
        ];

        let result = fs::create_dir_all(setup.setup_path.join("existing-directory"));
        assert!(result.is_ok());

        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());
        validate_stow_result(&setup.setup_path, &expected_files);
    }

    #[test]
    fn basic_stow_test() {
        let setup = StowSetup::new("basic_stow_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let expected_files = [
            setup.setup_path.join("linked-file.txt"),
            setup.setup_path.join("linked-directory"),
        ];

        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());
        validate_stow_result(&setup.setup_path, &expected_files);
    }

    #[test]
    fn dotfiles_stow_test() {
        let setup = StowSetup::new("dotfiles_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let expected_files = [
            setup.setup_path.join(".bashrc"),
            setup.setup_path.join("regular-file.txt"),
        ];

        let command = setup
            .default_builder()
            .stow()
            .with_dot_file_prefix(Some("dot-".to_string()))
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_ok());
        validate_stow_result(&setup.setup_path, &expected_files);

        // Verify .bashrc is a symlink to dot-bashrc
        let bashrc_link = fs::read_link(setup.setup_path.join(".bashrc")).unwrap_or_default();
        assert!(bashrc_link.ends_with("dot-bashrc"));
    }

    #[test]
    fn ignored_items_stow_test() {
        let setup = StowSetup::new("ignored_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let expected_files = [setup.setup_path.join("keep-file.txt")];

        let mut ignored = HashSet::new();
        ignored.insert("ignored-file.txt".to_string());

        let command = setup.default_builder().stow().with_ignored(ignored).build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_ok());
        validate_stow_result(&setup.setup_path, &expected_files);
        assert!(!setup.setup_path.join("ignored-file.txt").exists());
    }

    #[test]
    fn conflict_error_test() {
        let setup = StowSetup::new("conflict_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        // Create a file in the target that conflicts with something in the stow directory
        let result = fs::write(setup.setup_path.join("conflict-file.txt"), "existing");
        assert!(result.is_ok());

        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::DirectoryEntryAlreadyExists(path) => {
                assert!(path.contains("conflict-file.txt"));
            }
            e => panic!("Expected DirectoryEntryAlreadyExists error, got {e:?}"),
        }
    }

    #[test]
    fn folding_stow_test() {
        let setup = StowSetup::new("folding_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        // In folding mode (default), dir1 should be linked directly

        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_ok());

        let dir1_path = setup.setup_path.join("dir1");
        assert!(dir1_path.is_symlink());
    }

    #[test]
    fn no_folding_stow_test() {
        let setup = StowSetup::new("no_folding_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        // In no-folding mode, dir1 should be created and file1.txt linked inside it

        let command = setup.default_builder().stow().with_no_folding(true).build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_ok());

        let dir1_path = setup.setup_path.join("dir1");
        assert!(dir1_path.exists());
        assert!(dir1_path.is_dir());
        assert!(!dir1_path.is_symlink());

        let file1_path = dir1_path.join("file1.txt");
        assert!(file1_path.is_symlink());
    }

    #[test]
    fn override_existing_file_test() {
        let setup = StowSetup::new("override_file_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        let target_file = setup.setup_path.join("file.txt");
        let result = fs::write(&target_file, "existing content");
        assert!(result.is_ok());

        let mut overrides = HashSet::new();
        overrides.insert(".*file.txt".to_string());

        let command = setup
            .default_builder()
            .stow()
            .with_overrides(overrides)
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_ok());

        assert!(target_file.exists());
        assert!(target_file.is_symlink());
        let link_target = fs::read_link(&target_file);
        assert!(link_target.is_ok());
        let link_target = link_target.unwrap();
        assert!(link_target.ends_with("file.txt"));
        let content = fs::read_to_string(&target_file);
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), "original content\n");
    }

    #[test]
    fn directory_vs_file_conflict_test() {
        let setup = StowSetup::new("dir_conflict_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        // Target has a file named "dir1"
        let target_dir1 = setup.setup_path.join("dir1");
        let result = fs::write(&target_dir1, "i am a file");
        assert!(result.is_ok());

        // Source has a directory named "dir1" (setup by StowSetup::new from our manual mkdir)

        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::DirectoryEntryAlreadyExists(name) => {
                assert_eq!(name, "dir1");
            }
            e => panic!("Expected DirectoryEntryAlreadyExists error, got {e:?}"),
        }

        // It should NOT have stowed (since it's not overridden and is_directory(item) && is_directory(full_path) is false)
        assert!(target_dir1.exists());
        assert!(!target_dir1.is_symlink());
        assert!(target_dir1.is_file());
        let content = fs::read_to_string(&target_dir1);
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), "i am a file");
    }

    #[test]
    fn ignored_is_not_overridden_test() {
        let setup = StowSetup::new("ignore_override_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        let target_file = setup.setup_path.join("ignored-and-overridden.txt");

        let mut ignored = HashSet::new();
        ignored.insert(".*ignored.*".to_string());

        let mut overrides = HashSet::new();
        overrides.insert(".*overridden.*".to_string());

        let command = setup
            .default_builder()
            .stow()
            .with_ignored(ignored)
            .with_overrides(overrides)
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // With the change, is_ignored returns true if matched, and doesn't check overrides.
        // So the file should NOT be stowed.
        assert!(!target_file.exists());
    }

    #[test]
    fn target_missing_error_test() {
        let setup = StowSetup::new("target_missing_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let target_path = setup.setup_path.join("non-existent-target");
        // Ensure target path does not exist
        if target_path.exists() {
            let result = fs::remove_dir_all(&target_path);
            assert!(result.is_ok());
        }

        let command = CommandBuilder::<CommandOperationImpl>::new()
            .with_target(target_path)
            .with_directory(setup.directory.clone())
            .stow()
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidTargetDirectory(path) => {
                assert!(path.contains("non-existent-target"));
            }
            e => panic!("Expected InvalidTargetDirectory error, got {e:?}"),
        }
    }

    #[test]
    fn stow_dir_missing_error_test() {
        let setup = StowSetup::new("stow_dir_missing_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let stow_dir = setup.directory.join("non-existent-stow-dir");

        let command = CommandBuilder::<CommandOperationImpl>::new()
            .with_target(setup.setup_path.clone())
            .with_directory(stow_dir)
            .stow()
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::StowDirectoryNotFound(path) => {
                assert!(path.contains("non-existent-stow-dir"));
            }
            e => panic!("Expected StowDirectoryNotFound error, got {e:?}"),
        }
    }

    #[test]
    fn same_dir_error_test() {
        let setup = StowSetup::new("same_dir_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        let command = setup
            .default_builder()
            .with_target(setup.directory.clone())
            .stow()
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidStowDirectory(path) => {
                let dir_str = setup.directory.to_str();
                assert!(dir_str.is_some());
                assert!(path.contains(dir_str.unwrap()));
            }
            e => panic!("Expected InvalidStowDirectory error, got {e:?}"),
        }
    }

    #[test]
    fn target_is_file_error_test() {
        let setup = StowSetup::new("target_is_file_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let target_path = setup.setup_path.join("target-file");
        let result = fs::write(&target_path, "I am a file");
        assert!(result.is_ok());

        let command = setup
            .default_builder()
            .with_target(target_path)
            .stow()
            .build();
        assert!(command.is_ok());

        let result = command.unwrap().execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidTargetDirectory(path) => {
                assert!(path.contains("target-file"));
            }
            e => panic!("Expected InvalidTargetDirectory error, got {e:?}"),
        }
    }

    #[test]
    fn missing_target_build_error_test() {
        let result = CommandBuilder::<CommandOperationImpl>::new()
            .with_directory(PathBuf::from("/some/dir"))
            .stow()
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            CommandBuildError::MissingTargetDirectory => {}
            e => panic!("Expected MissingTargetDirectory error, got {e:?}"),
        }
    }

    #[test]
    fn missing_stow_dir_build_error_test() {
        let result = CommandBuilder::<CommandOperationImpl>::new()
            .with_target(PathBuf::from("/some/target"))
            .stow()
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            CommandBuildError::MissingStowDirectory => {}
            e => panic!("Expected MissingStowDirectory error, got {e:?}"),
        }
    }

    #[test]
    fn idempotent_stow_test() {
        let setup = StowSetup::new("idempotent_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();
        let expected_files = [setup.setup_path.join("file1.txt")];

        // First execution
        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());
        validate_stow_result(&setup.setup_path, &expected_files);

        // Second execution - should be idempotent and skip existing correct links
        let command2 = setup.default_builder().stow().build();
        assert!(command2.is_ok());
        let result = command2.unwrap().execute();
        assert!(result.is_ok());
        validate_stow_result(&setup.setup_path, &expected_files);
    }

    #[test]
    fn restow_test() {
        let setup = StowSetup::new("restow_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        let target_file = setup.setup_path.join("file.txt");
        let command_provider = || -> Command<DirectoryReader, CommandOperationImpl> {
            let command = setup.default_builder().restow().build();
            assert!(command.is_ok());
            command.unwrap()
        };

        // Initial stow via restow
        let command = command_provider();
        let result = command.execute();
        assert!(result.is_ok());

        assert!(target_file.exists());
        assert!(target_file.is_symlink());

        // Restow again
        let command = command_provider();
        let result = command.execute();
        assert!(result.is_ok());
        assert!(target_file.exists());
        assert!(target_file.is_symlink());
    }

    #[test]
    fn basic_unstow_test() {
        let setup = StowSetup::new_with_data("basic_unstow_test", "basic_stow_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        // Stow first
        let command = setup.default_builder().stow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify stowed
        assert!(setup.setup_path.join("linked-file.txt").exists());
        assert!(setup.setup_path.join("linked-directory").exists());

        // Unstow
        let command = setup.default_builder().unstow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify unstowed
        assert!(!setup.setup_path.join("linked-file.txt").exists());
        assert!(!setup.setup_path.join("linked-directory").exists());
    }

    #[test]
    fn unstow_with_folding_disabled_test() {
        let setup = StowSetup::new_with_data("unstow_with_folding_disabled_test", "no_folding_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        // Stow first with folding disabled
        let command = setup.default_builder().stow().with_no_folding(true).build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify stowed
        let target_dir = setup.setup_path.join("dir1");
        let target_file = target_dir.join("file1.txt");
        assert!(target_dir.exists());
        assert!(target_dir.is_dir());
        assert!(target_file.exists());
        assert!(target_file.is_symlink());

        // Unstow
        let command = setup.default_builder().unstow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify unstowed and directory cleaned up
        assert!(!target_file.exists());
        assert!(!target_dir.exists());
    }

    #[test]
    fn unstow_dotfiles_test() {
        let setup = StowSetup::new_with_data("unstow_dotfiles_test", "dotfiles_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        // Stow first with dot-file prefix
        let command = setup
            .default_builder()
            .stow()
            .with_dot_file_prefix(Some("dot-".to_string()))
            .build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify stowed
        let dot_bashrc = setup.setup_path.join(".bashrc");
        let regular_file = setup.setup_path.join("regular-file.txt");
        assert!(dot_bashrc.exists());
        assert!(dot_bashrc.is_symlink());
        assert!(regular_file.exists());
        assert!(regular_file.is_symlink());

        // Unstow
        let command = setup
            .default_builder()
            .unstow()
            .with_dot_file_prefix(Some("dot-".to_string()))
            .build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify unstowed
        assert!(!dot_bashrc.exists());
        assert!(!regular_file.exists());
    }

    #[test]
    fn unstow_nested_directories_test() {
        let setup = StowSetup::new_with_data("unstow_nested_directories_test", "basic_stow_test");
        assert!(setup.is_ok());
        let setup = setup.unwrap();

        // Stow first with folding disabled
        let command = setup.default_builder().stow().with_no_folding(true).build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify stowed
        let target_dir = setup.setup_path.join("linked-directory");
        let target_file = target_dir.join("sub-file.txt");
        let target_file2 = setup.setup_path.join("linked-file.txt");
        assert!(target_dir.exists());
        assert!(target_dir.is_dir());
        assert!(target_file.exists());
        assert!(target_file.is_symlink());
        assert!(target_file2.exists());
        assert!(target_file2.is_symlink());

        // Unstow
        let command = setup.default_builder().unstow().build();
        assert!(command.is_ok());
        let result = command.unwrap().execute();
        assert!(result.is_ok());

        // Verify unstowed and directory cleaned up
        assert!(!target_file.exists());
        assert!(!target_dir.exists());
        assert!(!target_file2.exists());
    }
}
