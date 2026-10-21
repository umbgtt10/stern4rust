// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Why --fix leaves a file's constants where they are. Each reason is read by
// somebody about to move them by hand, so it says what the fixer could not be
// sure of.

use stern4rust::finding::model::layout::fix_refusal::FixRefusal;

#[test]
fn reason_of_a_file_with_a_macro_rules_names_its_textual_order() {
    // Arrange & Act
    let reason = FixRefusal::MacroRules.reason();

    // Assert
    assert!(reason.contains("`macro_rules!`"), "got {reason:?}");
    assert!(reason.contains("textual order"), "got {reason:?}");
}

#[test]
fn reason_of_a_free_comment_names_the_comment() {
    // Arrange & Act
    let reason = FixRefusal::FreeComment.reason();

    // Assert
    assert!(reason.contains("comment"), "got {reason:?}");
}

#[test]
fn reason_of_a_macro_invocation_names_its_expansion() {
    // Arrange & Act
    let reason = FixRefusal::MacroInvocation.reason();

    // Assert
    assert!(reason.contains("expansion"), "got {reason:?}");
}

#[test]
fn reason_of_a_shared_line_names_the_line() {
    // Arrange & Act
    let reason = FixRefusal::SharedLine.reason();

    // Assert
    assert!(reason.contains("shares a line"), "got {reason:?}");
}

#[test]
fn reason_of_an_opening_comment_names_the_header() {
    // Arrange & Act
    let reason = FixRefusal::OpeningComment.reason();

    // Assert
    assert!(reason.contains("header"), "got {reason:?}");
}
