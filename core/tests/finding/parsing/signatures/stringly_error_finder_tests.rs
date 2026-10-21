// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The first `Result` in a type whose error is a sentence rather than a kind: a
// `String`, a `&str`, or a boxed `dyn ...Error`. Wherever the `Result` sits, and
// written as it is written.

use stern4rust::finding::parsing::signatures::stringly_error_finder::StringlyErrorFinder;
use syn::Type;
use syn::parse_str;

fn first_in(source: &str) -> Option<String> {
    StringlyErrorFinder::first_in(&parse_str::<Type>(source).expect("parses as a type"))
}

#[test]
fn first_in_a_boxed_trait_object_that_is_not_an_error_finds_nothing() {
    // Arrange & Act
    let found = first_in("Result<u8, Box<dyn Display>>");

    // Assert
    assert_eq!(found, None);
}

// An alias with one argument names its error elsewhere -- `io::Result<u8>` is
// `io::Error` -- so there is nothing here to judge.
#[test]
fn first_in_a_result_alias_with_one_argument_finds_nothing() {
    // Arrange & Act
    let found = first_in("io::Result<u8>");

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_a_result_of_a_borrowed_str_finds_it_without_its_lifetime() {
    // Arrange & Act
    let found = first_in("Result<u8, &'a str>");

    // Assert
    assert_eq!(found.as_deref(), Some("&str"));
}

#[test]
fn first_in_a_result_of_a_boxed_error_finds_it() {
    // Arrange & Act
    let found = first_in("Result<(), Box<dyn Error>>");

    // Assert
    assert_eq!(found.as_deref(), Some("Box<dyn Error>"));
}

#[test]
fn first_in_a_result_of_a_boxed_error_with_bounds_finds_it() {
    // Arrange & Act
    let found = first_in("Result<(), Box<dyn std::error::Error + Send + Sync>>");

    // Assert
    assert_eq!(
        found.as_deref(),
        Some("Box<dyn std::error::Error + Send + Sync>")
    );
}

#[test]
fn first_in_a_result_of_a_string_finds_it() {
    // Arrange & Act
    let found = first_in("Result<u8, String>");

    // Assert
    assert_eq!(found.as_deref(), Some("String"));
}

#[test]
fn first_in_a_result_of_a_typed_error_finds_nothing() {
    // Arrange & Act
    let found = first_in("Result<u8, ReadError>");

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_a_result_wrapped_in_an_option_finds_it() {
    // Arrange & Act
    let found = first_in("Option<Result<u8, String>>");

    // Assert
    assert_eq!(found.as_deref(), Some("String"));
}

// A string returned is a value, not a failure.
#[test]
fn first_in_a_string_outside_any_result_finds_nothing() {
    // Arrange & Act
    let found = first_in("Vec<String>");

    // Assert
    assert_eq!(found, None);
}
