// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a file's misplaced constants are and where they go: after the item that
// ends the constants group, or before the first item when nothing precedes it.

use stern4rust::finding::model::constant_layout::ConstantLayout;
use stern4rust::finding::model::misplaced_constant::MisplacedConstant;

fn constant(name: &str, first_line: usize, last_line: usize) -> MisplacedConstant {
    MisplacedConstant {
        name: String::from(name),
        line: last_line,
        first_line,
        last_line,
    }
}

fn layout() -> ConstantLayout {
    ConstantLayout {
        leader: String::from("Widget"),
        insertion_line: 4,
        follows_constant: false,
        misplaced: vec![constant("LOW", 7, 7), constant("HIGH", 9, 11)],
        refusal: None,
    }
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
