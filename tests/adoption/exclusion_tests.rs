// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One `--exclude` pattern, matched against the package-relative path.
//
// The path is package-relative on purpose: a pattern that had to know where the
// repository sits on disk could not be written down in a config file and
// checked in, which is the whole point of having one.
//
// An unusable pattern is an error rather than a pattern that matches nothing.
// Matching nothing is a legitimate result -- a tree that has since been deleted
// -- and the two must not be reported the same way.

use std::path::Path;
use stern4rust::adoption::exclusion::Exclusion;

fn exclusion(pattern: &str) -> Exclusion {
    Exclusion::new(pattern).expect("valid pattern")
}

#[test]
fn matches_a_path_below_an_excluded_directory_is_true() {
    // Arrange & Act
    let matched = exclusion("fixture/**").matches(Path::new("fixture/deep/nested/a.rs"));

    // Assert
    assert!(matched);
}

#[test]
fn matches_a_path_outside_the_pattern_is_false() {
    // Arrange & Act
    let matched = exclusion("fixture/**").matches(Path::new("src/a.rs"));

    // Assert
    assert!(!matched);
}

// A prefix match is not enough: `fixtures_of_mine/` is not `fixture/`.
#[test]
fn matches_a_path_whose_directory_merely_starts_with_the_pattern_is_false() {
    // Arrange & Act
    let matched = exclusion("fixture/**").matches(Path::new("fixtures_of_mine/a.rs"));

    // Assert
    assert!(!matched);
}

#[test]
fn matches_a_wildcard_on_the_file_name_is_true() {
    // Arrange & Act
    let matched = exclusion("**/*_generated.rs").matches(Path::new("src/deep/thing_generated.rs"));

    // Assert
    assert!(matched);
}

// Windows hands the walker backslashes; a pattern written with forward slashes
// has to match all the same, or an exclusion checked in on one machine would
// silently stop working on another.
#[test]
fn matches_a_windows_separator_path_is_true() {
    // Arrange & Act
    let matched = exclusion("fixture/**").matches(Path::new(r"fixture\deep\a.rs"));

    // Assert
    assert!(matched);
}

#[test]
fn new_of_an_unusable_pattern_is_an_error() {
    // Arrange & Act
    let built = Exclusion::new("fixture/[");

    // Assert
    assert!(built.is_err());
}

#[test]
fn pattern_is_the_text_it_was_built_with() {
    // Arrange & Act
    let pattern = exclusion("fixture/**").pattern().to_string();

    // Assert
    assert_eq!(pattern, "fixture/**");
}
