// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One publicly reachable function, identified by name and arity.
//
// Arity rather than parameter types, and that limit is deliberate: at a call
// site `check(3, &paths)` offers two arguments and nothing that says whether
// they fit `usize` and `&[&str]`. Arity is free and separates `new()` from
// `new(a, b)`, which is most of what a bare name confuses.

use stern4rust::finding::model::public_entry_point::PublicEntryPoint;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let entry = PublicEntryPoint::new("with_fixed", 1);

    // Assert
    assert_eq!(entry.name, "with_fixed");
    assert_eq!(entry.arity, 1);
}

#[test]
fn signature_of_an_entry_point_names_it_with_its_arity() {
    // Arrange & Act
    let signature = PublicEntryPoint::new("with_baseline", 3).signature();

    // Assert
    assert_eq!(signature, "with_baseline/3");
}

// Two entry points sharing a name must stay distinguishable on the page.
#[test]
fn signature_of_two_arities_differs_for_the_same_name() {
    // Arrange
    let nullary = PublicEntryPoint::new("new", 0);
    let binary = PublicEntryPoint::new("new", 2);

    // Act
    let signature = nullary.signature();

    // Assert
    assert_eq!(signature, "new/0");
    assert_ne!(signature, binary.signature());
}
