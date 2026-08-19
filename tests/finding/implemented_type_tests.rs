// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One of a file's subjects: a type it both declares and gives behaviour to.
//
// The line is the declaration's rather than the impl block's, because the
// declaration is what a reader moves when the file turns out to have two
// subjects. The suggested file is what turns the correction from a convention
// into a path.

use stern4rust::finding::implemented_type::ImplementedType;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = ImplementedType::new("ColumnWidths", 42);

    // Assert
    assert_eq!(found.name, "ColumnWidths");
    assert_eq!(found.line, 42);
}

#[test]
fn suggested_file_of_a_multi_word_name_separates_the_words() {
    // Arrange & Act
    let file = ImplementedType::new("ColumnWidths", 1).suggested_file();

    // Assert
    assert_eq!(file, "column_widths.rs");
}

// A leading capital starts the name rather than a new word, so the suggestion
// does not open with an underscore.
#[test]
fn suggested_file_of_a_single_letter_name_does_not_lead_with_an_underscore() {
    // Arrange & Act
    let file = ImplementedType::new("A", 1).suggested_file();

    // Assert
    assert_eq!(file, "a.rs");
}

#[test]
fn suggested_file_of_a_single_word_name_is_that_word() {
    // Arrange & Act
    let file = ImplementedType::new("Widget", 1).suggested_file();

    // Assert
    assert_eq!(file, "widget.rs");
}

// An acronym separates letter by letter. The suggestion is a starting point for
// a reader rather than a name to be adopted unread, and this is the one shape
// where it reads worse than what a person would pick.
#[test]
fn suggested_file_of_an_acronym_separates_every_capital() {
    // Arrange & Act
    let file = ImplementedType::new("HttpClient", 1).suggested_file();

    // Assert
    assert_eq!(file, "http_client.rs");
}
