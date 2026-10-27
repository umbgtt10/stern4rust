// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The wiring from an invocation to a verdict. What a run finds is the command
// line's to test, end to end; what the core owes it is a run that could not
// judge anything coming back as an Err rather than as a verdict.

use std::path::PathBuf;
use stern4rust::runner::Runner;
use stern4rust::settings::invocation::Invocation;

#[test]
fn run_reporting_with_a_workspace_that_cannot_be_read_fails() {
    // Arrange
    let invocation = Invocation {
        manifest_directory: PathBuf::from("no/such/workspace"),
        manifest_path: Some(PathBuf::from("no/such/workspace/Cargo.toml")),
        ..Invocation::default()
    };

    // Act
    let result = Runner::run_reporting(&invocation);

    // Assert
    assert!(result.is_err());
}
