// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every member's files, each walked as its own scan walks it and named relative
// to the workspace root, so two members' `src/lib.rs` stay two files.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use stern4rust::settings::adoption::exclusion_set::ExclusionSet;
use stern4rust::workspace_sources::WorkspaceSources;

fn paths(root: &Path, members: &[(&str, &[&str])]) -> Vec<String> {
    let members: Vec<(PathBuf, ExclusionSet)> = members
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
    WorkspaceSources::read(root, &members)
        .iter()
        .map(|file| file.relative_path().to_string())
        .collect()
}

fn workspace(name: &str, files: &[&str]) -> PathBuf {
    let root = env::temp_dir().join(format!("stern4rust_sources_{name}"));
    let _ = fs::remove_dir_all(&root);
    for file in files {
        let path = root.join(file);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create parents");
        fs::write(path, "pub const LIMIT: u32 = 3;\n").expect("write the file");
    }
    root
}

#[test]
fn read_leaves_out_what_a_members_exclusions_cover() {
    // Arrange
    let root = workspace(
        "excluded",
        &[
            "alpha/src/lib.rs",
            "alpha/src/generated/wire.rs",
            "beta/src/generated/wire.rs",
        ],
    );

    // Act
    let read = paths(&root, &[("alpha", &["src/generated/**"]), ("beta", &[])]);

    // Assert
    assert_eq!(read, vec!["alpha/src/lib.rs", "beta/src/generated/wire.rs"]);
}

#[test]
fn read_names_each_members_file_from_the_workspace_root() {
    // Arrange
    let root = workspace(
        "named",
        &[
            "alpha/src/lib.rs",
            "beta/src/lib.rs",
            "beta/tests/all_tests.rs",
        ],
    );

    // Act
    let read = paths(&root, &[("alpha", &[]), ("beta", &[])]);

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
