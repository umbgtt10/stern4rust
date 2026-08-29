// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The four groups and the order they appear in. The ordering is derived rather
// than written out, so what is worth pinning is that the derivation says what
// the rule needs it to say: imports first, tests last.

use stern4rust::finding::model::section::Section;

// Helpers and tests are multi-line bodies, where the gap is what marks where
// one ends and the next begins.
#[test]
fn blank_lines_between_entries_of_helpers_and_tests_is_one() {
    // Arrange & Act & Assert
    assert_eq!(Section::Helpers.blank_lines_between_entries(), 1);
    assert_eq!(Section::Tests.blank_lines_between_entries(), 1);
}

// The two groups written without gaps. Both are lists of one-line declarations
// read by scanning down a column, so a blank line between each entry doubles
// the height of the list and tells the reader nothing.
#[test]
fn blank_lines_between_entries_of_imports_and_constants_is_none() {
    // Arrange & Act & Assert
    assert_eq!(Section::Imports.blank_lines_between_entries(), 0);
    assert_eq!(Section::Constants.blank_lines_between_entries(), 0);
}

// The label reads as the thing itself, so an offence says "a constant follows a
// helper" rather than naming an enum variant at the reader.
#[test]
fn label_reads_as_the_thing_itself() {
    // Arrange & Act & Assert
    assert_eq!(Section::Imports.label(), "import");
    assert_eq!(Section::Constants.label(), "constant");
    assert_eq!(Section::Helpers.label(), "helper");
    assert_eq!(Section::Tests.label(), "test");
}

#[test]
fn ordering_places_constants_before_helpers() {
    // Arrange & Act
    let constants = Section::Constants;

    // Assert
    assert!(constants < Section::Helpers);
}

#[test]
fn ordering_places_imports_before_every_other_section() {
    // Arrange & Act
    let imports = Section::Imports;

    // Assert
    assert!(imports < Section::Constants);
    assert!(imports < Section::Helpers);
    assert!(imports < Section::Tests);
}

#[test]
fn ordering_places_tests_after_every_other_section() {
    // Arrange & Act
    let tests = Section::Tests;

    // Assert
    assert!(tests > Section::Imports);
    assert!(tests > Section::Constants);
    assert!(tests > Section::Helpers);
}
