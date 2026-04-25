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

use rstow_commands::{CommandBuildError, CommandBuilder, CommandError, CommandOperationImpl};
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

#[test]
fn dotfiles_stow_test() {
    let setup = StowSetup::new("dotfiles_test");
    assert!(setup.is_ok());
    let setup = setup.unwrap();
    let expected_files = [
        setup.setup_path.join(".bashrc"),
        setup.setup_path.join("regular-file.txt"),
    ];

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .with_dot_file_prefix(Some("dot-".to_string()))
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .with_ignored(ignored)
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandError::DirectoryEntryAlreadyExists(path) => {
            assert!(path.contains("conflict-file.txt"));
        }
        e => panic!("Expected DirectoryEntryAlreadyExists error, got {e:?}"),
    }
}

#[test]
fn invalid_item_error_test() {
    let setup = StowSetup::new("invalid_item_test");
    assert!(setup.is_ok());
    let setup = setup.unwrap();
    // invalid_item_test contains a symlink, which is not allowed in stow directory according to CommandError::InvalidStowItem

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandError::InvalidStowItem(path) => {
            assert!(path.contains("symlink-in-stow"));
        }
        e => panic!("Expected InvalidStowItem error, got {e:?}"),
    }
}

#[test]
fn folding_stow_test() {
    let setup = StowSetup::new("folding_test");
    assert!(setup.is_ok());
    let setup = setup.unwrap();
    // In folding mode (default), dir1 should be linked directly

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .with_no_folding(true)
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
    assert!(result.is_ok());

    let dir1_path = setup.setup_path.join("dir1");
    assert!(dir1_path.exists());
    assert!(dir1_path.is_dir());
    assert!(!dir1_path.is_symlink());

    let file1_path = dir1_path.join("file1.txt");
    assert!(file1_path.is_symlink());
}

#[test]
fn overrides_stow_test() {
    let setup = StowSetup::new("override_test");
    assert!(setup.is_ok());
    let setup = setup.unwrap();
    let expected_files = [setup.setup_path.join("ignored-but-overridden.txt")];

    let mut ignored = HashSet::new();
    ignored.insert(".*ignored.*".to_string());

    let mut overrides = HashSet::new();
    overrides.insert(".*overridden.*".to_string());

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .with_ignored(ignored)
        .with_overrides(overrides)
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
    assert!(result.is_ok());
    validate_stow_result(&setup.setup_path, &expected_files);
    assert!(setup.setup_path.join("ignored-but-overridden.txt").exists());
    assert!(!setup.setup_path.join("truly-ignored.txt").exists());
}

#[test]
fn target_missing_error_test() {
    let setup = StowSetup::new("target_missing_test");
    assert!(setup.is_ok());
    let setup = setup.unwrap();
    let target_path = setup.setup_path.join("non-existent-target");
    // Ensure target path does not exist
    if target_path.exists() {
        fs::remove_dir_all(&target_path).unwrap();
    }

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(target_path)
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
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
    let command = command.unwrap();

    let result = command.execute();
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.directory.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandError::InvalidStowDirectory(path) => {
            assert!(path.contains(setup.directory.to_str().unwrap()));
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

    let command = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(target_path)
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command.is_ok());
    let command = command.unwrap();

    let result = command.execute();
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
    let err = result.err().unwrap();
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
    let err = result.err().unwrap();
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

    // Second execution - should be idempotent and skip existing correct links
    let command2 = CommandBuilder::<CommandOperationImpl>::new()
        .with_target(setup.setup_path.clone())
        .with_directory(setup.directory.clone())
        .stow()
        .build();

    assert!(command2.is_ok());
    let command2 = command2.unwrap();

    let result2 = command2.execute();
    assert!(result2.is_ok());
    validate_stow_result(&setup.setup_path, &expected_files);
}
