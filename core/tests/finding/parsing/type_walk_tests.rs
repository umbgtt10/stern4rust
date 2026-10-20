// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The types a type is made of, one level down -- the shape every search over a
// type walks. A callback's parameter list, an `impl Trait` bound and a `dyn` bound
// are not part of it: what a closure receives is the callback's contract.

use quote::ToTokens;
use stern4rust::finding::parsing::type_walk::TypeWalk;
use syn::Type;
use syn::parse_str;

fn texts(children: &[&Type]) -> Vec<String> {
    children
        .iter()
        .map(|child| child.to_token_stream().to_string())
        .collect()
}

fn ty(source: &str) -> Type {
    parse_str(source).expect("parses as a type")
}

#[test]
fn children_of_a_callback_are_none() {
    // Arrange
    let callback = ty("impl Fn(u8, u8) -> u8");

    // Act
    let children = TypeWalk::children(&callback);

    // Assert
    assert!(children.is_empty());
}

#[test]
fn children_of_a_generic_are_its_type_arguments() {
    // Arrange
    let generic = ty("Result<u8, String>");

    // Act
    let children = TypeWalk::children(&generic);

    // Assert
    assert_eq!(texts(&children), ["u8", "String"]);
}

#[test]
fn children_of_a_plain_path_are_none() {
    // Arrange
    let plain = ty("u64");

    // Act
    let children = TypeWalk::children(&plain);

    // Assert
    assert!(children.is_empty());
}

#[test]
fn children_of_a_reference_are_what_it_points_at() {
    // Arrange
    let reference = ty("&'a mut Vec<u8>");

    // Act
    let children = TypeWalk::children(&reference);

    // Assert
    assert_eq!(texts(&children), ["Vec < u8 >"]);
}

#[test]
fn children_of_a_tuple_are_its_elements() {
    // Arrange
    let tuple = ty("(u8, &str)");

    // Act
    let children = TypeWalk::children(&tuple);

    // Assert
    assert_eq!(texts(&children), ["u8", "& str"]);
}

#[test]
fn children_of_an_array_and_a_slice_are_their_elements() {
    // Arrange
    let array = ty("[u16; 4]");
    let slice = ty("[u32]");

    // Act
    let of_array = TypeWalk::children(&array);
    let of_slice = TypeWalk::children(&slice);

    // Assert
    assert_eq!(texts(&of_array), ["u16"]);
    assert_eq!(texts(&of_slice), ["u32"]);
}
