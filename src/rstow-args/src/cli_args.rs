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

use rstow_commands::{Command, CommandOperation, DirectoryReader};
use std::fmt::{Display, Formatter};
use tracing_appender::non_blocking::WorkerGuard;

pub struct CliArgs<T: CommandOperation<DirectoryReader>> {
    pub command: Command<DirectoryReader, T>,
    pub(crate) _guard: Option<WorkerGuard>,
}

impl<T: CommandOperation<DirectoryReader>> Display for CliArgs<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CliArgs {{ command: {} }}", self.command)
    }
}
