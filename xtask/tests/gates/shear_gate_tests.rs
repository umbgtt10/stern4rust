// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;
use xtask::gates::shear_gate::ShearGate;

const NO_EXCEPTIONS: &str = r#"{"packages":[{"name":"slotgate","metadata":null}],"metadata":null}"#;

fn gate(runner: &FakeCommandRunner) -> ShearGate<'_> {
    ShearGate::new(runner, String::from("/repo/Cargo.toml"))
}

#[test]
fn label_names_the_dependency_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Dependencies");
}

#[test]
fn run_reads_the_workspace_metadata_then_runs_shear_locked_and_strict() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(NO_EXCEPTIONS);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        runner.calls(),
        vec![
            vec![
                String::from("metadata"),
                String::from("--no-deps"),
                String::from("--format-version"),
                String::from("1"),
                String::from("--manifest-path"),
                String::from("/repo/Cargo.toml"),
            ],
            vec![
                String::from("shear"),
                String::from("--locked"),
                String::from("--deny-warnings"),
                String::from("/repo"),
            ],
        ]
    );
}

#[test]
fn run_with_an_exception_without_a_reason_names_it_and_runs_no_shear() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(
        r#"{"packages":[{"name":"core","metadata":{"cargo-shear":{"ignored":["proc-macro2"]}}}],"metadata":null}"#,
    );

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Found(String::from(
            "cargo-shear ignores without a reason: core: proc-macro2"
        )))
    );
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn run_with_exit_code_one_reports_a_dependency_finding() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_stdout(NO_EXCEPTIONS)
        .with_streaming_code(Some(1))
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Found(String::from(
            "an unused or misplaced dependency"
        )))
    );
}

#[test]
fn run_with_exit_code_two_reports_it_could_not_run() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_stdout(NO_EXCEPTIONS)
        .with_streaming_code(Some(2))
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Err(GateFailure::CouldNotRun(Some(2))));
}

#[test]
fn run_with_metadata_failing_reports_its_exit_code_and_runs_no_shear() {
    // Arrange
    let runner = FakeCommandRunner::new().with_capturing_code(Some(101));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Err(GateFailure::CouldNotRun(Some(101))));
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn run_with_metadata_unreadable_reports_it_and_runs_no_shear() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout("not a cargo metadata document");

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(
        matches!(&result, Err(GateFailure::Found(text)) if text.starts_with("could not read cargo metadata: ")),
        "{result:?}"
    );
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint_and_runs_nothing() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::NotInstalled(String::from("cargo-shear")))
    );
    assert!(runner.calls().is_empty());
}
