// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A program the runner could not start, said the way the run reports it.

use xtask::process::launch_failure::LaunchFailure;

#[test]
fn to_string_names_the_program_and_why() {
    // Arrange
    let failure = LaunchFailure {
        program: String::from("cargo"),
        reason: String::from("No such file or directory"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(text, "failed to launch cargo: No such file or directory");
}
