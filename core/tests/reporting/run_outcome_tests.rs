// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The verdict, and the one place the exit-code contract is stated. 2 has to stay
// distinct from 1: a wrapper that cannot tell a broken rule from a broken tool
// will eventually treat a crash as a pass.

use stern4rust::reporting::run_outcome::RunOutcome;

#[test]
fn exit_code_of_clean_is_zero() {
    // Arrange & Act
    let code = RunOutcome::Clean.exit_code();

    // Assert
    assert_eq!(code, 0);
}

// Not 1. A tool that could not run returns 1 through main's error arm, and the
// two must stay tellable apart from a script.
#[test]
fn exit_code_of_rules_broken_is_two() {
    // Arrange & Act
    let code = RunOutcome::RulesBroken.exit_code();

    // Assert
    assert_eq!(code, 2);
}

#[test]
fn of_a_single_offence_is_rules_broken() {
    // Arrange & Act
    let outcome = RunOutcome::of(1);

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
}

#[test]
fn of_no_offences_is_clean() {
    // Arrange & Act
    let outcome = RunOutcome::of(0);

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}
