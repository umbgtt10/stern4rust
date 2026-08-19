// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What the exclusions did, as well as what survived them.
//
// The count is carried per pattern rather than as one total, because a pattern
// that matched nothing is the case a total would hide -- and a stale exclusion
// silencing a tree that no longer exists is indistinguishable, from the report
// alone, from one doing real work.

use std::path::PathBuf;
use stern4rust::adoption::exclusion_outcome::ExclusionOutcome;

fn outcome(excluded: &[(&str, usize)]) -> ExclusionOutcome {
    ExclusionOutcome::new(
        Vec::new(),
        excluded
            .iter()
            .map(|(pattern, count)| ((*pattern).to_string(), *count))
            .collect(),
    )
}

#[test]
fn excluded_total_of_no_patterns_is_zero() {
    // Arrange & Act
    let total = outcome(&[]).excluded_total();

    // Assert
    assert_eq!(total, 0);
}

#[test]
fn excluded_total_sums_every_pattern() {
    // Arrange & Act
    let total = outcome(&[("a/**", 2), ("b/**", 3)]).excluded_total();

    // Assert
    assert_eq!(total, 5);
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = ExclusionOutcome::new(vec![PathBuf::from("src/a.rs")], vec![("x/**".into(), 1)]);

    // Assert
    assert_eq!(found.kept, [PathBuf::from("src/a.rs")]);
    assert_eq!(found.excluded, [("x/**".to_string(), 1)]);
}

#[test]
fn unmatched_patterns_lists_only_those_that_covered_nothing() {
    // Arrange
    let built = outcome(&[("live/**", 4), ("dead/**", 0), ("also_dead/**", 0)]);

    // Act
    let unmatched = built.unmatched_patterns();

    // Assert
    assert_eq!(unmatched, ["dead/**", "also_dead/**"]);
}

#[test]
fn unmatched_patterns_with_every_pattern_matching_is_empty() {
    // Arrange
    let built = outcome(&[("a/**", 1)]);

    // Act
    let unmatched = built.unmatched_patterns();

    // Assert
    assert!(unmatched.is_empty());
}
