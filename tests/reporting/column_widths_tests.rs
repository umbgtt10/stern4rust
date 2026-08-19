// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// How wide each column of the table has to be.
//
// Two properties matter and pull in opposite directions: a column must be wide
// enough for its widest entry, so nothing overflows, and never narrower than its
// own heading, so the header row still lines up when every offence is short.

use stern4rust::reporting::column_widths::ColumnWidths;
use stern4rust::reporting::offence::Offence;

fn offence(file: &str, line: usize, rule: &'static str, description: &str) -> Offence {
    Offence::new(file, line, rule, description.to_string(), "fix".to_string())
}

#[test]
fn of_no_offences_falls_back_to_the_heading_widths() {
    // Arrange & Act
    let widths = ColumnWidths::of(&[]);

    // Assert
    assert_eq!(widths.file, "file".len());
    assert_eq!(widths.line, "line".len());
    assert_eq!(widths.rule, "rule".len());
    assert_eq!(widths.description, "offence".len());
}

// Never narrower than the heading, however short every entry is.
#[test]
fn of_offences_narrower_than_their_headings_keeps_the_heading_width() {
    // Arrange & Act
    let widths = ColumnWidths::of(&[offence("a.rs", 1, "x", "y")]);

    // Assert
    assert_eq!(widths.line, "line".len());
    assert_eq!(widths.rule, "rule".len());
    assert_eq!(widths.description, "offence".len());
}

// A path longer than its heading must widen the column rather than overflow it.
#[test]
fn of_offences_takes_the_widest_entry_of_each_column() {
    // Arrange
    let long = "crates/deeply/nested/package/src/module/subject.rs";

    // Act
    let widths = ColumnWidths::of(&[
        offence("src/a.rs", 1, "header", "short"),
        offence(
            long,
            1234,
            "test-file-structure",
            "a much longer description",
        ),
    ]);

    // Assert
    assert_eq!(widths.file, long.len());
    assert_eq!(widths.line, "1234".len());
    assert_eq!(widths.rule, "test-file-structure".len());
    assert_eq!(widths.description, "a much longer description".len());
}
