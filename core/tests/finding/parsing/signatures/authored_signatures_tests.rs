// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The signatures a file's author can change: free functions, inherent methods
// and trait declarations, inline modules included. A trait impl's signatures are
// the trait's, so they are left out -- the rules that judge a signature report it
// at the declaration instead.

use stern4rust::finding::parsing::signatures::authored_signatures::AuthoredSignatures;
use syn::Item;
use syn::parse_file;

fn items(source: &str) -> Vec<Item> {
    parse_file(source).expect("parses").items
}

#[test]
fn of_a_free_function_returns_its_signature() {
    // Arrange
    let parsed = items("pub fn run(count: u32) -> u32 {\n    count\n}\n");

    // Act
    let signatures = AuthoredSignatures::of(&parsed);

    // Assert
    let names: Vec<String> = signatures.iter().map(|sig| sig.ident.to_string()).collect();
    assert_eq!(names, ["run"]);
}

#[test]
fn of_a_trait_declaration_returns_its_methods() {
    // Arrange
    let parsed =
        items("pub trait Store {\n    fn get(&self) -> u32;\n    fn put(&self, value: u32);\n}\n");

    // Act
    let signatures = AuthoredSignatures::of(&parsed);

    // Assert
    let names: Vec<String> = signatures.iter().map(|sig| sig.ident.to_string()).collect();
    assert_eq!(names, ["get", "put"]);
}

#[test]
fn of_a_trait_impl_returns_nothing() {
    // Arrange
    let parsed = items(
        "struct Meter;\n\nimpl Default for Meter {\n    fn default() -> Self {\n        Meter\n    }\n}\n",
    );

    // Act
    let signatures = AuthoredSignatures::of(&parsed);

    // Assert
    assert!(signatures.is_empty());
}

#[test]
fn of_an_inherent_impl_returns_its_methods() {
    // Arrange
    let parsed = items(
        "struct Meter;\n\nimpl Meter {\n    pub fn new() -> Self {\n        Meter\n    }\n\n    \
         fn total(&self) -> u32 {\n        0\n    }\n}\n",
    );

    // Act
    let signatures = AuthoredSignatures::of(&parsed);

    // Assert
    let names: Vec<String> = signatures.iter().map(|sig| sig.ident.to_string()).collect();
    assert_eq!(names, ["new", "total"]);
}

#[test]
fn of_an_inline_module_returns_the_functions_it_holds() {
    // Arrange
    let parsed = items("mod inner {\n    pub fn helper() {}\n}\n");

    // Act
    let signatures = AuthoredSignatures::of(&parsed);

    // Assert
    let names: Vec<String> = signatures.iter().map(|sig| sig.ident.to_string()).collect();
    assert_eq!(names, ["helper"]);
}

#[test]
fn of_items_without_functions_returns_nothing() {
    // Arrange
    let parsed = items("pub struct Meter;\n\npub const LIMIT: u32 = 3;\n");

    // Act
    let signatures = AuthoredSignatures::of(&parsed);

    // Assert
    assert!(signatures.is_empty());
}
