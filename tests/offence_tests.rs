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
    let offence = Offence::new(
        "src/a.rs",
        12,
        "header",
        "expected something".to_string(),
        "fix it like this".to_string(),
    );

    // Assert
    assert_eq!(offence.file, "src/a.rs");
    assert_eq!(offence.line, 12);
    assert_eq!(offence.rule, "header");
    assert_eq!(offence.description, "expected something");
    assert_eq!(offence.correction, "fix it like this");
}

// A rule opts into the machine-readable extras, so every existing construction
// site keeps working and a rule that has nothing precise to add says nothing.
#[test]
fn new_leaves_the_machine_readable_fields_empty() {
    // Arrange & Act
    let offence = Offence::new(
        "src/a.rs",
        12,
        "header",
        "expected something".to_string(),
        "fix".to_string(),
    );

    // Assert
    assert_eq!(offence.subject, None);
    assert_eq!(offence.expected, None);
}

#[test]
fn offences_differing_only_by_line_are_not_equal() {
    // Arrange & Act
    let first = Offence::new(
        "src/a.rs",
        1,
        "header",
        "same".to_string(),
        "fix".to_string(),
    );
    let second = Offence::new(
        "src/a.rs",
        2,
        "header",
        "same".to_string(),
        "fix".to_string(),
    );

    // Assert
    assert_ne!(first, second);
}

// Offences are found in whatever order the rules happen to run, which puts the
// tree-wide ones after every per-file one. Grouping by file is what lets a
// reader -- or a tool consuming the report -- fix one file at a time.
#[test]
fn sort_key_groups_offences_by_file_before_line() {
    // Arrange & Act
    let first = Offence::new("src/a.rs", 90, "header", "x".to_string(), "fix".to_string());
    let second = Offence::new("src/b.rs", 2, "header", "x".to_string(), "fix".to_string());

    // Assert
    assert!(first.sort_key() < second.sort_key());
}

#[test]
fn sort_key_orders_two_offences_in_the_same_file_by_line() {
    // Arrange & Act
    let first = Offence::new("src/a.rs", 2, "header", "x".to_string(), "fix".to_string());
    let second = Offence::new("src/a.rs", 10, "header", "x".to_string(), "fix".to_string());

    // Assert
    assert!(first.sort_key() < second.sort_key());
}

// The correct text, where the rule knows it. This is what turns a report into
// something that can be applied rather than only read: the header rule knows the
// whole header, so it can hand it over instead of describing one wrong line.
#[test]
fn with_expected_attaches_the_text_the_rule_knows_is_correct() {
    // Arrange & Act
    let offence = Offence::new("src/a.rs", 1, "header", "x".to_string(), "fix".to_string())
        .with_expected("// Copyright\n// MIT");

    // Assert
    assert_eq!(offence.expected, Some("// Copyright\n// MIT".to_string()));
}

#[test]
fn with_subject_names_the_thing_the_offence_is_about() {
    // Arrange & Act
    let offence = Offence::new("src/a.rs", 1, "header", "x".to_string(), "fix".to_string())
        .with_subject("the constant `LIMIT`");

    // Assert
    assert_eq!(offence.subject, Some("the constant `LIMIT`".to_string()));
}
