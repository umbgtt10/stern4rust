// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The walked files arranged as the directories they sit in.
//
// A registry declares the files beside it and the folders directly beneath it,
// so "is this file reached" is only answerable of a directory as a whole.
//
// Two decisions carry weight. `main.rs` counts as a registry alongside `lib.rs`,
// because a file declared only from `main.rs` is reached and reporting it would
// send the reader to add a line that already exists elsewhere. And a subfolder
// is a module only when it has a registry of its own -- one without is
// `tests-layout`'s finding, and declaring it would not compile.

use std::path::Path;
use stern4rust::finding::package_tree::PackageTree;
use stern4rust::source_file::SourceFile;

fn tree(paths: &[&str]) -> PackageTree {
    let files: Vec<SourceFile> = paths
        .iter()
        .map(|path| SourceFile::new(path, "\n"))
        .collect();
    PackageTree::of(&files)
}

// Ancestors are included even when they hold no file of their own, and the
// package root is one of them. A `src/` whose sources all live one level down is
// still a directory, and a rule counting what it contains has to be able to ask
// about it -- without this, a tree of nothing but subfolders is invisible.
#[test]
fn directories_includes_every_ancestor_not_only_those_holding_files() {
    // Arrange
    let tree = tree(&["src/rules/mod.rs", "tests/all_tests.rs"]);

    // Act
    let directories = tree.directories();

    // Assert
    assert_eq!(
        directories,
        [
            Path::new(""),
            Path::new("src"),
            Path::new("src/rules"),
            Path::new("tests")
        ]
    );
}

#[test]
fn expected_modules_in_counts_a_sibling_file() {
    // Arrange
    let tree = tree(&["src/lib.rs", "src/widget.rs"]);

    // Act
    let expected = tree.expected_modules_in(Path::new("src"));

    // Assert
    assert_eq!(expected, ["widget"]);
}

// A subfolder with a registry is a module in its own right.
#[test]
fn expected_modules_in_counts_a_subfolder_that_has_a_registry() {
    // Arrange
    let tree = tree(&["src/lib.rs", "src/rules/mod.rs", "src/rules/alpha.rs"]);

    // Act
    let expected = tree.expected_modules_in(Path::new("src"));

    // Assert
    assert_eq!(expected, ["rules"]);
}

// Without a registry the folder cannot be declared yet, so demanding it would
// instruct the reader to write a line that would not compile.
#[test]
fn expected_modules_in_ignores_a_subfolder_without_a_registry() {
    // Arrange
    let tree = tree(&["src/lib.rs", "src/loose/alpha.rs"]);

    // Act
    let expected = tree.expected_modules_in(Path::new("src"));

    // Assert
    assert!(expected.is_empty(), "expected none, got {expected:?}");
}

// A registry does not declare itself.
#[test]
fn expected_modules_in_ignores_the_registries_themselves() {
    // Arrange
    let tree = tree(&["src/lib.rs", "src/main.rs"]);

    // Act
    let expected = tree.expected_modules_in(Path::new("src"));

    // Assert
    assert!(expected.is_empty(), "expected none, got {expected:?}");
}

#[test]
fn files_in_lists_the_files_of_one_directory() {
    // Arrange
    let tree = tree(&["src/lib.rs", "src/widget.rs", "tests/all_tests.rs"]);

    // Act
    let files = tree.files_in(Path::new("src"));

    // Assert
    assert_eq!(files, [Path::new("src/lib.rs"), Path::new("src/widget.rs")]);
}

#[test]
fn registries_in_of_a_directory_without_one_is_empty() {
    // Arrange
    let tree = tree(&["src/widget.rs"]);

    // Act
    let registries = tree.registries_in(Path::new("src"));

    // Assert
    assert!(registries.is_empty(), "expected none, got {registries:?}");
}

// lib.rs is the index and main.rs is an entry point, so an offence about a
// missing declaration belongs on lib.rs when a package has both.
#[test]
fn registries_in_puts_lib_ahead_of_main() {
    // Arrange
    let tree = tree(&["src/main.rs", "src/lib.rs"]);

    // Act
    let registries = tree.registries_in(Path::new("src"));

    // Assert
    assert_eq!(
        registries,
        [Path::new("src/lib.rs"), Path::new("src/main.rs")]
    );
}

#[test]
fn subdirectories_of_lists_only_the_directories_directly_beneath() {
    // Arrange
    let tree = tree(&["src/lib.rs", "src/rules/mod.rs", "src/rules/deep/mod.rs"]);

    // Act
    let found = tree.subdirectories_of(Path::new("src"));

    // Assert
    assert_eq!(found, [Path::new("src/rules")]);
}
