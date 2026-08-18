// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The wiring from arguments to a verdict, run against this crate's own tree.
//
// The verdict is a return value rather than a call to exit, which is the only
// reason any of this is reachable from a test at all. Two outcomes have to stay
// tellable apart: a broken rule is a successful run with a finding, while a tool
// that could not run is an Err. Collapsing them would let "I could not read your
// code" reach a gate script as a pass.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use stern4rust::args::Args;
use stern4rust::run_outcome::RunOutcome;
use stern4rust::runner::Runner;

const THIS_CRATE: &str = "cargo-stern4rust";

fn args_from(parts: &[&str]) -> Args {
    Args::parse_from(parts.iter().map(|part| (*part).to_string()))
}

fn header_file(name: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("stern4rust_header_{name}.txt"));
    fs::write(&path, contents).expect("write the header file");
    path
}

fn run_with_header(name: &str, contents: &str) -> RunOutcome {
    let path = header_file(name, contents);
    Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        THIS_CRATE,
        "--header-file",
        &path.to_string_lossy(),
    ]))
    .expect("the run itself should succeed")
}

// A run with no rules configured would report "all rules satisfied" having
// checked nothing, so it is an error rather than a clean verdict.
#[test]
fn run_without_any_rule_configured_is_an_error() {
    // Arrange
    let args = args_from(&["cargo-stern4rust", "--manifest-path", "Cargo.toml"]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_without_any_rule_configured_says_which_flag_would_enable_one() {
    // Arrange
    let args = args_from(&["cargo-stern4rust", "--manifest-path", "Cargo.toml"]);

    // Act
    let error = Runner::run(args).expect_err("an error");

    // Assert
    assert!(error.to_string().contains("--header-file"));
}

// This crate keeps its own rule, so pointing stern4rust at itself with its own
// header is the end-to-end case: it walks, reads and judges every file here.
#[test]
fn run_against_this_crate_with_its_own_header_is_clean() {
    // Arrange & Act
    let outcome = run_with_header(
        "own",
        "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
         // Licensed under the MIT License\n\
         // SPDX-License-Identifier: MIT\n",
    );

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}

#[test]
fn run_against_this_crate_with_a_header_it_does_not_carry_reports_rules_broken() {
    // Arrange & Act
    let outcome = run_with_header("foreign", "// Copyright 1999 Someone Else\n");

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
}

#[test]
fn run_with_an_unreadable_header_file_is_an_error() {
    // Arrange
    let absent = std::env::temp_dir().join("stern4rust_header_absent.txt");
    let _ = fs::remove_file(&absent);
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--header-file",
        &absent.to_string_lossy(),
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

// A typo in a gate script must fail loudly rather than scan nothing and pass.
#[test]
fn run_against_an_unknown_package_is_an_error() {
    // Arrange
    let path = header_file("unknown_package", "// Copyright 2025\n");
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        "no-such-package",
        "--header-file",
        &path.to_string_lossy(),
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}
