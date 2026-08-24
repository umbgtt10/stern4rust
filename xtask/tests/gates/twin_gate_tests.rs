// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::gates::gate::Gate;
use xtask::gates::twin_gate::TwinGate;

fn gate(runner: &FakeCommandRunner) -> TwinGate<'_> {
    TwinGate::new(
        runner,
        String::from("Cargo.toml"),
        vec![String::from("cargo-stern4rust")],
    )
}

#[test]
fn label_names_the_mirrored_tests_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Mirrored tests");
}

#[test]
fn run_passes_the_manifest_path_and_package_to_the_tool() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    assert_eq!(
        runner.calls()[0],
        vec![
            String::from("twin4rust"),
            String::from("--manifest-path"),
            String::from("Cargo.toml"),
            String::from("--package"),
            String::from("cargo-stern4rust"),
        ]
    );
}

#[test]
fn run_with_a_non_zero_exit_code_reports_a_missing_mirrored_test() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(String::from("source files without a mirrored test"))
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

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.contains("cargo install cargo-twin4rust")));
}
