// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The wiring from parsed arguments to a report. There is no rule implemented
// yet, so what these pin is the shape of the run rather than any verdict: it
// completes, and it does not exit the process on the way through. Once a rule
// lands, the exit-code contract belongs here too.

use clap::Parser;
use stern4rust::args::Args;
use stern4rust::runner::Runner;

fn args_from(parts: &[&str]) -> Args {
    Args::parse_from(parts.iter().map(|part| (*part).to_string()))
}

#[test]
fn run_without_any_arguments_completes() {
    // Arrange
    let args = args_from(&["cargo-stern4rust"]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_packages_completes() {
    // Arrange
    let args = args_from(&[
        "cargo-stern4rust",
        "--package",
        "alpha",
        "--package",
        "beta",
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_a_manifest_path_completes() {
    // Arrange
    let args = args_from(&["cargo-stern4rust", "--manifest-path", "Cargo.toml"]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_ok());
}
