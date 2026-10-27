// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The offences a repository has already agreed to live with.
//
// --rule lets a codebase enforce one rule at a time. What it cannot express is
// "every rule, against new code only", which is what a codebase with six
// hundred existing offences needs -- otherwise the choice is between a gate
// that fails forever and no gate at all.
//
// Counts, not a set: fixing one of two identical offences and introducing
// another must still pass, while introducing a third must not.

use std::collections::BTreeMap;
use stern4rust::reporting::offence::Offence;
use stern4rust::settings::adoption::baseline::Baseline;

fn offence(file: &str, line: usize, rule: &'static str, description: &str) -> Offence {
    Offence::new(file, line, rule, description.to_string(), "fix".to_string())
}

#[test]
fn apply_forgives_an_offence_that_moved_to_a_new_line() {
    // Arrange
    let baseline = Baseline::of(&[offence("src/a.rs", 12, "header", "wrong")]);

    // Act
    let outcome = baseline.apply(vec![offence("src/a.rs", 400, "header", "wrong")]);

    // Assert
    assert!(outcome.kept().is_empty());
    assert_eq!(outcome.suppressed(), 1);
}

#[test]
fn apply_forgives_each_recorded_offence_once() {
    // Arrange
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
    ]);

    // Act
    let outcome = baseline.apply(vec![
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
        offence("src/a.rs", 3, "header", "wrong"),
    ]);

    // Assert
    assert_eq!(outcome.kept().len(), 1);
    assert_eq!(outcome.suppressed(), 2);
}

#[test]
fn apply_of_an_empty_baseline_keeps_everything() {
    // Arrange
    let baseline = Baseline::default();

    // Act
    let outcome = baseline.apply(vec![offence("src/a.rs", 1, "header", "wrong")]);

    // Assert
    assert_eq!(outcome.kept().len(), 1);
    assert_eq!(outcome.suppressed(), 0);
}

#[test]
fn apply_reports_an_offence_the_baseline_never_saw() {
    // Arrange
    let baseline = Baseline::of(&[offence("src/a.rs", 1, "header", "wrong")]);

    // Act
    let outcome = baseline.apply(vec![offence("src/b.rs", 1, "header", "wrong")]);

    // Assert
    assert_eq!(outcome.kept().len(), 1);
    assert_eq!(outcome.kept()[0].file(), "src/b.rs");
    assert_eq!(outcome.suppressed(), 0);
}

// An entry matching nothing describes an offence somebody has since fixed. Dead
// weight that makes the file look like it is still holding something back.
#[test]
fn apply_reports_how_many_entries_matched_nothing() {
    // Arrange
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/gone.rs", 1, "header", "wrong"),
    ]);

    // Act
    let outcome = baseline.apply(vec![offence("src/a.rs", 1, "header", "wrong")]);

    // Assert
    assert_eq!(outcome.stale(), 1);
    assert!(outcome.is_stale());
}

#[test]
fn counts_names_each_fingerprint_with_how_often_it_was_recorded() {
    // Arrange
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
    ]);

    // Act
    let counts = baseline.counts();

    // Assert
    assert_eq!(counts.values().copied().collect::<Vec<_>>(), [2]);
}

#[test]
fn len_counts_every_recorded_occurrence() {
    // Arrange & Act
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
        offence("src/b.rs", 1, "header", "wrong"),
    ]);

    // Assert
    assert_eq!(baseline.len(), 3);
    assert!(!baseline.is_empty());
}

#[test]
fn new_holds_the_counts_it_was_given() {
    // Arrange
    let counts = BTreeMap::from([(String::from("src/a.rs\u{1f}header\u{1f}wrong"), 2)]);

    // Act
    let baseline = Baseline::new(counts.clone());

    // Assert
    assert_eq!(baseline.counts(), &counts);
    assert_eq!(baseline.len(), 2);
}
