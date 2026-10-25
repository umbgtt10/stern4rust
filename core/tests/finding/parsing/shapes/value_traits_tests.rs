// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The traits a record may implement and stay a record: printing, comparing,
// converting and defaulting are what a value does. A fixed list, so no package
// decides for itself what counts as behaviour.

use stern4rust::finding::parsing::shapes::value_traits::ValueTraits;

#[test]
fn contains_a_trait_off_the_list_returns_false() {
    // Arrange
    let names = ["Iterator", "Drop", "Rule", "Visit"];

    // Act
    let contained: Vec<bool> = names
        .iter()
        .map(|name| ValueTraits::contains(name))
        .collect();

    // Assert
    assert_eq!(contained, [false; 4]);
}

#[test]
fn contains_a_trait_on_the_list_returns_true() {
    // Arrange
    let names = [
        "Clone",
        "Debug",
        "Display",
        "Default",
        "From",
        "TryFrom",
        "Serialize",
    ];

    // Act
    let contained: Vec<bool> = names
        .iter()
        .map(|name| ValueTraits::contains(name))
        .collect();

    // Assert
    assert_eq!(contained, [true; 7]);
}
