// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use xtask::boundary::disk_source_files::DiskSourceFiles;
use xtask::boundary::source_files::SourceFiles;

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(full_path, contents).expect("failed to write file");
}

#[test]
fn read_of_a_file_on_disk_returns_its_contents() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    write_file(dir.path(), "lib.rs", "pub mod a;");

    // Act
    let source = DiskSourceFiles::new().read(&dir.path().join("lib.rs"));

    // Assert
    assert_eq!(source, Ok(String::from("pub mod a;")));
}

#[test]
fn read_of_a_file_that_does_not_exist_returns_a_failure_naming_it() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    let missing = dir.path().join("lib.rs");

    // Act
    let source = DiskSourceFiles::new().read(&missing);

    // Assert
    assert_eq!(source.expect_err("missing").path, missing);
}

#[test]
fn rust_files_of_a_directory_that_does_not_exist_returns_a_failure() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");

    // Act
    let files = DiskSourceFiles::new().rust_files(&dir.path().join("src"));

    // Assert
    assert!(files.is_err());
}

#[test]
fn rust_files_of_a_tree_lists_its_rust_files_recursively_in_path_order() {
    // Arrange
    let dir = TempDir::new().expect("temp dir");
    write_file(dir.path(), "z.rs", "");
    write_file(dir.path(), "a/b.rs", "");
    write_file(dir.path(), "readme.md", "");

    // Act
    let files = DiskSourceFiles::new()
        .rust_files(dir.path())
        .expect("listable");

    // Assert
    assert_eq!(
        files,
        vec![dir.path().join("a").join("b.rs"), dir.path().join("z.rs")]
    );
}
