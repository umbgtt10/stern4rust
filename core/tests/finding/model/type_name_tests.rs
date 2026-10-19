// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A type's name as the words it is written in.
//
// Two rules turn a type into the file it would live in, and named-subject also
// compares a file's name with the last words of a type's. Splitting at every
// capital reads an acronym in capitals as one word per letter, so a run of
// capitals is one word, ended by the capital that starts the next one.

use stern4rust::finding::model::type_name::TypeName;

#[test]
fn file_name_joins_the_words_with_underscores() {
    // Arrange & Act
    let file = TypeName::new("ColumnWidths").file_name();

    // Assert
    assert_eq!(file, "column_widths.rs");
}

#[test]
fn words_of_a_name_in_capitals_alone_is_one_word() {
    // Arrange & Act
    let name = TypeName::new("IO");

    // Assert
    assert_eq!(name.words(), ["io"]);
}

// Not the house style, but a name rustc accepts, and the underscore is already
// a word break.
#[test]
fn words_of_a_name_with_an_underscore_split_there() {
    // Arrange & Act
    let name = TypeName::new("Wire_Message");

    // Assert
    assert_eq!(name.words(), ["wire", "message"]);
}

// A digit belongs to the word it follows: `Ipv4Addr` is an Ipv4 address.
#[test]
fn words_of_a_name_with_digits_keep_them_with_the_word_before() {
    // Arrange & Act
    let name = TypeName::new("Ipv4Addr");

    // Assert
    assert_eq!(name.words(), ["ipv4", "addr"]);
}

#[test]
fn words_of_a_pascal_case_name_split_at_each_capital() {
    // Arrange & Act
    let name = TypeName::new("UdpEiListener");

    // Assert
    assert_eq!(name.words(), ["udp", "ei", "listener"]);
}

#[test]
fn words_of_a_run_of_capitals_end_it_before_the_capital_that_starts_a_word() {
    // Arrange & Act
    let name = TypeName::new("HTTPServer");

    // Assert
    assert_eq!(name.words(), ["http", "server"]);
}
