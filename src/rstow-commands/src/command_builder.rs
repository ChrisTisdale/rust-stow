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

use crate::command::CommandData;
use crate::command_operation::SimulatedData;
use crate::{
    Command, CommandBuildError, CommandOperation, CommandOperationImpl, DirectoryReader, RestowData, StowData,
    StowOptions, UnstowData,
};
use std::collections::HashSet;
use std::path::PathBuf;

/// Builds `Command` values for stow, unstow, and restow operations.
///
/// This builder stores the shared command configuration, such as the target
/// directory, stow directory, and the selected command operation mode.
#[derive(Default)]
pub struct CommandBuilder<T: CommandOperation<DirectoryReader>> {
    target: Option<PathBuf>,
    directory: Option<PathBuf>,
    operation: T,
}

/// Builder for constructing stow commands.
///
/// This type extends [`CommandBuilder`] with stow-specific options such as
/// ignored patterns, folder folding, and adopt behavior.
#[derive(Default)]
pub struct StowCommandBuilder<T: CommandOperation<DirectoryReader>> {
    builder: CommandBuilder<T>,
    ignored: HashSet<String>,
    overrides: HashSet<String>,
    dot_file_prefix: Option<String>,
    no_folding: bool,
}

/// Builder for constructing unstow commands.
///
/// This type wraps [`CommandBuilder`] and provides the configuration needed
/// to build an unstow command.
#[derive(Default)]
pub struct UnstowCommandBuilder<T: CommandOperation<DirectoryReader>> {
    builder: CommandBuilder<T>,
}

/// Builder for constructing restow commands.
///
/// This type reuses the stowed configuration builder and adds restow-specific
/// command construction.
#[derive(Default)]
pub struct RestowCommandBuilder<T: CommandOperation<DirectoryReader>> {
    stow_command: StowCommandBuilder<T>,
}

impl<T: CommandOperation<DirectoryReader> + Default> CommandBuilder<T> {
    /// Creates a new command builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target directory for the command.
    #[must_use]
    pub fn with_target(mut self, target: PathBuf) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets the stowed directory for the command.
    #[must_use]
    pub fn with_directory(mut self, directory: PathBuf) -> Self {
        self.directory = Some(directory);
        self
    }

    /// Switches the builder to simulated command execution mode.
    ///
    /// In simulated mode, filesystem changes are reported rather than applied.
    #[must_use]
    pub fn simulate(self) -> CommandBuilder<CommandOperationImpl> {
        CommandBuilder::<CommandOperationImpl> {
            target: self.target,
            directory: self.directory,
            operation: CommandOperationImpl::Simulated(SimulatedData::default()),
        }
    }

    /// Switches the builder to normal command execution mode.
    #[must_use]
    pub fn command(self) -> CommandBuilder<CommandOperationImpl> {
        CommandBuilder::<CommandOperationImpl> {
            target: self.target,
            directory: self.directory,
            operation: CommandOperationImpl::Default,
        }
    }

    /// Converts this builder into a stow-command builder.
    #[must_use]
    pub fn stow(self) -> StowCommandBuilder<T> {
        StowCommandBuilder {
            builder: self,
            ignored: HashSet::new(),
            overrides: HashSet::new(),
            dot_file_prefix: None,
            no_folding: false,
        }
    }

    /// Converts this builder into an unstow-command builder.
    #[must_use]
    pub const fn unstow(self) -> UnstowCommandBuilder<T> {
        UnstowCommandBuilder { builder: self }
    }

    /// Converts this builder into a restow-command builder.
    #[must_use]
    pub fn restow(self) -> RestowCommandBuilder<T> {
        RestowCommandBuilder {
            stow_command: self.stow(),
        }
    }
}

impl<T: CommandOperation<DirectoryReader> + Default> UnstowCommandBuilder<T> {
    /// Creates a new unstow command builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target directory for the unstow command.
    #[must_use]
    pub fn with_target(mut self, target: PathBuf) -> Self {
        self.builder = self.builder.with_target(target);
        self
    }

    /// Sets the stowed directory for the unstow command.
    #[must_use]
    pub fn with_directory(mut self, directory: PathBuf) -> Self {
        self.builder = self.builder.with_directory(directory);
        self
    }

    /// Switches the unstow command into simulated execution mode.
    #[must_use]
    pub fn simulate(self) -> UnstowCommandBuilder<CommandOperationImpl> {
        UnstowCommandBuilder::<CommandOperationImpl> {
            builder: self.builder.simulate(),
        }
    }

    /// Switches the unstow command into normal execution mode.
    #[must_use]
    pub fn command(self) -> UnstowCommandBuilder<CommandOperationImpl> {
        UnstowCommandBuilder::<CommandOperationImpl> {
            builder: self.builder.command(),
        }
    }

    /// Builds the unstow command with the provided configuration.
    ///
    /// Returns a `Result` containing the constructed `Command` if successful, or a `CommandBuildError` if any required configuration is missing.
    ///
    /// # Errors
    ///
    /// `CommandBuildError::MissingTargetDirectory`: Indicates that the target directory for unstow operation is missing.
    /// `CommandBuildError::MissingStowDirectory`: Indicates that the stowed directory for unstow operation is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandBuilder, CommandOperationImpl};
    ///
    /// let command = CommandBuilder::<CommandOperationImpl>::new()
    ///     .simulate()
    ///     .with_target(Path::new("/path/to/target").to_path_buf())
    ///     .with_directory(Path::new("/path/to/stow").to_path_buf())
    ///     .unstow()
    ///     .build();
    ///
    /// match command {
    ///    Ok(cmd) => {
    ///        // Successfully built the Command
    ///    },
    ///    Err(e) => {
    ///        eprintln!("Failed to build command: {:?}", e);
    ///    }
    /// }
    /// ```
    pub fn build(self) -> Result<Command<DirectoryReader, T>, CommandBuildError> {
        let target = self
            .builder
            .target
            .map_or_else(|| Err(CommandBuildError::MissingTargetDirectory), Ok)?;
        let directory = self
            .builder
            .directory
            .map_or_else(|| Err(CommandBuildError::MissingStowDirectory), Ok)?;

        let data = UnstowData::new(target, directory);
        Ok(Command::Unstow(CommandData {
            data,
            operation: self.builder.operation,
            _marker: std::marker::PhantomData,
        }))
    }
}

impl<T: CommandOperation<DirectoryReader> + Default> StowCommandBuilder<T> {
    /// Creates a new stow command builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables folder folding while stowing.
    #[must_use]
    pub const fn with_no_folding(mut self, no_folding: bool) -> Self {
        self.no_folding = no_folding;
        self
    }

    /// Sets the target directory for the stow command.
    #[must_use]
    pub fn with_target(mut self, target: PathBuf) -> Self {
        self.builder = self.builder.with_target(target);
        self
    }

    /// Sets the stowed directory for the stow command.
    #[must_use]
    pub fn with_directory(mut self, directory: PathBuf) -> Self {
        self.builder = self.builder.with_directory(directory);
        self
    }

    /// Replaces the current set of ignored patterns.
    #[must_use]
    pub fn with_ignored(mut self, ignored: HashSet<String>) -> Self {
        self.ignored = ignored;
        self
    }

    /// Adds a single ignored pattern.
    #[must_use]
    pub fn with_ignored_item(mut self, item: String) -> Self {
        self.ignored.insert(item);
        self
    }

    /// Replaces the current set of overrides.
    #[must_use]
    pub fn with_overrides(mut self, overrides: HashSet<String>) -> Self {
        self.overrides = overrides;
        self
    }

    /// Adds a single override pattern.
    #[must_use]
    pub fn with_override_item(mut self, item: String) -> Self {
        self.overrides.insert(item);
        self
    }

    /// Sets the dot file prefix for the stow command.
    #[must_use]
    pub fn with_dot_file_prefix(mut self, prefix: Option<String>) -> Self {
        self.dot_file_prefix = prefix;
        self
    }

    /// Switches the stow command into simulated execution mode.
    #[must_use]
    pub fn simulate(self) -> StowCommandBuilder<CommandOperationImpl> {
        StowCommandBuilder::<CommandOperationImpl> {
            builder: self.builder.simulate(),
            ignored: self.ignored,
            overrides: HashSet::new(),
            dot_file_prefix: self.dot_file_prefix,
            no_folding: self.no_folding,
        }
    }

    /// Switches the stow command into normal execution mode.
    #[must_use]
    pub fn command(self) -> StowCommandBuilder<CommandOperationImpl> {
        StowCommandBuilder::<CommandOperationImpl> {
            builder: self.builder.command(),
            ignored: self.ignored,
            overrides: self.overrides,
            dot_file_prefix: self.dot_file_prefix,
            no_folding: self.no_folding,
        }
    }

    /// Builds a `Command<T>` object from the current state of the builder.
    /// This method validates the builder's configuration and constructs a `Command`
    /// if all required fields are properly initialized. If any required fields are
    /// missing, an appropriate error is returned.
    ///
    /// returns: `Result<Command<T>, CommandBuildError>` A `Command` object constructed from the builder's configuration.
    ///
    /// # Errors
    ///
    /// * `CommandBuildError::MissingTargetDirectory` - Returned if the `target` directory is not specified in the builder.
    /// * `CommandBuildError::MissingStowDirectory` - Returned if the `stow` directory is not specified in the builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandBuilder, CommandOperationImpl};
    ///
    /// let command = CommandBuilder::<CommandOperationImpl>::new()
    ///     .simulate()
    ///     .with_target(Path::new("/path/to/target").to_path_buf())
    ///     .with_directory(Path::new("/path/to/stow").to_path_buf())
    ///     .stow()
    ///     .build();
    ///
    /// match command {
    ///    Ok(cmd) => {
    ///        // Successfully built the Command
    ///    },
    ///    Err(e) => {
    ///        eprintln!("Failed to build command: {:?}", e);
    ///    }
    /// }
    /// ```
    pub fn build(self) -> Result<Command<DirectoryReader, T>, CommandBuildError> {
        let operation = self.builder.operation;
        let target = self
            .builder
            .target
            .map_or_else(|| Err(CommandBuildError::MissingTargetDirectory), Ok)?;
        let directory = self
            .builder
            .directory
            .map_or_else(|| Err(CommandBuildError::MissingStowDirectory), Ok)?;
        let stow_options = StowOptions::new(
            self.dot_file_prefix,
            self.no_folding,
            self.ignored.iter(),
            self.overrides.iter(),
        );

        let data = StowData::new(target, directory, stow_options);
        Ok(Command::Stow(CommandData {
            data,
            operation,
            _marker: std::marker::PhantomData,
        }))
    }
}

impl<T: CommandOperation<DirectoryReader> + Default> RestowCommandBuilder<T> {
    /// Creates a new restow command builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables folder folding while restowing.
    #[must_use]
    pub fn with_no_folding(mut self, no_folding: bool) -> Self {
        self.stow_command = self.stow_command.with_no_folding(no_folding);
        self
    }

    /// Sets the target directory for the restow command.
    #[must_use]
    pub fn with_target(mut self, target: PathBuf) -> Self {
        self.stow_command = self.stow_command.with_target(target);
        self
    }

    /// Sets the stowed directory for the restow command.
    #[must_use]
    pub fn with_directory(mut self, directory: PathBuf) -> Self {
        self.stow_command = self.stow_command.with_directory(directory);
        self
    }

    /// Replaces the current set of ignored patterns.
    #[must_use]
    pub fn with_ignored(mut self, ignored: HashSet<String>) -> Self {
        self.stow_command = self.stow_command.with_ignored(ignored);
        self
    }

    /// Adds a single ignored pattern.
    #[must_use]
    pub fn with_ignored_item(mut self, item: String) -> Self {
        self.stow_command = self.stow_command.with_ignored_item(item);
        self
    }

    /// Replaces the current set of overrides.
    #[must_use]
    pub fn with_overrides(mut self, overrides: HashSet<String>) -> Self {
        self.stow_command = self.stow_command.with_overrides(overrides);
        self
    }

    /// Adds a single override pattern.
    #[must_use]
    pub fn with_override_item(mut self, item: String) -> Self {
        self.stow_command = self.stow_command.with_override_item(item);
        self
    }

    /// Switches the restow command into simulated execution mode.
    #[must_use]
    pub fn simulate(self) -> RestowCommandBuilder<CommandOperationImpl> {
        RestowCommandBuilder::<CommandOperationImpl> {
            stow_command: self.stow_command.simulate(),
        }
    }

    /// Switches the restow command into normal execution mode.
    #[must_use]
    pub fn command(self) -> RestowCommandBuilder<CommandOperationImpl> {
        RestowCommandBuilder::<CommandOperationImpl> {
            stow_command: self.stow_command.command(),
        }
    }

    /// Sets the dot file prefix for the restow command.
    #[must_use]
    pub fn with_dot_file_prefix(mut self, prefix: Option<String>) -> Self {
        self.stow_command = self.stow_command.with_dot_file_prefix(prefix);
        self
    }

    /// Builds a `Command<T>` object from the current state of the builder.
    /// This method validates the builder's configuration and constructs a `Command`
    /// if all required fields are properly initialized. If any required fields are
    /// missing, an appropriate error is returned.
    ///
    /// returns: `Result<Command<T>, CommandBuildError>` A `Command` object constructed from the builder's configuration.
    ///
    /// # Errors
    ///
    /// * `CommandBuildError::MissingTargetDirectory` - Returned if the `target` directory is not specified in the builder.
    /// * `CommandBuildError::MissingStowDirectory` - Returned if the `stow` directory is not specified in the builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use rstow_commands::{CommandBuilder, CommandOperationImpl};
    ///
    /// let command = CommandBuilder::<CommandOperationImpl>::new()
    ///     .simulate()
    ///     .with_target(Path::new("/path/to/target").to_path_buf())
    ///     .with_directory(Path::new("/path/to/stow").to_path_buf())
    ///     .restow()
    ///     .build();
    ///
    /// match command {
    ///    Ok(cmd) => {
    ///        // Successfully built the Command
    ///    },
    ///    Err(e) => {
    ///        eprintln!("Failed to build command: {:?}", e);
    ///    }
    /// }
    /// ```
    pub fn build(self) -> Result<Command<DirectoryReader, T>, CommandBuildError> {
        let cmd = self.stow_command;
        let operation = cmd.builder.operation;
        let target = cmd
            .builder
            .target
            .map_or_else(|| Err(CommandBuildError::MissingTargetDirectory), Ok)?;
        let directory = cmd
            .builder
            .directory
            .map_or_else(|| Err(CommandBuildError::MissingStowDirectory), Ok)?;

        let stow_options = StowOptions::new(
            cmd.dot_file_prefix,
            cmd.no_folding,
            cmd.ignored.iter(),
            cmd.overrides.iter(),
        );

        let data = RestowData::new(target, directory, stow_options);
        Ok(Command::Restow(CommandData {
            data,
            operation,
            _marker: std::marker::PhantomData,
        }))
    }
}
