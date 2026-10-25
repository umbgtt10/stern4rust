// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The names a crate's public API hands another crate: the signatures of its
// pub functions, pub methods and trait methods, the types of a pub struct's
// pub fields, a pub enum's variants, and pub aliases, constants and statics.
// What only the crate itself can reach is left out.

use std::collections::BTreeSet;
use stern4rust::finding::parsing::signatures::public_api_names::PublicApiNames;
use stern4rust::source_file::SourceFile;

fn names(contents: &str) -> BTreeSet<String> {
    PublicApiNames::of(&[SourceFile::new("src/lib.rs", contents)])
}

#[test]
fn of_a_file_that_does_not_parse_names_nothing() {
    // Arrange & Act
    let found = names("pub fn parse() -> LogLine {");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_a_private_fn_names_nothing() {
    // Arrange & Act
    let found = names("fn parse(kind: Kind) -> LogLine {\n    todo!()\n}\n");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_a_private_structs_pub_field_names_nothing() {
    // Arrange & Act
    let found = names("struct Window {\n    pub size: Size,\n}\n");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_a_private_trait_names_nothing() {
    // Arrange & Act
    let found = names("trait Port {\n    fn send(&self, frame: Frame) -> Ack;\n}\n");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_a_pub_alias_const_and_static_names_their_types() {
    // Arrange & Act
    let found = names(
        "pub type Votes = Vec<Ballot>;\npub const LIMIT: Limit = Limit;\npub static NAME: Label = Label;\n",
    );

    // Assert
    assert!(found.contains("Ballot"), "{found:?}");
    assert!(found.contains("Limit"), "{found:?}");
    assert!(found.contains("Label"), "{found:?}");
}

#[test]
fn of_a_pub_crate_fn_names_nothing() {
    // Arrange & Act
    let found = names("pub(crate) fn parse(kind: Kind) -> LogLine {\n    todo!()\n}\n");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_a_pub_enums_variants_name_their_types() {
    // Arrange & Act
    let found = names("pub enum Event {\n    Vote { ballot: Ballot },\n    Ack(Receipt),\n}\n");

    // Assert
    assert!(found.contains("Ballot"), "{found:?}");
    assert!(found.contains("Receipt"), "{found:?}");
}

#[test]
fn of_a_pub_fn_names_the_types_in_its_signature() {
    // Arrange & Act
    let found = names("pub fn parse(line: &str, kind: Kind) -> Option<LogLine> {\n    None\n}\n");

    // Assert
    assert!(found.contains("Kind"), "{found:?}");
    assert!(found.contains("LogLine"), "{found:?}");
}

#[test]
fn of_a_pub_item_in_an_inline_module_names_its_types() {
    // Arrange & Act
    let found =
        names("pub mod inner {\n    pub fn run() -> Outcome {\n        Outcome\n    }\n}\n");

    // Assert
    assert!(found.contains("Outcome"), "{found:?}");
}

#[test]
fn of_a_pub_method_names_its_signature_and_a_private_one_does_not() {
    // Arrange & Act
    let found = names(
        "impl Parser {\n    pub fn next(&self) -> Token {\n        todo!()\n    }\n\n    fn peek(&self) -> Lookahead {\n        todo!()\n    }\n}\n",
    );

    // Assert
    assert!(found.contains("Token"), "{found:?}");
    assert!(!found.contains("Lookahead"), "{found:?}");
}

#[test]
fn of_a_pub_structs_pub_field_names_its_type_and_a_private_field_does_not() {
    // Arrange & Act
    let found = names("pub struct Report {\n    pub verdict: Verdict,\n    total: Total,\n}\n");

    // Assert
    assert!(found.contains("Verdict"), "{found:?}");
    assert!(!found.contains("Total"), "{found:?}");
}

#[test]
fn of_a_pub_traits_methods_name_their_signatures() {
    // Arrange & Act
    let found = names("pub trait Port {\n    fn send(&self, frame: Frame) -> Ack;\n}\n");

    // Assert
    assert!(found.contains("Frame"), "{found:?}");
    assert!(found.contains("Ack"), "{found:?}");
}

#[test]
fn of_a_trait_impl_names_nothing() {
    // Arrange & Act
    let found = names(
        "impl Display for Window {\n    fn fmt(&self, formatter: &mut Formatter) -> Result {\n        Ok(())\n    }\n}\n",
    );

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_an_item_in_a_cfg_test_module_names_nothing() {
    // Arrange & Act
    let found =
        names("#[cfg(test)]\nmod tests {\n    pub fn fake() -> Fake {\n        Fake\n    }\n}\n");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}
