// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Everything a source file exposes that a test could call: a free `pub fn`, a
// `pub fn` in an inherent impl, and every method of a `pub trait`.
//
// A method implementing a trait carries no visibility of its own and is reached
// through the trait rather than named, so it is deliberately not counted.

use stern4rust::finding::parsing::public_entry_point_finder::PublicEntryPointFinder;
use stern4rust::source_file::SourceFile;

fn find(body: &str) -> Vec<String> {
    PublicEntryPointFinder::find(&SourceFile::new("src/a.rs", body))
        .expect("parses")
        .iter()
        .map(|entry| entry.signature())
        .collect()
}

#[test]
fn find_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = PublicEntryPointFinder::find(&SourceFile::new("src/a.rs", "pub fn broken( {\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn find_of_a_free_public_function_counts_it() {
    // Arrange & Act
    let found = find("pub fn run(one: usize) {}\n");

    // Assert
    assert_eq!(found, ["run/1"]);
}

#[test]
fn find_of_a_private_function_counts_nothing() {
    // Arrange & Act
    let found = find("fn hidden(one: usize) {}\n");

    // Assert
    assert!(found.is_empty(), "got {found:?}");
}

// A trait's methods are as public as the trait and need no `pub` of their own.
#[test]
fn find_of_a_public_trait_counts_every_method() {
    // Arrange & Act
    let found = find(
        "pub trait Rule {\n    fn name(&self) -> &'static str;\n    fn check(&self, one: usize);\n}\n",
    );

    // Assert
    assert_eq!(found, ["name/0", "check/1"]);
}

// It carries no visibility and is reached through the trait, not by name.
#[test]
fn find_of_a_trait_implementation_counts_nothing() {
    // Arrange & Act
    let found =
        find("pub struct A;\nimpl Rule for A {\n    fn name(&self) -> &'static str { \"a\" }\n}\n");

    // Assert
    assert!(found.is_empty(), "got {found:?}");
}

// The receiver is not an argument at the call site, so it is not counted.
#[test]
fn find_of_an_inherent_method_does_not_count_the_receiver() {
    // Arrange & Act
    let found = find(
        "pub struct A;\nimpl A {\n    pub fn with_fixed(self, n: usize) -> Self { self }\n}\n",
    );

    // Assert
    assert_eq!(found, ["with_fixed/1"]);
}

#[test]
fn find_of_an_inline_module_descends_into_it() {
    // Arrange & Act
    let found = find("pub mod inner {\n    pub fn run() {}\n}\n");

    // Assert
    assert_eq!(found, ["run/0"]);
}
