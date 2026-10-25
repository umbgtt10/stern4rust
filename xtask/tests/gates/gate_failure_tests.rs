// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Why a gate failed, one variant per way it can, each said exactly as the run
// has always reported it.

use xtask::crap::report_failure::ReportFailure;
use xtask::gates::gate_failure::GateFailure;
use xtask::process::launch_failure::LaunchFailure;

#[test]
fn from_a_launch_failure_is_not_launched() {
    // Arrange
    let launch = LaunchFailure {
        program: String::from("cargo"),
        reason: String::from("denied"),
    };

    // Act
    let failure = GateFailure::from(launch.clone());

    // Assert
    assert_eq!(failure, GateFailure::NotLaunched(launch));
}

#[test]
fn to_string_of_a_finding_is_the_finding() {
    // Arrange & Act
    let text = GateFailure::Found(String::from("source files without a mirrored test")).to_string();

    // Assert
    assert_eq!(text, "source files without a mirrored test");
}

#[test]
fn to_string_of_a_missing_tool_says_how_to_install_it() {
    // Arrange & Act
    let text = GateFailure::NotInstalled(String::from("cargo-twin4rust")).to_string();

    // Assert
    assert_eq!(
        text,
        "cargo-twin4rust is not installed -- run: cargo install cargo-twin4rust"
    );
}

#[test]
fn to_string_of_a_run_ended_by_a_signal_says_so() {
    // Arrange & Act
    let text = GateFailure::CouldNotRun(None).to_string();

    // Assert
    assert_eq!(text, "could not run, terminated by signal");
}

#[test]
fn to_string_of_a_run_that_could_not_finish_gives_its_exit_code() {
    // Arrange & Act
    let text = GateFailure::CouldNotRun(Some(1)).to_string();

    // Assert
    assert_eq!(text, "could not run, exit code 1");
}

#[test]
fn to_string_of_an_unexpected_exit_gives_the_code() {
    // Arrange & Act
    let text = GateFailure::Exited(Some(3)).to_string();

    // Assert
    assert_eq!(text, "exit code Some(3)");
}

#[test]
fn to_string_of_an_unreported_run_gives_its_exit_code_stderr_and_stdout_tail() {
    // Arrange
    let failure = GateFailure::Unreported {
        failure: ReportFailure::Missing,
        exit_code: Some(101),
        stderr: String::from("boom"),
        stdout_tail: String::from("last line"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "could not find a JSON report in crap4rust's output (exit code Some(101))\nstderr: boom\nstdout tail:\nlast line"
    );
}
