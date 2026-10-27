// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::boundary::source_files_tests::InMemorySourceFiles;
use crate::process::command_runner_tests::FakeCommandRunner;
use std::path::PathBuf;
use xtask::boundary::boundary_rule::BoundaryRule;
use xtask::boundary::crate_root_attributes::CrateRootAttributes;
use xtask::boundary::package_dependencies::PackageDependencies;
use xtask::gates::boundary_gate::BoundaryGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;

const CLEAN_METADATA: &str =
    r#"{"packages":[{"name":"core","dependencies":[{"name":"syn","kind":null}]}]}"#;
const CLEAN_ROOT: &str =
    "#![no_std]\n#![forbid(unsafe_code)]\n\nextern crate alloc;\n\npub mod a;\n";

fn rule() -> BoundaryRule {
    BoundaryRule {
        package: String::from("core"),
        allowed_crates: vec![String::from("syn")],
        crate_root: PathBuf::from("/w/core/src/lib.rs"),
        required_attributes: vec![String::from("no_std"), String::from("forbid(unsafe_code)")],
        source_dirs: vec![PathBuf::from("/w/core/src")],
        forbidden_modules: vec![String::from("std::fs")],
        forbidden_macros: vec![String::from("println")],
        forbidden_methods: vec![String::from("exists")],
    }
}

fn run_with(runner: &FakeCommandRunner, sources: &InMemorySourceFiles) -> Result<(), GateFailure> {
    let dependencies = PackageDependencies::new();
    let attributes = CrateRootAttributes::new();
    BoundaryGate::new(
        runner,
        sources,
        &dependencies,
        &attributes,
        String::from("/w/Cargo.toml"),
        rule(),
    )
    .run()
}

#[test]
fn label_names_the_technology_free_core() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let sources = InMemorySourceFiles::new();
    let dependencies = PackageDependencies::new();
    let attributes = CrateRootAttributes::new();
    let gate = BoundaryGate::new(
        &runner,
        &sources,
        &dependencies,
        &attributes,
        String::from("/w/Cargo.toml"),
        rule(),
    );

    // Act
    let label = gate.label();

    // Assert
    assert_eq!(label, "Technology-free core");
}

#[test]
fn run_over_a_clean_core_reads_the_metadata_and_passes() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CLEAN_METADATA);
    let sources = InMemorySourceFiles::new().with_file("/w/core/src/lib.rs", CLEAN_ROOT);

    // Act
    let result = run_with(&runner, &sources);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        runner.calls(),
        vec![vec![
            String::from("metadata"),
            String::from("--no-deps"),
            String::from("--format-version"),
            String::from("1"),
            String::from("--manifest-path"),
            String::from("/w/Cargo.toml"),
        ]]
    );
}

#[test]
fn run_with_a_crate_off_the_allow_list_a_missing_attribute_and_forbidden_uses_names_them_all() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(
        r#"{"packages":[{"name":"core","dependencies":[{"name":"walkdir","kind":"dev"}]}]}"#,
    );
    let sources = InMemorySourceFiles::new()
        .with_file(
            "/w/core/src/lib.rs",
            "#![forbid(unsafe_code)]\n\nextern crate std;\n",
        )
        .with_file("/w/core/src/a.rs", "use std::fs;")
        .with_file(
            "/w/core/src/b.rs",
            "fn f(p: &Path) { if p.exists() { println!(); } }",
        );

    // Act
    let result = run_with(&runner, &sources);

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Found(String::from(
            "core depends on walkdir, which its rule does not allow\n/w/core/src/lib.rs does not carry #![no_std]\n/w/core/src/a.rs uses std::fs\n/w/core/src/b.rs uses .exists(), println!\n/w/core/src/lib.rs uses extern crate std"
        )))
    );
}

#[test]
fn run_with_a_crate_root_that_cannot_be_read_fails() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CLEAN_METADATA);
    let sources = InMemorySourceFiles::new();

    // Act
    let result = run_with(&runner, &sources);

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Found(String::from(
            "could not read /w/core/src/lib.rs: not held"
        )))
    );
}

#[test]
fn run_with_a_source_directory_that_cannot_be_listed_fails() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CLEAN_METADATA);
    let sources = InMemorySourceFiles::new().with_file("/w/core/src/lib.rs", CLEAN_ROOT);
    let dependencies = PackageDependencies::new();
    let attributes = CrateRootAttributes::new();
    let gate = BoundaryGate::new(
        &runner,
        &sources,
        &dependencies,
        &attributes,
        String::from("/w/Cargo.toml"),
        BoundaryRule {
            source_dirs: vec![PathBuf::from("/w/core/tests")],
            ..rule()
        },
    );

    // Act
    let result = gate.run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Found(String::from(
            "could not read /w/core/tests: not held"
        )))
    );
}

#[test]
fn run_with_a_source_that_does_not_parse_names_it() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CLEAN_METADATA);
    let sources = InMemorySourceFiles::new()
        .with_file("/w/core/src/lib.rs", CLEAN_ROOT)
        .with_file("/w/core/src/broken.rs", "fn {");

    // Act
    let result = run_with(&runner, &sources);

    // Assert
    assert!(
        matches!(&result, Err(GateFailure::Found(text)) if text.starts_with("could not parse /w/core/src/broken.rs: ")),
        "{result:?}"
    );
}

#[test]
fn run_with_metadata_failing_reports_its_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_capturing_code(Some(101));
    let sources = InMemorySourceFiles::new().with_file("/w/core/src/lib.rs", CLEAN_ROOT);

    // Act
    let result = run_with(&runner, &sources);

    // Assert
    assert_eq!(result, Err(GateFailure::CouldNotRun(Some(101))));
}

#[test]
fn run_with_the_package_missing_from_the_metadata_fails() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(r#"{"packages":[]}"#);
    let sources = InMemorySourceFiles::new().with_file("/w/core/src/lib.rs", CLEAN_ROOT);

    // Act
    let result = run_with(&runner, &sources);

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Found(String::from(
            "core is not in the workspace"
        )))
    );
}
