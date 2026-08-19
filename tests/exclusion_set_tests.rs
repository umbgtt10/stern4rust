// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every `--exclude` pattern this run was given, applied to the walked paths.
//
// The property worth stating is that exclusion is **counted**, not merely done.
// A tree removed from the report without a number beside it is the silent skip
// the walker had until 0.4.0, and an exclusion nobody can count is worse than
// the offences it hides.
//
// Patterns are matched against the package-relative path, so the walker's
// absolute paths have to be reduced against the root before anything is judged.

use std::path::Path;
use std::path::PathBuf;
use stern4rust::exclusion_set::ExclusionSet;

const ROOT: &str = "/repo";

fn paths(relatives: &[&str]) -> Vec<PathBuf> {
    relatives
        .iter()
        .map(|relative| Path::new(ROOT).join(relative))
        .collect()
}

fn set(patterns: &[&str]) -> ExclusionSet {
    let owned: Vec<String> = patterns.iter().map(|p| (*p).to_string()).collect();
    ExclusionSet::new(&owned).expect("valid patterns")
}

#[test]
fn apply_attributes_a_doubly_covered_path_to_the_first_pattern_only() {
    // Arrange
    let set = set(&["fixture/**", "**/*.rs"]);

    // Act
    let outcome = set.apply(paths(&["fixture/a.rs"]), Path::new(ROOT));

    // Assert
    assert_eq!(outcome.excluded_total(), 1);
    assert_eq!(outcome.excluded[0], ("fixture/**".to_string(), 1));
    assert_eq!(outcome.excluded[1], ("**/*.rs".to_string(), 0));
}

#[test]
fn apply_counts_what_each_pattern_removed() {
    // Arrange
    let set = set(&["fixture/**", "vendor/**"]);

    // Act
    let outcome = set.apply(
        paths(&[
            "fixture/a.rs",
            "fixture/deep/b.rs",
            "vendor/c.rs",
            "src/d.rs",
        ]),
        Path::new(ROOT),
    );

    // Assert
    assert_eq!(outcome.excluded[0], ("fixture/**".to_string(), 2));
    assert_eq!(outcome.excluded[1], ("vendor/**".to_string(), 1));
    assert_eq!(outcome.excluded_total(), 3);
}

#[test]
fn apply_keeps_a_path_no_pattern_covers() {
    // Arrange
    let set = set(&["fixture/**"]);

    // Act
    let outcome = set.apply(paths(&["fixture/a.rs", "src/b.rs"]), Path::new(ROOT));

    // Assert
    assert_eq!(outcome.kept, paths(&["src/b.rs"]));
}

// The case a total would hide: a pattern naming a tree that no longer exists
// still looks like it is doing something.
#[test]
fn apply_reports_a_pattern_that_matched_nothing() {
    // Arrange
    let set = set(&["deleted_tree/**"]);

    // Act
    let outcome = set.apply(paths(&["src/a.rs"]), Path::new(ROOT));

    // Assert
    assert_eq!(outcome.unmatched_patterns(), ["deleted_tree/**"]);
    assert_eq!(outcome.excluded_total(), 0);
}

#[test]
fn apply_with_no_patterns_keeps_everything() {
    // Arrange
    let set = set(&[]);

    // Act
    let outcome = set.apply(paths(&["src/a.rs", "tests/b.rs"]), Path::new(ROOT));

    // Assert
    assert_eq!(outcome.kept.len(), 2);
    assert!(outcome.excluded.is_empty());
}

#[test]
fn is_empty_with_no_patterns_is_true() {
    // Arrange & Act
    let empty = set(&[]).is_empty();

    // Assert
    assert!(empty);
}

#[test]
fn new_of_an_unusable_pattern_is_an_error() {
    // Arrange & Act
    let built = ExclusionSet::new(&["fixture/[".to_string()]);

    // Assert
    assert!(built.is_err());
}
