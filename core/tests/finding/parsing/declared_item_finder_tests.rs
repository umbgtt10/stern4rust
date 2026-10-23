// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The types and the traits a file declares, in the order it declares them.
//
// Types are structs and enums, the two kinds single-implemented-type counts. A
// type alias names another type rather than declaring one, and a trait is found
// apart from the types because named-subject ranks it ahead of them. Both walks
// descend into inline modules, as ImplementedTypeFinder does.

use stern4rust::finding::model::declared_item::DeclaredItem;
use stern4rust::finding::parsing::declared_item_finder::DeclaredItemFinder;
use syn::Item;
use syn::parse_file;

fn items(source: &str) -> Vec<Item> {
    parse_file(source).expect("parses").items
}

fn names(found: Vec<DeclaredItem>) -> Vec<String> {
    found
        .into_iter()
        .map(|item| item.name().to_string())
        .collect()
}

#[test]
fn traits_descends_into_an_inline_module() {
    // Arrange
    let items = items("mod inner {\n    pub trait Hidden {}\n}\n");

    // Act
    let found = DeclaredItemFinder::traits(&items);

    // Assert
    assert_eq!(names(found), ["Hidden"]);
}

#[test]
fn traits_finds_every_trait_in_source_order_and_no_type() {
    // Arrange
    let items = items("pub trait Send {}\npub struct Envelope;\npub trait Receive {}\n");

    // Act
    let found = DeclaredItemFinder::traits(&items);

    // Assert
    assert_eq!(names(found), ["Send", "Receive"]);
}

#[test]
fn types_descends_into_an_inline_module() {
    // Arrange
    let items = items("mod inner {\n    pub enum Hidden {\n        One,\n    }\n}\n");

    // Act
    let found = DeclaredItemFinder::types(&items);

    // Assert
    assert_eq!(names(found), ["Hidden"]);
}

#[test]
fn types_finds_structs_and_enums_in_source_order_and_no_trait_or_alias() {
    // Arrange
    let items = items(
        "pub enum TimerKey {\n    Round,\n}\npub trait Source {}\npub type Pair = (u8, u8);\npub struct Action;\n",
    );

    // Act
    let found = DeclaredItemFinder::types(&items);

    // Assert
    assert_eq!(names(found), ["TimerKey", "Action"]);
}

#[test]
fn types_records_the_line_of_each_declaration() {
    // Arrange
    let items = items("pub struct First;\n\npub struct Second;\n");

    // Act
    let found = DeclaredItemFinder::types(&items);

    // Assert
    assert_eq!(
        found,
        [
            DeclaredItem::new("First", 1),
            DeclaredItem::new("Second", 3)
        ]
    );
}
