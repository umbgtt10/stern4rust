// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::settings::adoption::exclusion_set::ExclusionSet;
use crate::source_file::SourceFile;
use crate::source_path::SourcePath;
use crate::source_reader::SourceReader;
use crate::traits::source_tree::SourceTree;

// Every file of a workspace's members, each member walked as its own scan walks
// it -- through its own exclusions -- and every file named relative to the
// workspace root, so two members' `src/lib.rs` stay two files.
//
// What a question about the whole workspace reads: whether a constant is
// shared depends on files in members the run may not be walking. A file that
// cannot be read reads nothing here; readable-source reports it where it is
// judged.
pub struct WorkspaceSources<'a> {
    source_tree: &'a dyn SourceTree,
}

impl<'a> WorkspaceSources<'a> {
    pub fn new(source_tree: &'a dyn SourceTree) -> Self {
        Self { source_tree }
    }

    // A member whose files cannot be listed reads nothing here, for the same
    // reason an unreadable file does: the member's own scan reports it.
    pub fn read(
        &self,
        root: &SourcePath,
        members: &[(SourcePath, ExclusionSet)],
    ) -> Vec<SourceFile> {
        let reader = SourceReader::new(self.source_tree);
        members
            .iter()
            .flat_map(|(member, exclusions)| {
                let listed = self.source_tree.rust_files(member).unwrap_or_default();
                let (kept, _) = exclusions.apply(listed, member).into_parts();
                kept.into_iter()
                    .filter_map(|path| reader.read(root, &path).ok())
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
