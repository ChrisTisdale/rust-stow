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

use rstow_commands::{CommandBuilder, CommandOperationImpl};
use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

struct StowSetup {
    setup_path: PathBuf,
    directory: PathBuf,
}

impl StowSetup {
    fn new(test_name: &str) -> Result<Self, Box<dyn Error>> {
        let project_root = env::var("CARGO_MANIFEST_DIR")?;
        let setup_path = PathBuf::from(&project_root)
            .join("tests")
            .join("scratch")
            .join("stow")
            .join(test_name);

        let directory = PathBuf::from(project_root)
            .join("tests")
            .join("stow_tests")
            .join(test_name);

        if !setup_path.exists() {
            fs::create_dir_all(&setup_path)?;
        }

        Ok(Self {
            setup_path,
            directory,
        })
    }
}

impl Drop for StowSetup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.setup_path);
    }
}

fn validate_stow_result(path: &PathBuf, expected_files: &[PathBuf]) {
    for file in fs::read_dir(path).unwrap() {
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();
    let result = command.execute();
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();
    let result = command.execute();
    assert!(result.is_ok());
    validate_stow_result(&setup.setup_path, &expected_files);
}
