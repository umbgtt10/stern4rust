// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::path::PathBuf;

use walkdir::WalkDir;

// Every .rs file under a package, tests included.
//
// "Every .rs file" is the rule's own wording, so src/ and tests/ are both in
// scope -- a test file without the header is exactly as wrong as a source file
// without it. target/ is skipped because it holds generated code nobody wrote
// and build scripts write files there that no rule should judge.
pub struct SourceWalker;

impl SourceWalker {
    pub fn walk(root: &Path) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|entry| !Self::is_skipped(entry.path()))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect();
        paths.sort();
        paths
    }

    fn is_skipped(path: &Path) -> bool {
        matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("target") | Some(".git")
        )
    }
}
