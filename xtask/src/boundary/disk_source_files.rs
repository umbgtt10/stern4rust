// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::boundary::source_files::SourceFiles;
use crate::boundary::source_read_failure::SourceReadFailure;

// The `.rs` files beneath a directory on disk, in path order. A directory that
// does not exist is a failure rather than an empty tree, so a moved folder
// cannot pass the scan by holding nothing.
pub struct DiskSourceFiles;

impl DiskSourceFiles {
    pub fn new() -> Self {
        Self
    }

    fn failure(path: &Path, error: &io::Error) -> SourceReadFailure {
        SourceReadFailure {
            path: path.to_path_buf(),
            reason: error.to_string(),
        }
    }

    fn walk(dir: &Path) -> Result<Vec<PathBuf>, SourceReadFailure> {
        let entries = fs::read_dir(dir).map_err(|error| Self::failure(dir, &error))?;
        let mut files = Vec::new();
        for entry in entries {
            let path = entry.map_err(|error| Self::failure(dir, &error))?.path();
            if path.is_dir() {
                files.extend(Self::walk(&path)?);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
        Ok(files)
    }
}

impl Default for DiskSourceFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceFiles for DiskSourceFiles {
    fn read(&self, file: &Path) -> Result<String, SourceReadFailure> {
        fs::read_to_string(file).map_err(|error| Self::failure(file, &error))
    }

    fn rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>, SourceReadFailure> {
        let mut files = Self::walk(dir)?;
        files.sort();
        Ok(files)
    }
}
