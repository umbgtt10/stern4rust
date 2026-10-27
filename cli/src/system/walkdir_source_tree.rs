// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;

use stern4rust::source_path::SourcePath;
use stern4rust::source_read_failure::SourceReadFailure;
use stern4rust::source_write_failure::SourceWriteFailure;
use stern4rust::traits::source_tree::SourceTree;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::system::portable_path::PortablePath;

// Every .rs file under a package, tests included.
//
// "Every .rs file" is the rule's own wording, so src/ and tests/ are both in
// scope -- a test file without the header is exactly as wrong as a source file
// without it. target/ is skipped because it holds generated code nobody wrote
// and build scripts write files there that no rule should judge.
//
// Nothing else is skipped. An earlier version also skipped any directory
// holding its own Cargo.toml, so that a fixture crate nested under
// tests/fixtures/ would not be judged as though this package had written it.
// That was the wrong place to solve it: sample code a tool analyses does not
// belong inside the package that ships, and a linter quietly declining to look
// at part of a tree is the kind of silence this tool exists to refuse. The fix
// belongs in the layout -- fixtures live beside the package, not within it --
// and where a tree genuinely cannot be moved, in an explicit exclusion the
// reader can see in the report.
//
// A Rust file whose path is not UTF-8 cannot become a `SourcePath`, so it fails
// the listing by name rather than being judged under a name it does not have.
#[derive(Debug, Clone, Copy, Default)]
pub struct WalkdirSourceTree;

impl WalkdirSourceTree {
    pub fn new() -> Self {
        Self
    }

    fn is_skipped(path: &Path) -> bool {
        matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("target") | Some(".git")
        )
    }

    fn source_path(entry: &DirEntry) -> Result<SourcePath, SourceReadFailure> {
        PortablePath::of(entry.path()).ok_or_else(|| SourceReadFailure {
            path: PortablePath::of_text(&entry.path().to_string_lossy()),
            cause: Box::new(Error::new(
                ErrorKind::InvalidData,
                "the path is not valid UTF-8",
            )),
        })
    }
}

impl SourceTree for WalkdirSourceTree {
    fn contains(&self, path: &SourcePath) -> bool {
        Path::new(path.as_str()).exists()
    }

    fn read_source(&self, file: &SourcePath) -> Result<String, SourceReadFailure> {
        fs::read_to_string(Path::new(file.as_str())).map_err(|error| SourceReadFailure {
            path: file.clone(),
            cause: Box::new(error),
        })
    }

    fn rust_files(&self, root: &SourcePath) -> Result<Vec<SourcePath>, SourceReadFailure> {
        let mut paths: Vec<_> = WalkDir::new(root.as_str())
            .into_iter()
            .filter_entry(|entry| !Self::is_skipped(entry.path()))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "rs")
            })
            .collect();
        paths.sort_by(|left, right| left.path().cmp(right.path()));
        paths.iter().map(Self::source_path).collect()
    }

    fn write_source(&self, file: &SourcePath, contents: &str) -> Result<(), SourceWriteFailure> {
        fs::write(Path::new(file.as_str()), contents).map_err(|error| SourceWriteFailure {
            path: file.clone(),
            cause: Box::new(error),
        })
    }
}
