// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use xtask::gates::gate::Gate;

// A gate that records whether it ran, so Stage2's ordering and its
// stop-at-the-first-failure behaviour can be observed without a real tool.
pub struct FakeGate {
    label: String,
    outcome: Result<(), String>,
    ran: RefCell<bool>,
}

impl FakeGate {
    pub fn passing(label: &str) -> Self {
        Self {
            label: String::from(label),
            outcome: Ok(()),
            ran: RefCell::new(false),
        }
    }

    pub fn failing(label: &str, reason: &str) -> Self {
        Self {
            label: String::from(label),
            outcome: Err(String::from(reason)),
            ran: RefCell::new(false),
        }
    }

    pub fn has_run(&self) -> bool {
        *self.ran.borrow()
    }
}

impl Gate for FakeGate {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn run(&self) -> Result<(), String> {
        *self.ran.borrow_mut() = true;
        self.outcome.clone()
    }
}

#[test]
fn label_through_a_trait_object_returns_the_given_label() {
    // Arrange
    let gate = FakeGate::passing("House rules");
    let as_trait_object: &dyn Gate = &gate;

    // Act
    let label = as_trait_object.label();

    // Assert
    assert_eq!(label, "House rules");
}

#[test]
fn run_on_a_failing_gate_returns_its_reason() {
    // Arrange
    let gate = FakeGate::failing("a gate", "because");

    // Act
    let result = gate.run();

    // Assert
    assert_eq!(result, Err(String::from("because")));
}

#[test]
fn run_on_a_passing_gate_returns_ok_and_marks_it_run() {
    // Arrange
    let gate = FakeGate::passing("a gate");

    // Act
    let result = gate.run();

    // Assert
    assert!(result.is_ok());
    assert!(gate.has_run());
}
