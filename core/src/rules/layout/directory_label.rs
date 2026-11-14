// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;

use crate::source_path::SourcePath;

// A directory as an offence names it: with forward slashes, and the package
// root -- no name at all, or a dot -- said in words.
pub struct DirectoryLabel;

impl DirectoryLabel {
    pub fn of(path: &SourcePath) -> String {
        let shown = path.as_str().replace('\\', "/");
        if shown.is_empty() || shown == "." {
            return "the package root".to_string();
        }
        shown
    }
}
