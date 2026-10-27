// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use stern4rust::source_path::SourcePath;
use stern4rust::source_read_failure::SourceReadFailure;
use stern4rust::source_write_failure::SourceWriteFailure;
use stern4rust::traits::source_tree::SourceTree;

// The tree every core test reads, held in memory: a path either holds its
// contents or is listed and cannot be read, and what `--fix` writes back is kept
// for the test to read. No test of the core touches a disk.
pub struct InMemorySourceTree {
    files: RefCell<BTreeMap<SourcePath, Option<String>>>,
    written: RefCell<Vec<SourcePath>>,
}

impl InMemorySourceTree {
    pub fn new() -> Self {
        Self {
            files: RefCell::new(BTreeMap::new()),
            written: RefCell::new(Vec::new()),
        }
    }

    pub fn contents_of(&self, path: &str) -> Option<String> {
        self.files.borrow().get(&path_of(path)).cloned().flatten()
    }

    pub fn with_file(self, path: &str, contents: &str) -> Self {
        self.files
            .borrow_mut()
            .insert(path_of(path), Some(String::from(contents)));
        self
    }

    pub fn with_unreadable_file(self, path: &str) -> Self {
        self.files.borrow_mut().insert(path_of(path), None);
        self
    }

    pub fn written(&self) -> Vec<SourcePath> {
        self.written.borrow().clone()
    }
}

impl Default for InMemorySourceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceTree for InMemorySourceTree {
    fn contains(&self, path: &SourcePath) -> bool {
        self.files.borrow().contains_key(path)
    }

    fn read_source(&self, file: &SourcePath) -> Result<String, SourceReadFailure> {
        self.files
            .borrow()
            .get(file)
            .cloned()
            .flatten()
            .ok_or_else(|| SourceReadFailure {
                path: file.clone(),
                cause: Box::new(NotHeld),
            })
    }

    fn rust_files(&self, root: &SourcePath) -> Result<Vec<SourcePath>, SourceReadFailure> {
        Ok(self
            .files
            .borrow()
            .keys()
            .filter(|path| path.strip_prefix(root).is_some())
            .filter(|path| path.extension() == Some("rs"))
            .cloned()
            .collect())
    }

    fn write_source(&self, file: &SourcePath, contents: &str) -> Result<(), SourceWriteFailure> {
        self.written.borrow_mut().push(file.clone());
        self.files
            .borrow_mut()
            .insert(file.clone(), Some(String::from(contents)));
        Ok(())
    }
}

#[derive(Debug)]
pub struct NotHeld;

impl fmt::Display for NotHeld {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("not held")
    }
}

impl Error for NotHeld {}

pub fn path_of(text: &str) -> SourcePath {
    SourcePath::new(String::from(text))
}
