// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every public entry point is called by at least one test.
//
// The question `test-naming` gave up on, asked from the other end. Starting from
// the declared entry points needs no guess about intent: a `pub fn` either
// appears at a call site under `tests/` or it does not. It also sidesteps
// derives, which defeated every attempt to work back from a test's name.
//
// Matched on name and arity. Types and parameter order are not checked and
// cannot be without type inference, so the rule under-reports rather than
// accusing tested code.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::tested_public_api_rule::TestedPublicApiRule;
use stern4rust::source_file::SourceFile;

fn check(files: &[(&str, &str)]) -> Vec<Offence> {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, body)| SourceFile::new(path, body))
        .collect();
    TestedPublicApiRule::new().check_workspace(&sources)
}

// A call in src is not a test exercising it.
#[test]
fn check_workspace_of_a_call_from_source_only_reports_it() {
    // Arrange & Act
    let found = check(&[
        ("src/a.rs", "pub fn run(one: usize) {}\n"),
        ("src/b.rs", "pub fn caller() { run(1); }\n"),
        (
            "tests/b_tests.rs",
            "#[test]\nfn caller_of_nothing_works() { caller(); }\n",
        ),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].subject.as_deref(), Some("run/1"));
}

// The call most tests care about lives inside an assertion macro.
#[test]
fn check_workspace_of_a_call_inside_an_assert_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        (
            "src/a.rs",
            "pub struct A;\nimpl A {\n    pub fn is_stale(&self) -> bool { true }\n}\n",
        ),
        (
            "tests/a_tests.rs",
            "#[test]\nfn is_stale_of_a_thing_is_true() { assert!(a.is_stale()); }\n",
        ),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// Arity is what separates two entry points a bare name would confuse.
#[test]
fn check_workspace_of_a_call_with_the_wrong_arity_reports_it() {
    // Arrange & Act
    let found = check(&[
        ("src/a.rs", "pub fn run(one: usize, two: usize) {}\n"),
        (
            "tests/a_tests.rs",
            "#[test]\nfn run_of_one_works() { run(1); }\n",
        ),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].subject.as_deref(), Some("run/2"));
}

#[test]
fn check_workspace_of_a_called_entry_point_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/a.rs", "pub fn run(one: usize) {}\n"),
        (
            "tests/a_tests.rs",
            "#[test]\nfn run_of_one_works() { run(1); }\n",
        ),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn check_workspace_of_a_private_function_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/a.rs", "fn hidden(one: usize) {}\n"),
        (
            "tests/a_tests.rs",
            "#[test]\nfn other_of_nothing_works() { let _ = 1; }\n",
        ),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// The failure the rule exists for.
#[test]
fn check_workspace_of_an_uncalled_entry_point_reports_it() {
    // Arrange & Act
    let found = check(&[
        ("src/a.rs", "pub fn run(one: usize) {}\n"),
        (
            "tests/a_tests.rs",
            "#[test]\nfn other_of_nothing_works() { let _ = 1; }\n",
        ),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "tested-public-api");
    assert_eq!(found[0].file, "src/a.rs");
    assert_eq!(found[0].subject.as_deref(), Some("run/1"));
}

#[test]
fn name_is_tested_public_api() {
    // Arrange & Act
    let name = TestedPublicApiRule::new().name();

    // Assert
    assert_eq!(name, "tested-public-api");
}
