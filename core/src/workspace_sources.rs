// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::path::PathBuf;

use crate::adoption::exclusion_set::ExclusionSet;
use crate::source_file::SourceFile;
use crate::source_reader::SourceReader;
use crate::source_walker::SourceWalker;

// Every file of a workspace's members, each member walked as its own scan walks
// it -- through its own exclusions -- and every file named relative to the
// workspace root, so two members' `src/lib.rs` stay two files.
//
// What a question about the whole workspace reads: whether a constant is
// shared depends on files in members the run may not be walking. A file that
// cannot be read reads nothing here; readable-source reports it where it is
// judged.
pub struct WorkspaceSources;

impl WorkspaceSources {
    pub fn read(root: &Path, members: &[(PathBuf, ExclusionSet)]) -> Vec<SourceFile> {
        members
            .iter()
            .flat_map(|(member, exclusions)| {
                exclusions
                    .apply(SourceWalker::walk(member), member)
                    .kept
                    .into_iter()
                    .filter_map(|path| SourceReader::read(root, &path).ok())
            })
            .collect()
    }
}
