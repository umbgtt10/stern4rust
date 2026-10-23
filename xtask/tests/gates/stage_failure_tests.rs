// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The gate that stopped stage 2, and why.

use xtask::gates::gate_failure::GateFailure;
use xtask::gates::stage_failure::StageFailure;

#[test]
fn new_keeps_the_label_and_the_failure() {
    // Arrange & Act
    let failure = StageFailure::new(
        String::from("House rules"),
        GateFailure::Found(String::from("a rule was broken")),
    );

    // Assert
    assert_eq!(failure.label(), "House rules");
    assert_eq!(
        failure.failure().to_string(),
        GateFailure::Found(String::from("a rule was broken")).to_string()
    );
}

#[test]
fn to_string_puts_the_gates_label_before_its_failure() {
    // Arrange
    let failure = StageFailure::new(
        String::from("House rules"),
        GateFailure::Found(String::from("a rule was broken")),
    );

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "House rules (a rule was broken)");
}
