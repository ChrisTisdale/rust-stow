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

use rstow_args::{CliError, CommandLineProcessor};
use tracing::{info, trace};

fn main() -> Result<(), CliError> {
    let result = process_command_line_args();
    if let Err(e) = result {
        if let CliError::CommandLineParsingError(source) = e {
            source.exit();
        }

        #[cfg(debug_assertions)]
        eprintln!("Failed to process the audit. {e}");
        return Err(e);
    }

    Ok(())
}

fn process_command_line_args() -> Result<(), CliError> {
    let args = CommandLineProcessor::get_cli_args()?;
    trace!("Processed Commandline Arguments: {args}");
    let command = args.command;
    let command_text = format!("{command}");
    command.execute()?;
    info!("Successfully processed command {command_text}");
    Ok(())
}
