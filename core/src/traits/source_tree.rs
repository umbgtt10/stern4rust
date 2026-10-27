// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::source_path::SourcePath;
use crate::source_read_failure::SourceReadFailure;
use crate::source_write_failure::SourceWriteFailure;

// The files a run reads and, under `--fix`, writes back. A root that does not
// exist holds no files; `rust_files` lists every `.rs` file beneath a root, in
// path order, leaving `target/` and `.git/` out.
pub trait SourceTree {
    fn contains(&self, path: &SourcePath) -> bool;

    fn read_source(&self, file: &SourcePath) -> Result<String, SourceReadFailure>;

    fn rust_files(&self, root: &SourcePath) -> Result<Vec<SourcePath>, SourceReadFailure>;

    fn write_source(&self, file: &SourcePath, contents: &str) -> Result<(), SourceWriteFailure>;
}
