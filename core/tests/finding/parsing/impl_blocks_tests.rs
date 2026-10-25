// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The impl blocks of a file: the types they are for, and the identifiers their
// bodies use. A free function an impl block calls is that block's helper.

use stern4rust::finding::parsing::impl_blocks::ImplBlocks;
use stern4rust::finding::parsing::item_naming::ItemNaming;
use syn::Item;
use syn::Type;
use syn::parse_file;
use syn::parse_str;

fn items(source: &str) -> Vec<Item> {
    parse_file(source).expect("parses").items
}

// A trait impl's methods are the trait's; only the type's own impls are its.
#[test]
fn inherent_leaves_trait_impls_out_and_descends_into_inline_modules() {
    // Arrange
    let items = items(
        "impl Widget {}\n\nimpl Display for Widget {}\n\nmod detail {\n    impl super::Gadget {}\n}\n",
    );

    // Act
    let inherent = ImplBlocks::inherent(&items);

    // Assert
    let types: Vec<String> = inherent
        .iter()
        .map(|block| ItemNaming::implemented_type(&block.self_ty))
        .collect();
    assert_eq!(types, vec!["Widget", "Gadget"]);
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
fn target_of_a_generic_impl_is_the_last_segment_of_its_type() {
    // Arrange
    let self_ty: Type = parse_str("crate::store::Store<u8>").expect("parses");

    // Act
    let target = ImplBlocks::target(&self_ty);

    // Assert
    assert_eq!(target.as_deref(), Some("Store"));
}

#[test]
fn target_of_a_reference_type_is_none() {
    // Arrange
    let self_ty: Type = parse_str("&Store").expect("parses");

    // Act
    let target = ImplBlocks::target(&self_ty);

    // Assert
    assert_eq!(target, None);
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
