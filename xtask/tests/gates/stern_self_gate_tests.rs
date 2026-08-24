// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::gates::gate::Gate;
use xtask::gates::stern_self_gate::SternSelfGate;

fn gate(runner: &FakeCommandRunner) -> SternSelfGate<'_> {
    SternSelfGate::new(
        runner,
        String::from("cargo-stern4rust"),
        String::from("Cargo.toml"),
        vec![String::from("cargo-stern4rust")],
    )
}

// Everything before the `--` is cargo's; everything after it is the tool's.
fn split_at_separator(call: &[String]) -> (Vec<String>, Vec<String>) {
    let separator = call
        .iter()
        .position(|argument| argument == "--")
        .expect("the call separates cargo's arguments from the tool's");
    (call[..separator].to_vec(), call[separator + 1..].to_vec())
}

#[test]
fn label_names_the_own_rules_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Own rules");
}

#[test]
fn run_builds_the_tool_from_source_rather_than_an_install() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let call = &runner.calls()[0];
    assert_eq!(call[0], "run");
}

// Two flags that look alike sit on either side of the `--`. cargo gets `--bin`
// to choose what to build; the tool gets `--package` to choose what to read.
// Putting either on the wrong side still runs, which is what makes it worth
// pinning.
#[test]
fn run_gives_cargo_the_binary_and_the_tool_the_package() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let (for_cargo, for_tool) = split_at_separator(&runner.calls()[0]);
    assert!(for_cargo.contains(&String::from("--bin")));
    assert!(!for_cargo.contains(&String::from("--package")));
    assert!(for_tool.contains(&String::from("--package")));
    assert!(for_tool.contains(&String::from("--manifest-path")));
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

// 2 is the tool's own verdict; anything else non-zero means it never got far
// enough to have one. Collapsing them would let a build failure read as a
// clean codebase.
#[test]
fn run_with_exit_code_one_reports_a_failure_to_run() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Err(String::from("could not run, exit code 1")));
}

#[test]
fn run_with_exit_code_two_reports_a_broken_rule() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Err(String::from("a house coding rule was broken")));
}

// None means the process was signalled rather than exiting. That is a failure
// to run, not a verdict, and it must not be mistaken for either.
#[test]
fn run_with_no_exit_code_reports_a_failure_to_run() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(None);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(String::from("could not run, terminated by signal"))
    );
}
