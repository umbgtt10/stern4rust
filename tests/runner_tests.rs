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
use std::env;
use std::fs;
use std::path::PathBuf;
use stern4rust::args::Args;
use stern4rust::run_outcome::RunOutcome;
use stern4rust::runner::Runner;

const THIS_CRATE: &str = "cargo-stern4rust";

fn args_from(parts: &[&str]) -> Args {
    Args::parse_from(parts.iter().map(|part| (*part).to_string()))
}

fn config_directory(name: &str, contents: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("stern4rust_run_{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(path.join("src")).expect("create the package");
    fs::write(
        path.join("Cargo.toml"),
        "[package]
name = \"probe\"
version = \"0.1.0\"
edition = \"2021\"
",
    )
    .expect("write the manifest");
    fs::write(
        path.join("src/lib.rs"),
        "pub mod widget;
",
    )
    .expect("write the registry");
    fs::write(
        path.join("src/widget.rs"),
        "pub struct Widget;
",
    )
    .expect("write the module");
    fs::write(path.join("stern4rust.toml"), contents).expect("write the config");
    path
}

fn header_file(name: &str, contents: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("stern4rust_header_{name}.txt"));
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

#[test]
fn run_against_this_crate_with_a_header_it_does_not_carry_reports_rules_broken() {
    // Arrange & Act
    let outcome = run_with_header("foreign", "// Copyright 1999 Someone Else\n");

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
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

// A switch that quietly matched nothing would look exactly like a switch that
// worked, which is the whole reason this is an error rather than a no-op.
// The end-to-end proof that an exclusion removes files from judgement rather
// than merely from the report: the same run that reports offences against a
// header this crate does not carry finds nothing once every file is excluded.
// A repository states its settings once instead of at every invocation, and
// the report names the file so the switches in force are not invisible.
#[test]
fn run_with_a_config_file_applies_its_settings() {
    // Arrange
    let path = config_directory(
        "applies",
        "rules = [\"tests-layout\"]
",
    );

    // Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &path.join("Cargo.toml").to_string_lossy(),
    ]))
    .expect("the run itself should succeed");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}

#[test]
fn run_with_an_exclusion_covering_every_file_finds_nothing_to_judge() {
    // Arrange
    let path = header_file(
        "excluded",
        "// nobody carries this header
",
    );

    // Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        THIS_CRATE,
        "--header-file",
        &path.to_string_lossy(),
        "--exclude",
        "**/*.rs",
    ]))
    .expect("the run itself should succeed");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}

// A file that exists and cannot be understood must not be treated as absent:
// running as though it were would apply a configuration nobody chose.
#[test]
fn run_with_an_invalid_config_file_is_an_error() {
    // Arrange
    let path = config_directory(
        "invalid",
        "rules = 7
",
    );

    // Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &path.join("Cargo.toml").to_string_lossy(),
    ]));

    // Assert
    assert!(outcome.is_err());
}

#[test]
fn run_with_an_unknown_rule_name_is_an_error() {
    // Arrange
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--skip",
        "test-file-strucutre",
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_with_an_unreadable_header_file_is_an_error() {
    // Arrange
    let absent = env::temp_dir().join("stern4rust_header_absent.txt");
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

// Asking for a rule by name and getting an empty run is worse than not asking:
// the registry's usual habit of leaving an unconfigurable rule out silently is
// right for an omission and wrong for a request.
// An unusable pattern is a 1, not a silent run that excludes nothing: a gate
// whose exclude glob has a typo would otherwise judge the tree it was told to
// leave alone and report offences nobody asked about.
#[test]
fn run_with_an_unusable_exclude_pattern_is_an_error() {
    // Arrange & Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        THIS_CRATE,
        "--exclude",
        "fixture/[",
    ]));

    // Assert
    assert!(outcome.is_err());
}

#[test]
fn run_with_the_header_rule_selected_but_no_header_file_is_an_error() {
    // Arrange
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--rule",
        "header",
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

// Without --header-file the header rule cannot hold, but the structure rule
// needs nothing and still does. A tool that reported "all rules satisfied"
// having checked nothing would be worse than one that says so, which is why the
// registry leaves an unconfigurable rule out rather than registering it silent.
#[test]
fn run_without_a_header_file_still_applies_the_rules_that_need_no_configuration() {
    // Arrange
    let args = args_from(&["cargo-stern4rust", "--manifest-path", "Cargo.toml"]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_ok());
}
