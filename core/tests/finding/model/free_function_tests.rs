// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A function that belongs to no impl block, where it is declared, and the names
// it uses -- which is what tells a helper of the file's subject from a stranger.

use std::collections::BTreeSet;
use stern4rust::finding::model::free_function::FreeFunction;

fn uses(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| name.to_string()).collect()
}

#[test]
fn names_an_identifier_it_does_not_use_is_false() {
    // Arrange
    let found = FreeFunction::new(String::from("parse"), 3, uses(&["Option", "Widget"]));

    // Act
    let named = found.names("Gadget");

    // Assert
    assert!(!named);
}

#[test]
fn names_an_identifier_it_uses_is_true() {
    // Arrange
    let found = FreeFunction::new(String::from("parse"), 3, uses(&["Option", "Widget"]));

    // Act
    let named = found.names("Widget");

    // Assert
    assert!(named);
}

#[test]
fn new_keeps_its_name_and_line() {
    // Arrange & Act
    let found = FreeFunction::new(String::from("parse"), 3, BTreeSet::new());

    // Assert
    assert_eq!(found.name(), "parse");
    assert_eq!(found.line(), 3);
}
