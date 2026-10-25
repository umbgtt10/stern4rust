// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which files of the workspace read a name, asked from inside one package: the
// file asking is named by its package-relative path and left out of the answer,
// and the readers are named by their workspace-relative paths.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use stern4rust::finding::model::name_readers::NameReaders;

fn readers() -> NameReaders {
    NameReaders::new(BTreeMap::from([(
        String::from("LIMIT"),
        BTreeSet::from([
            String::from("node/src/limits.rs"),
            String::from("node/src/pool.rs"),
            String::from("types/tests/pool_tests.rs"),
        ]),
    )]))
}

#[test]
fn others_from_the_workspace_root_reads_paths_as_they_are() {
    // Arrange
    let readers = NameReaders::new(BTreeMap::from([(
        String::from("LIMIT"),
        BTreeSet::from([String::from("src/limits.rs"), String::from("src/pool.rs")]),
    )]));

    // Act
    let others = readers.others("LIMIT", "src/limits.rs");

    // Assert
    assert_eq!(others, vec![String::from("src/pool.rs")]);
}

#[test]
fn others_of_a_name_nobody_reads_is_empty() {
    // Arrange
    let readers = readers().in_package("node");

    // Act
    let others = readers.others("UNREAD", "src/limits.rs");

    // Assert
    assert!(others.is_empty());
}

#[test]
fn others_under_a_root_keeps_that_packages_readers_by_their_package_paths() {
    // Arrange
    let readers = readers().in_package("node");

    // Act
    let others = readers.others_under("LIMIT", "src/limits.rs", "src/");

    // Assert
    assert_eq!(others, vec![String::from("src/pool.rs")]);
}

#[test]
fn others_under_from_the_workspace_root_reads_paths_as_they_are() {
    // Arrange
    let readers = readers();

    // Act
    let others = readers.others_under("LIMIT", "node/src/limits.rs", "node/src/");

    // Assert
    assert_eq!(others, vec![String::from("node/src/pool.rs")]);
}

#[test]
fn others_within_a_package_leaves_the_asking_file_out() {
    // Arrange
    let readers = readers().in_package("node");

    // Act
    let others = readers.others("LIMIT", "src/limits.rs");

    // Assert
    assert_eq!(
        others,
        vec![
            String::from("node/src/pool.rs"),
            String::from("types/tests/pool_tests.rs"),
        ]
    );
}

#[test]
fn reads_of_a_file_that_reads_a_name_is_true() {
    // Arrange
    let readers = readers().in_package("node");

    // Act
    let reads = (
        readers.reads("LIMIT", "src/limits.rs"),
        readers.reads("LIMIT", "src/executor.rs"),
    );

    // Assert
    assert_eq!(reads, (true, false));
}
