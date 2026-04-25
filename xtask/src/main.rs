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

use anyhow::{Error, Result};
use clap::CommandFactory;
use clap_mangen::Man;
use rstow_args::CommandLineProcessor;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::{env, fs};

const APP_NAME: &str = "rstow";
static PROJECT_ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .map(Path::to_path_buf)
        .unwrap()
});

static DIST_DIR: LazyLock<PathBuf> = LazyLock::new(|| PROJECT_ROOT.join("target").join("dist"));

fn main() -> Result<()> {
    try_dist()
}

fn try_dist() -> Result<()> {
    let args = env::args().nth(1);
    match args.as_deref() {
        Some("dist") => dist()?,
        _ => print_help(),
    }

    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

dist            builds application and man pages
"
    );
}

fn dist() -> Result<()> {
    let dir = &*DIST_DIR;
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir)?;

    dist_binary()?;
    dist_manpage()?;
    Ok(())
}

fn dist_binary() -> Result<()> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let root = &*PROJECT_ROOT;
    let dist = &*DIST_DIR;
    let status = Command::new(cargo)
        .current_dir(PROJECT_ROOT.as_path())
        .args(["build", "--release"])
        .status()?;

    if !status.success() {
        return Err(Error::msg("cargo build failed"));
    }

    let output_dir = root.join("target").join("release").join(APP_NAME);
    fs::copy(&output_dir, dist.join(APP_NAME))?;

    Ok(())
}

fn dist_manpage() -> Result<()> {
    let command = CommandLineProcessor::command();
    let dist = &*DIST_DIR;
    let mut file = File::create(dist.join(format!("{APP_NAME}.1")))?;
    Man::new(command).render(&mut file)?;
    Ok(())
}
