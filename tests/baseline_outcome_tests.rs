// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a baseline let through, and what it hid.
//
// The suppressed count travels with the surviving offences because a run
// reporting nothing while hiding four hundred findings would be the most
// comfortable lie this tool could tell.

use stern4rust::baseline_outcome::BaselineOutcome;
use stern4rust::offence::Offence;

fn offence() -> Offence {
    Offence::new(
        "src/a.rs",
        1,
        "header",
        "wrong".to_string(),
        "fix".to_string(),
    )
}

#[test]
fn is_stale_with_a_dead_entry_is_true() {
    // Arrange & Act
    let outcome = BaselineOutcome::new(Vec::new(), 0, 2);

    // Assert
    assert!(outcome.is_stale());
}

#[test]
fn is_stale_with_no_dead_entries_is_false() {
    // Arrange & Act
    let outcome = BaselineOutcome::new(Vec::new(), 7, 0);

    // Assert
    assert!(!outcome.is_stale());
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let outcome = BaselineOutcome::new(vec![offence()], 3, 1);

    // Assert
    assert_eq!(outcome.kept.len(), 1);
    assert_eq!(outcome.suppressed, 3);
    assert_eq!(outcome.stale, 1);
}
