// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every member's files, each walked as its own scan walks it and named relative
// to the workspace root, so two members' `src/lib.rs` stay two files.

use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::in_memory_source_tree::path_of;
use stern4rust::settings::adoption::exclusion_set::ExclusionSet;
use stern4rust::source_path::SourcePath;
use stern4rust::workspace_sources::WorkspaceSources;

const ROOT: &str = "/repo";

fn paths(tree: &InMemorySourceTree, members: &[(&str, &[&str])]) -> Vec<String> {
    let root = path_of(ROOT);
    let members: Vec<(SourcePath, ExclusionSet)> = members
        .iter()
        .map(|(member, excluded)| {
            let patterns: Vec<String> = excluded
                .iter()
                .map(|pattern| (*pattern).to_string())
                .collect();
            (
                root.join(member),
                ExclusionSet::new(&patterns).expect("usable patterns"),
            )
        })
        .collect();
    WorkspaceSources::new(tree)
        .read(&root, &members)
        .iter()
        .map(|file| file.relative_path().to_string())
        .collect()
}

fn workspace(files: &[&str]) -> InMemorySourceTree {
    files.iter().fold(InMemorySourceTree::new(), |tree, file| {
        tree.with_file(&format!("{ROOT}/{file}"), "pub const LIMIT: u32 = 3;\n")
    })
}

#[test]
fn read_leaves_out_what_a_members_exclusions_cover() {
    // Arrange
    let tree = workspace(&[
        "alpha/src/lib.rs",
        "alpha/src/generated/wire.rs",
        "beta/src/generated/wire.rs",
    ]);

    // Act
    let read = paths(&tree, &[("alpha", &["src/generated/**"]), ("beta", &[])]);

    // Assert
    assert_eq!(read, vec!["alpha/src/lib.rs", "beta/src/generated/wire.rs"]);
}

#[test]
fn read_names_each_members_file_from_the_workspace_root() {
    // Arrange
    let tree = workspace(&[
        "alpha/src/lib.rs",
        "beta/src/lib.rs",
        "beta/tests/all_tests.rs",
    ]);

    // Act
    let read = paths(&tree, &[("alpha", &[]), ("beta", &[])]);

    // Assert
    assert_eq!(
        read,
        vec![
            "alpha/src/lib.rs",
            "beta/src/lib.rs",
            "beta/tests/all_tests.rs"
        ]
    );
}
