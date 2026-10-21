// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A type as it is written: lifetimes dropped, and a space only between two words,
// after a separator and around a `+`, so a token stream's spacing never shows.

use stern4rust::finding::parsing::signatures::type_text::TypeText;
use syn::Type;
use syn::parse_str;

fn text(source: &str) -> String {
    TypeText::of(&parse_str::<Type>(source).expect("parses as a type"))
}

#[test]
fn of_a_reference_drops_its_lifetime() {
    // Arrange & Act
    let written = text("&'a mut Vec<u8>");

    // Assert
    assert_eq!(written, "&mut Vec<u8>");
}

#[test]
fn of_a_trait_object_spaces_its_bounds() {
    // Arrange & Act
    let written = text("Box<dyn std::error::Error + Send + Sync>");

    // Assert
    assert_eq!(written, "Box<dyn std::error::Error + Send + Sync>");
}

#[test]
fn of_a_tuple_spaces_after_each_comma() {
    // Arrange & Act
    let written = text("(u64,u64)");

    // Assert
    assert_eq!(written, "(u64, u64)");
}
