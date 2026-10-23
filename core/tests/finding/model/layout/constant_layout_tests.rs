// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a file's misplaced constants are and where they go: after the item that
// ends the constants group, or before the first item when nothing precedes it.

use stern4rust::finding::model::layout::constant_layout::ConstantLayout;
use stern4rust::finding::model::layout::misplaced_constant::MisplacedConstant;

fn constant(name: &str, first_line: usize, last_line: usize) -> MisplacedConstant {
    MisplacedConstant::new(String::from(name), last_line, first_line, last_line)
}

fn layout() -> ConstantLayout {
    ConstantLayout::new(
        String::from("Widget"),
        4,
        false,
        vec![constant("LOW", 7, 7), constant("HIGH", 9, 11)],
        None,
    )
}

#[test]
fn moves_a_line_between_two_misplaced_constants_returns_false() {
    // Arrange & Act
    let moved = layout().moves(8);

    // Assert
    assert!(!moved);
}

#[test]
fn moves_a_line_of_a_misplaced_constant_returns_true() {
    // Arrange & Act
    let moved = layout().moves(10);

    // Assert
    assert!(moved);
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = layout();

    // Assert
    assert_eq!(found.leader(), "Widget");
    assert_eq!(found.insertion_line(), 4);
    assert!(!found.follows_constant());
    assert_eq!(found.misplaced().len(), 2);
    assert_eq!(found.refusal(), None);
}
