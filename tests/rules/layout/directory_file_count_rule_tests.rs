// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A directory holds a number of files a reader can hold in their head.
//
// Registries do not count: a mod.rs is an index of the directory rather than
// something in it, and counting the list against the length of the list makes
// no sense. main.rs does count -- it is an entry point holding real code.
//
// The limit is configuration rather than a constant because it is the one rule
// whose number is taste rather than fact.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::layout::directory_file_count_rule::DirectoryFileCountRule;
use stern4rust::source_file::SourceFile;

fn check(limit: usize, paths: &[&str]) -> Vec<Offence> {
    let files: Vec<SourceFile> = paths
        .iter()
        .map(|path| SourceFile::new(path, "pub struct A;\n"))
        .collect();
    DirectoryFileCountRule::new(limit).check_workspace(&files)
}

fn check_numbered(limit: usize, directory: &str, count: usize, extra: &[&str]) -> Vec<Offence> {
    let owned = numbered(directory, count);
    let mut paths: Vec<&str> = owned.iter().map(String::as_str).collect();
    paths.extend(extra);
    check(limit, &paths)
}

fn numbered(directory: &str, count: usize) -> Vec<String> {
    (0..count).map(|n| format!("{directory}/f{n}.rs")).collect()
}

#[test]
fn check_workspace_at_the_limit_reports_nothing() {
    // Arrange & Act
    let found = check_numbered(3, "src", 3, &[]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn check_workspace_counts_each_directory_separately() {
    // Arrange
    let owned: Vec<String> = numbered("src", 3)
        .into_iter()
        .chain(numbered("tests", 3))
        .collect();
    let paths: Vec<&str> = owned.iter().map(String::as_str).collect();

    // Act
    let found = check(3, &paths);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// main.rs is an entry point holding real code, not an index.
#[test]
fn check_workspace_counts_main_against_the_limit() {
    // Arrange & Act
    let found = check_numbered(3, "src", 3, &["src/main.rs"]);

    // Assert
    assert_eq!(found.len(), 1);
    assert!(found[0].description.contains("holds 4 files"));
}

// An index of the directory is not something in it.
#[test]
fn check_workspace_does_not_count_a_registry_against_the_limit() {
    // Arrange & Act
    let found = check_numbered(3, "src", 3, &["src/lib.rs", "src/mod.rs"]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// tests/all_tests.rs is an index for the same reason lib.rs is.
#[test]
fn check_workspace_does_not_count_all_tests_against_the_limit() {
    // Arrange & Act
    let found = check_numbered(3, "tests", 3, &["tests/all_tests.rs"]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn check_workspace_of_a_directory_without_an_index_names_the_directory() {
    // Arrange & Act
    let found = check_numbered(3, "src", 4, &[]);

    // Assert
    assert_eq!(found[0].file, "src");
}

#[test]
fn check_workspace_over_the_limit_reports_the_directory() {
    // Arrange & Act
    let found = check_numbered(3, "src", 4, &[]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "directory-file-count");
    assert_eq!(found[0].subject.as_deref(), Some("src"));
    assert!(found[0].description.contains("holds 4 files"));
}

// The index is where the split has to be written, so that is where the offence
// is reported.
#[test]
fn check_workspace_reports_against_the_directorys_index() {
    // Arrange & Act
    let found = check_numbered(3, "src", 4, &["src/lib.rs"]);

    // Assert
    assert_eq!(found[0].file, "src/lib.rs");
}

#[test]
fn name_is_directory_file_count() {
    // Arrange & Act
    let name = DirectoryFileCountRule::default().name();

    // Assert
    assert_eq!(name, "directory-file-count");
}
