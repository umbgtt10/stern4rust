// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which rules ran against one package, and which did not.
//
// The whole point is the comparison. A workspace whose members all answer to the
// same rules should read exactly as a single package does -- one roster, said
// once -- and only a workspace whose members genuinely differ should cost the
// reader a block each. So the question this type has to answer is not "what did
// this package do" but "did this package do the same as that one".

use stern4rust::reporting::package_roster::PackageRoster;

fn roster(name: &str, applied: &[&str], skipped: &[&str]) -> PackageRoster {
    PackageRoster::new(
        name,
        applied.iter().map(|rule| (*rule).to_string()).collect(),
        skipped.iter().map(|rule| (*rule).to_string()).collect(),
        Vec::new(),
    )
}

// The absences, said the way the report says them, so a reader sees why a rule
// is missing and not merely that it is.
#[test]
fn absences_names_a_skipped_rule_and_an_unconfigured_one_apart() {
    // Arrange
    let roster = PackageRoster::new(
        "system-tests",
        vec!["header".to_string()],
        vec!["paired-test-file".to_string()],
        vec![(
            "spdx-matches-manifest".to_string(),
            "needs a `license` field in Cargo.toml".to_string(),
        )],
    );

    // Act
    let absences = roster.absences();

    // Assert
    assert_eq!(
        absences,
        vec![
            "paired-test-file (skipped)".to_string(),
            "spdx-matches-manifest (needs a `license` field in Cargo.toml)".to_string(),
        ]
    );
}

#[test]
fn absences_of_a_roster_that_ran_everything_is_empty() {
    // Arrange
    let roster = roster("node", &["header", "test-naming"], &[]);

    // Act & Assert
    assert!(roster.absences().is_empty());
}

#[test]
fn agrees_with_a_roster_applying_something_else_is_false() {
    // Arrange
    let left = roster("node", &["header"], &[]);
    let right = roster("node-infra", &["header", "test-naming"], &[]);

    // Act & Assert
    assert!(!left.agrees_with(&right));
}

// Two packages running the same rules are one thing to report, and the name is
// the only thing that differs -- so the name is what the comparison ignores.
#[test]
fn agrees_with_a_roster_running_the_same_rules_is_true() {
    // Arrange
    let left = roster("node", &["header", "test-naming"], &["paired-test-file"]);
    let right = roster(
        "node-infra",
        &["header", "test-naming"],
        &["paired-test-file"],
    );

    // Act & Assert
    assert!(left.agrees_with(&right));
}

#[test]
fn agrees_with_a_roster_skipping_something_else_is_false() {
    // Arrange
    let left = roster("validation", &["header"], &["paired-test-file"]);
    let right = roster("system-tests", &["header"], &["test-naming"]);

    // Act & Assert
    assert!(!left.agrees_with(&right));
}

// A rule that could not run is an absence like any other, and two packages
// differing only in *why* a rule is missing still differ.
#[test]
fn agrees_with_a_roster_whose_rule_is_unconfigured_rather_than_skipped_is_false() {
    // Arrange
    let left = roster("node", &["header"], &["spdx-matches-manifest"]);
    let right = PackageRoster::new(
        "node-infra",
        vec!["header".to_string()],
        Vec::new(),
        vec![(
            "spdx-matches-manifest".to_string(),
            "needs a `license` field in Cargo.toml".to_string(),
        )],
    );

    // Act & Assert
    assert!(!left.agrees_with(&right));
}

#[test]
fn new_keeps_the_package_it_is_about() {
    // Arrange & Act
    let roster = roster("node", &["header"], &[]);

    // Assert
    assert_eq!(roster.package, "node");
}
