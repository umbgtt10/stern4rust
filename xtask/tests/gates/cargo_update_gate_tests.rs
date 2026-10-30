// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::gates::cargo_update_gate::CargoUpdateGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;

fn gate(runner: &FakeCommandRunner) -> CargoUpdateGate<'_> {
    CargoUpdateGate::new(runner, String::from("/repo/Cargo.toml"))
}

#[test]
fn label_names_the_update_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Update");
}

// Compatible versions only, into the lock file: no requirement in a manifest
// moves, so a published crate's users can still resolve what they could.
#[test]
fn run_updates_the_workspace_lock_file() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        runner.calls(),
        vec![vec![
            String::from("update"),
            String::from("--manifest-path"),
            String::from("/repo/Cargo.toml"),
        ]]
    );
}

#[test]
fn run_with_a_failing_update_reports_its_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(101));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Err(GateFailure::Exited(Some(101))));
}
