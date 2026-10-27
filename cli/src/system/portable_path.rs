// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::{MAIN_SEPARATOR, Path};

use stern4rust::source_path::SourcePath;

// Where a platform path becomes the core's: `/` is the core's only separator, so
// on Windows each `\` turns into one; on Unix `\` is an ordinary byte of a name
// and stays. `None` for a path that is not UTF-8, which the core cannot hold.
pub struct PortablePath;

impl PortablePath {
    pub fn of(path: &Path) -> Option<SourcePath> {
        path.to_str().map(Self::of_text)
    }

    pub fn of_text(text: &str) -> SourcePath {
        if MAIN_SEPARATOR == '\\' {
            SourcePath::new(text.replace('\\', "/"))
        } else {
            SourcePath::new(String::from(text))
        }
    }
}
