// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The counterweight to directory-file-count. That rule creates folders; this
// one stops the creating from being the answer to everything, because a
// directory with twenty subfolders is exactly as unreadable as one with a
// hundred files and looks tidier while being worse.
//
// Checked at every level, so pushing the sprawl one directory down does not
// escape it.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::directory_subfolder_count_rule::DirectorySubfolderCountRule;
use stern4rust::source_file::SourceFile;

fn check(limit: usize, paths: &[&str]) -> Vec<Offence> {
    let files: Vec<SourceFile> = paths
        .iter()
        .map(|path| SourceFile::new(path, "pub struct A;\n"))
        .collect();
    DirectorySubfolderCountRule::new(limit).check_workspace(&files)
}

fn check_folders(limit: usize, parent: &str, count: usize) -> Vec<Offence> {
    let owned = folders(parent, count);
    let paths: Vec<&str> = owned.iter().map(String::as_str).collect();
    check(limit, &paths)
}

fn folders(parent: &str, count: usize) -> Vec<String> {
    (0..count).map(|n| format!("{parent}/d{n}/f.rs")).collect()
}

#[test]
fn check_workspace_at_the_limit_reports_nothing() {
    // Arrange & Act
    let found = check_folders(3, "src", 3);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// Pushing the sprawl one level down must not escape the rule.
#[test]
fn check_workspace_counts_at_every_level() {
    // Arrange & Act
    let found = check_folders(2, "src/deep", 3);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].subject.as_deref(), Some("src/deep"));
}

// A folder holding no .rs file is invisible to the walk and is nobody's module.
#[test]
fn check_workspace_does_not_count_a_folder_with_no_source() {
    // Arrange & Act
    let found = check(1, &["src/a/f.rs", "src/lib.rs"]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn check_workspace_over_the_limit_reports_the_directory() {
    // Arrange & Act
    let found = check_folders(2, "src", 3);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "directory-subfolder-count");
    assert_eq!(found[0].subject.as_deref(), Some("src"));
    assert!(found[0].description.contains("holds 3 subfolders"));
}

#[test]
fn name_is_directory_subfolder_count() {
    // Arrange & Act
    let name = DirectorySubfolderCountRule::default().name();

    // Assert
    assert_eq!(name, "directory-subfolder-count");
}
