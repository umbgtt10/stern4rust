// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The currency every rule reports in. It carries no behaviour beyond
// construction, so what is worth stating is that construction keeps the four
// fields distinct -- a report whose file and rule columns were crossed would
// still print, and still be wrong.

use stern4rust::offence::Offence;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let offence = Offence::new("src/a.rs", 12, "header", "expected something".to_string());

    // Assert
    assert_eq!(offence.file, "src/a.rs");
    assert_eq!(offence.line, 12);
    assert_eq!(offence.rule, "header");
    assert_eq!(offence.description, "expected something");
}

#[test]
fn offences_differing_only_by_line_are_not_equal() {
    // Arrange & Act
    let first = Offence::new("src/a.rs", 1, "header", "same".to_string());
    let second = Offence::new("src/a.rs", 2, "header", "same".to_string());

    // Assert
    assert_ne!(first, second);
}

// Offences are found in whatever order the rules happen to run, which puts the
// tree-wide ones after every per-file one. Grouping by file is what lets a
// reader -- or a tool consuming the report -- fix one file at a time.
#[test]
fn sort_key_groups_offences_by_file_before_line() {
    // Arrange & Act
    let first = Offence::new("src/a.rs", 90, "header", "x".to_string());
    let second = Offence::new("src/b.rs", 2, "header", "x".to_string());

    // Assert
    assert!(first.sort_key() < second.sort_key());
}

#[test]
fn sort_key_orders_two_offences_in_the_same_file_by_line() {
    // Arrange & Act
    let first = Offence::new("src/a.rs", 2, "header", "x".to_string());
    let second = Offence::new("src/a.rs", 10, "header", "x".to_string());

    // Assert
    assert!(first.sort_key() < second.sort_key());
}
