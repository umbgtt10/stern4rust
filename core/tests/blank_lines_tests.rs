// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Lines with the blank ones at an end dropped -- what the fixers do where
// moved lines meet the lines that stay, so a gap is always the one blank line
// they put there.

use stern4rust::blank_lines::BlankLines;

#[test]
fn without_leading_drops_the_blank_lines_before_the_first_with_text() {
    // Arrange
    let lines = ["", "  ", "a", "", "b"];

    // Act
    let kept = BlankLines::without_leading(&lines);

    // Assert
    assert_eq!(kept, vec!["a", "", "b"]);
}

#[test]
fn without_trailing_drops_the_blank_lines_after_the_last_with_text() {
    // Arrange
    let lines = [
        String::from("a"),
        String::new(),
        String::from("b"),
        String::from("\t"),
        String::new(),
    ];

    // Act
    let kept = BlankLines::without_trailing(&lines);

    // Assert
    assert_eq!(kept, vec!["a", "", "b"]);
}

#[test]
fn without_trailing_of_only_blank_lines_is_empty() {
    // Arrange
    let lines = ["", " "];

    // Act
    let kept = BlankLines::without_trailing(&lines);

    // Assert
    assert!(kept.is_empty());
}
