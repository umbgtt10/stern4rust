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
//
// A directory holding its own Cargo.toml is skipped too, because it is a
// different package. Its files are that package's to answer for, under whatever
// rules it has chosen, and cargo would not compile them as part of this one
// either. The shape this was written for is a fixture crate under
// tests/fixtures/ -- sample code a tool analyses rather than code it ships.
pub struct SourceWalker;

impl SourceWalker {
    pub fn walk(root: &Path) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|entry| !Self::is_skipped(root, entry.path()))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect();
        paths.sort();
        paths
    }

    // The package being walked holds a manifest by definition, so it has to be
    // exempt from the nested-package rule. Without this the walk skips the root
    // and every run reports a clean tree.
    fn is_skipped(root: &Path, path: &Path) -> bool {
        if path == root {
            return false;
        }
        matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("target") | Some(".git")
        ) || Self::is_another_package(path)
    }

    fn is_another_package(path: &Path) -> bool {
        path.is_dir() && path.join("Cargo.toml").is_file()
    }
}
