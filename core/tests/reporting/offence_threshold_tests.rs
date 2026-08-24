// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// How much of the report is printed, when a first run against a large codebase
// can find a thousand offences.
//
// The cap is on what is shown and never on what is counted. Truncating the
// summary as well would produce a report that says a codebase has a hundred
// problems when it has a thousand -- the precise shape of quiet wrongness this
// tool exists to catch, and it would be this tool doing it.

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::offence_threshold::OffenceThreshold;

fn offences(count: usize) -> Vec<Offence> {
    (1..=count)
        .map(|line| {
            Offence::new(
                "src/a.rs",
                line,
                "header",
                "wrong".to_string(),
                "fix it".to_string(),
            )
        })
        .collect()
}

#[test]
fn default_is_one_hundred() {
    // Arrange & Act
    let threshold = OffenceThreshold::default();

    // Assert
    assert_eq!(threshold.limit(), 100);
}

#[test]
fn is_unlimited_of_a_positive_limit_is_false() {
    // Arrange & Act
    let threshold = OffenceThreshold::new(1);

    // Assert
    assert!(!threshold.is_unlimited());
}

// Zero is the escape hatch rather than a way to silence the report entirely --
// a limit of nothing would be a tool that finds problems and refuses to say
// which.
#[test]
fn is_unlimited_of_zero_is_true() {
    // Arrange & Act
    let threshold = OffenceThreshold::new(0);

    // Assert
    assert!(threshold.is_unlimited());
}

#[test]
fn kept_of_fewer_offences_than_the_limit_returns_all_of_them() {
    // Arrange
    let found = offences(3);

    // Act
    let kept = OffenceThreshold::new(10).kept(&found);

    // Assert
    assert_eq!(kept.len(), 3);
}

#[test]
fn kept_of_more_offences_than_the_limit_returns_the_limit() {
    // Arrange
    let found = offences(7);

    // Act
    let kept = OffenceThreshold::new(2).kept(&found);

    // Assert
    assert_eq!(kept.len(), 2);
}

// The offences are already sorted by file then line, so the ones kept are whole
// files from the top rather than a scattering across the tree. A reader fixes
// what is shown, re-runs, and gets the next file.
#[test]
fn kept_returns_the_offences_in_the_order_it_was_given() {
    // Arrange
    let found = offences(7);

    // Act
    let kept = OffenceThreshold::new(2).kept(&found);

    // Assert
    assert_eq!(kept[0].line, 1);
    assert_eq!(kept[1].line, 2);
}

#[test]
fn kept_with_no_limit_returns_all_of_them() {
    // Arrange
    let found = offences(500);

    // Act
    let kept = OffenceThreshold::new(0).kept(&found);

    // Assert
    assert_eq!(kept.len(), 500);
}

#[test]
fn omitted_of_fewer_offences_than_the_limit_is_none() {
    // Arrange & Act
    let omitted = OffenceThreshold::new(10).omitted(&offences(3));

    // Assert
    assert_eq!(omitted, 0);
}

#[test]
fn omitted_of_more_offences_than_the_limit_is_the_remainder() {
    // Arrange & Act
    let omitted = OffenceThreshold::new(2).omitted(&offences(7));

    // Assert
    assert_eq!(omitted, 5);
}

#[test]
fn omitted_with_no_limit_is_none() {
    // Arrange & Act
    let omitted = OffenceThreshold::new(0).omitted(&offences(500));

    // Assert
    assert_eq!(omitted, 0);
}
