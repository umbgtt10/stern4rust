// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The impl blocks of a file: the types they are for, and the identifiers their
// bodies use. A free function an impl block calls is that block's helper.

use stern4rust::finding::parsing::impl_blocks::ImplBlocks;
use syn::Item;
use syn::parse_file;

fn items(source: &str) -> Vec<Item> {
    parse_file(source).expect("parses").items
}

#[test]
fn names_an_identifier_an_impl_block_uses_is_true() {
    // Arrange
    let blocks = ImplBlocks::of(&items(
        "impl Widget {\n    fn draw(&self) {\n        border();\n    }\n}\n",
    ));

    // Act
    let named = blocks.names("border");

    // Assert
    assert!(named);
}

#[test]
fn names_an_identifier_used_only_outside_every_impl_is_false() {
    // Arrange
    let blocks = ImplBlocks::of(&items(
        "impl Widget {}\n\nfn border() {\n    helper();\n}\n",
    ));

    // Act
    let named = blocks.names("helper");

    // Assert
    assert!(!named);
}

#[test]
fn targets_descends_into_an_inline_module() {
    // Arrange
    let blocks = ImplBlocks::of(&items("mod detail {\n    impl Hidden {}\n}\n"));

    // Act
    let targets = blocks.targets();

    // Assert
    assert_eq!(targets, ["Hidden"]);
}

#[test]
fn targets_lists_the_type_of_every_impl_block_in_source_order() {
    // Arrange
    let blocks = ImplBlocks::of(&items(
        "pub struct Widget;\nimpl Widget {}\n\nimpl Display for Gadget {}\n",
    ));

    // Act
    let targets = blocks.targets();

    // Assert
    assert_eq!(targets, ["Widget", "Gadget"]);
}

#[test]
fn targets_of_a_file_without_impl_blocks_is_empty() {
    // Arrange
    let blocks = ImplBlocks::of(&items("pub struct Widget;\n\npub fn draw() {}\n"));

    // Act
    let targets = blocks.targets();

    // Assert
    assert!(targets.is_empty());
}
