// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::gates::dry_gate::DryGate;
use xtask::gates::gate::Gate;

fn gate(runner: &FakeCommandRunner) -> DryGate<'_> {
    DryGate::new(
        runner,
        String::from("/repo/core/src"),
        String::from("/repo/dry4rust-baseline.json"),
    )
}

#[test]
fn label_names_the_duplication_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Duplication");
}

// The baseline is what turns a zero ceiling into a gate on what a change adds
// rather than a wall made of what was already there.
#[test]
fn run_checks_the_path_against_the_baseline_with_zero_ceilings() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    assert_eq!(
        runner.calls()[0],
        vec![
            String::from("dry4rust"),
            String::from("--path"),
            String::from("/repo/core/src"),
            String::from("--baseline"),
            String::from("/repo/dry4rust-baseline.json"),
            String::from("check"),
            String::from("--max-exact"),
            String::from("0"),
            String::from("--max-near"),
            String::from("0"),
        ]
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

// 1 is dry4rust's own "a ceiling was exceeded", which has to read as
// duplication added rather than as the tool failing to run.
#[test]
fn run_with_exit_code_one_reports_duplication_beyond_the_baseline() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(String::from(
            "duplication was added beyond /repo/dry4rust-baseline.json"
        ))
    );
}

// 2 is a configuration dry4rust refused, or a baseline it could not read --
// the tool did not get as far as measuring anything.
#[test]
fn run_with_exit_code_two_reports_the_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.contains("exit code")));
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.contains("cargo install cargo-dry4rust")));
}
