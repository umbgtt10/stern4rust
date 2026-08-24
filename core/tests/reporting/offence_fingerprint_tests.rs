// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What makes two offences the same offence across two runs.
//
// The line is deliberately not part of it. An offence that moved because
// somebody added an import above it is the same offence, and a baseline keyed
// on the line would go stale on the first unrelated edit -- useless exactly
// when it is most needed, on a codebase under active change.

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::offence_fingerprint::OffenceFingerprint;

fn offence(file: &str, line: usize, rule: &'static str, description: &str) -> Offence {
    Offence::new(file, line, rule, description.to_string(), "fix".to_string())
}

// The property the whole baseline rests on.
#[test]
fn of_an_offence_that_moved_lines_is_unchanged() {
    // Arrange
    let before = offence("src/a.rs", 12, "header", "wrong");
    let after = offence("src/a.rs", 340, "header", "wrong");

    // Act
    let moved = OffenceFingerprint::of(&after);

    // Assert
    assert_eq!(moved, OffenceFingerprint::of(&before));
}

// The property this rests on: a rule may reword an offence without every
// baseline in the family reporting it as new. The subject is what the offence is
// about, and it survives a rewrite of the sentence around it.
#[test]
fn of_an_offence_whose_description_was_reworded_is_unchanged() {
    // Arrange
    let before = offence(
        "src/a.rs",
        1,
        "single-implemented-type",
        "`W` is a second type",
    )
    .with_subject("W");
    let after = offence(
        "src/a.rs",
        1,
        "single-implemented-type",
        "`W` is a second type with an impl block; this file's subject is already `V`",
    )
    .with_subject("W");

    // Act
    let reworded = OffenceFingerprint::of(&after);

    // Assert
    assert_eq!(reworded, OffenceFingerprint::of(&before));
}

#[test]
fn of_offences_differing_by_file_differ() {
    // Arrange
    let left = offence("src/a.rs", 1, "header", "wrong");
    let right = offence("src/b.rs", 1, "header", "wrong");

    // Act
    let fingerprint = OffenceFingerprint::of(&left);

    // Assert
    assert_ne!(fingerprint, OffenceFingerprint::of(&right));
}

#[test]
fn of_offences_differing_by_rule_differ() {
    // Arrange
    let left = offence("src/a.rs", 1, "header", "wrong");
    let right = offence("src/a.rs", 1, "tests-layout", "wrong");

    // Act
    let fingerprint = OffenceFingerprint::of(&left);

    // Assert
    assert_ne!(fingerprint, OffenceFingerprint::of(&right));
}

// Two findings of one rule in one file stay distinct, which is what the
// description was standing in for before the subject existed.
#[test]
fn of_offences_differing_by_subject_differ() {
    // Arrange
    let left = offence("src/a.rs", 1, "single-implemented-type", "same").with_subject("W");
    let right = offence("src/a.rs", 1, "single-implemented-type", "same").with_subject("V");

    // Act
    let fingerprint = OffenceFingerprint::of(&left);

    // Assert
    assert_ne!(fingerprint, OffenceFingerprint::of(&right));
}

#[test]
fn of_offences_with_no_subject_differing_by_description_differ() {
    // Arrange
    let left = offence("src/a.rs", 1, "header", "wrong");
    let right = offence("src/a.rs", 1, "header", "also wrong");

    // Act
    let fingerprint = OffenceFingerprint::of(&left);

    // Assert
    assert_ne!(fingerprint, OffenceFingerprint::of(&right));
}

// A separator that cannot occur in a path, a rule name or a description, so
// two fields cannot be run together into a third meaning.
#[test]
fn of_two_offences_cannot_collide_across_field_boundaries() {
    // Arrange
    let left = offence("a", 1, "b", "c");
    let right = offence("a\u{1f}b", 1, "b", "c");

    // Act
    let fingerprint = OffenceFingerprint::of(&left);

    // Assert
    assert_ne!(fingerprint, OffenceFingerprint::of(&right));
}
