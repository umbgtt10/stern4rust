// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use xtask::process::launch_failure::LaunchFailure;
use xtask::reports::scores::score_failure::ScoreFailure;

#[test]
fn from_of_a_launch_failure_holds_it() {
    // Arrange
    let launch = LaunchFailure {
        program: String::from("git"),
        reason: String::from("no such program"),
    };

    // Act
    let failure = ScoreFailure::from(launch.clone());

    // Assert
    assert_eq!(failure, ScoreFailure::Launch(launch));
}

#[test]
fn to_string_of_a_failed_command_names_it_and_carries_its_stderr() {
    // Arrange
    let failure = ScoreFailure::CommandFailed {
        command: String::from("cargo grip4rust --json /scratch/core"),
        stderr: String::from("error: no such directory"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "cargo grip4rust --json /scratch/core failed: error: no such directory"
    );
}

#[test]
fn to_string_of_an_unreadable_report_names_the_tool_and_why() {
    // Arrange
    let failure = ScoreFailure::Unreadable {
        tool: String::from("cargo-braintax4rust"),
        reason: String::from("no overall.total_braintax"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "cargo-braintax4rust's report is unreadable: no overall.total_braintax"
    );
}

#[test]
fn to_string_of_no_source_says_nothing_under_core_is_staged() {
    // Arrange & Act
    let text = ScoreFailure::NoSource.to_string();

    // Assert
    assert_eq!(text, "core/ has no staged source file");
}
