// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A module-level constant declared after an item that is not one: its name, the
// line naming it, and the whole block that moves with it.

use stern4rust::finding::model::layout::misplaced_constant::MisplacedConstant;

fn limit() -> MisplacedConstant {
    MisplacedConstant::new(String::from("LIMIT"), 9, 7, 10)
}

#[test]
fn holds_a_line_either_side_of_its_block_returns_false() {
    // Arrange
    let constant = limit();

    // Act
    let held = (constant.holds(6), constant.holds(11));

    // Assert
    assert_eq!(held, (false, false));
}

// The comment above a constant is part of its block, so it moves with it.
#[test]
fn holds_the_first_line_of_its_block_returns_true() {
    // Arrange & Act
    let held = limit().holds(7);

    // Assert
    assert!(held);
}

#[test]
fn holds_the_last_line_of_its_block_returns_true() {
    // Arrange & Act
    let held = limit().holds(10);

    // Assert
    assert!(held);
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = limit();

    // Assert
    assert_eq!(found.name(), "LIMIT");
    assert_eq!(found.line(), 9);
    assert_eq!((found.first_line(), found.last_line()), (7, 10));
}
