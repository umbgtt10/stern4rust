// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Whether a type holds a mutable reference anywhere it structurally can: the
// question `returned-mutations` asks of a parameter and `immutable-returns` of a
// return type. A callback's own contract is not descended into, and a search
// that skips statics passes over `&'static mut` while still looking inside it.

use stern4rust::finding::parsing::mutable_reference_search::MutableReferenceSearch;
use syn::Type;
use syn::parse_str;

fn ty(source: &str) -> Type {
    parse_str(source).expect("parses as a type")
}

// A bare function pointer taking a `&mut` is a callback contract by another
// spelling.
#[test]
fn finds_in_a_bare_fn_pointer_taking_a_mutable_reference_returns_false() {
    // Arrange
    let pointer = ty("fn(&mut u32)");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&pointer);

    // Assert
    assert!(!found);
}

#[test]
fn finds_in_a_callback_taking_a_mutable_reference_returns_false() {
    // Arrange
    let callback = ty("impl FnMut(&mut u32)");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&callback);

    // Assert
    assert!(!found);
}

// Skipping a static reference does not skip what it points at.
#[test]
fn finds_in_a_mutable_reference_behind_a_static_one_returns_true() {
    // Arrange
    let nested = ty("&'static mut Option<&'a mut u8>");

    // Act
    let found = MutableReferenceSearch::new(true).finds_in(&nested);

    // Assert
    assert!(found);
}

#[test]
fn finds_in_a_mutable_reference_inside_a_generic_returns_true() {
    // Arrange
    let wrapped = ty("Option<&mut u32>");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&wrapped);

    // Assert
    assert!(found);
}

#[test]
fn finds_in_a_mutable_reference_inside_a_slice_returns_true() {
    // Arrange
    let slice = ty("&[&mut u32]");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&slice);

    // Assert
    assert!(found);
}

#[test]
fn finds_in_a_mutable_reference_inside_a_tuple_returns_true() {
    // Arrange
    let tuple = ty("(u32, &mut u32)");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&tuple);

    // Assert
    assert!(found);
}

#[test]
fn finds_in_a_mutable_reference_inside_an_array_returns_true() {
    // Arrange
    let array = ty("[&mut u32; 2]");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&array);

    // Assert
    assert!(found);
}

#[test]
fn finds_in_a_mutable_reference_returns_true() {
    // Arrange
    let reference = ty("&mut u32");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&reference);

    // Assert
    assert!(found);
}

#[test]
fn finds_in_a_shared_reference_returns_false() {
    // Arrange
    let reference = ty("&u32");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&reference);

    // Assert
    assert!(!found);
}

#[test]
fn finds_in_a_static_mutable_reference_when_not_skipping_statics_returns_true() {
    // Arrange
    let reference = ty("&'static mut u32");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&reference);

    // Assert
    assert!(found);
}

// What an embedded executor hands over once: a static, not a type's internals.
#[test]
fn finds_in_a_static_mutable_reference_when_skipping_statics_returns_false() {
    // Arrange
    let reference = ty("&'static mut u32");

    // Act
    let found = MutableReferenceSearch::new(true).finds_in(&reference);

    // Assert
    assert!(!found);
}

#[test]
fn finds_in_an_owned_type_returns_false() {
    // Arrange
    let owned = ty("Vec<u32>");

    // Act
    let found = MutableReferenceSearch::new(false).finds_in(&owned);

    // Assert
    assert!(!found);
}
