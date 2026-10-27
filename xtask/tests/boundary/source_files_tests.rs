// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use xtask::boundary::source_files::SourceFiles;
use xtask::boundary::source_read_failure::SourceReadFailure;

// The seam the boundary gate reads source through. Lives here, beside the trait
// it implements, so the gate's tests share one fake: files held in memory, and a
// directory it was not given is one it cannot list.
pub struct InMemorySourceFiles {
    files: BTreeMap<PathBuf, String>,
}

impl InMemorySourceFiles {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files
            .insert(PathBuf::from(path), String::from(contents));
        self
    }
}

impl Default for InMemorySourceFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceFiles for InMemorySourceFiles {
    fn read(&self, file: &Path) -> Result<String, SourceReadFailure> {
        self.files.get(file).cloned().ok_or(SourceReadFailure {
            path: file.to_path_buf(),
            reason: String::from("not held"),
        })
    }

    fn rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>, SourceReadFailure> {
        let files: Vec<PathBuf> = self
            .files
            .keys()
            .filter(|path| path.starts_with(dir))
            .cloned()
            .collect();
        if files.is_empty() {
            return Err(SourceReadFailure {
                path: dir.to_path_buf(),
                reason: String::from("not held"),
            });
        }
        Ok(files)
    }
}

#[test]
fn read_of_a_file_it_holds_returns_its_contents() {
    // Arrange
    let sources = InMemorySourceFiles::new().with_file("/w/core/src/lib.rs", "pub mod a;");

    // Act
    let source = sources.read(Path::new("/w/core/src/lib.rs"));

    // Assert
    assert_eq!(source, Ok(String::from("pub mod a;")));
}

#[test]
fn rust_files_of_a_directory_it_holds_nothing_under_returns_a_failure() {
    // Arrange
    let sources = InMemorySourceFiles::new();

    // Act
    let files = sources.rust_files(Path::new("/w/core/src"));

    // Assert
    assert!(files.is_err());
}
