// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A library's public functions fail with an error type of their own. A `pub fn`
// of a library target returning a `Result` whose error is a `String`, a `&str` or
// a boxed `dyn ...Error` is reported at its name; a binary's files, trait impls
// and tests/ are left alone.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::typed_errors_rule::TypedErrorsRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "typed-errors";

fn check_workspace(files: &[(&str, &str)]) -> Vec<Offence> {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(file, contents)| SourceFile::new(file, contents))
        .collect();
    TypedErrorsRule::new().check_workspace(&files)
}

fn library(body: &str) -> Vec<Offence> {
    check_workspace(&[("src/lib.rs", "pub mod store;\n"), ("src/store.rs", body)])
}

// This rule answers the tree question only, and says so rather than inheriting
// a silence nobody chose.
#[test]
fn check_of_a_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/lib.rs",
        "pub fn read() -> Result<u8, String> {\n    Ok(0)\n}\n",
    );

    // Act
    let offences = TypedErrorsRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_binary_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/lib.rs", ""),
        (
            "src/main.rs",
            "mod cli;\n\npub fn run() -> Result<(), String> {\n    Ok(())\n}\n\nfn main() {}\n",
        ),
        (
            "src/cli.rs",
            "pub fn parse() -> Result<u8, String> {\n    Ok(0)\n}\n",
        ),
        (
            "src/bin/tool.rs",
            "pub fn go() -> Result<(), String> {\n    Ok(())\n}\n\nfn main() {}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_borrowed_str_reports_it() {
    // Arrange & Act
    let offences = library("pub fn read<'a>() -> Result<u8, &'a str> {\n    Ok(0)\n}\n");

    // Assert
    assert_eq!(offences[0].description(), "`read` fails with a `&str`");
}

#[test]
fn check_workspace_of_a_boxed_error_reports_it() {
    // Arrange & Act
    let offences = library(
        "pub fn read() -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {\n    Ok(0)\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`read` fails with a `Box<dyn std::error::Error + Send + Sync>`"
    );
}

#[test]
fn check_workspace_of_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = library("pub fn read() -> Result<u8, String> {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_private_function_reports_nothing() {
    // Arrange & Act
    let offences = library(
        "fn read() -> Result<u8, String> {\n    Ok(0)\n}\n\npub(crate) fn peek() -> Result<u8, String> {\n    Ok(0)\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_pub_trait_reports_it_at_its_declaration() {
    // Arrange & Act
    let offences = library(
        "pub trait Gate {\n    fn run(&self) -> Result<(), String>;\n}\n\npub struct Tests;\n\nimpl Gate for Tests {\n    \
         fn run(&self) -> Result<(), String> {\n        Ok(())\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line(), 2);
    assert_eq!(offences[0].subject(), Some("run"));
}

#[test]
fn check_workspace_of_a_string_error_reports_it_at_the_function() {
    // Arrange & Act
    let offences = library("pub fn read() -> Result<u8, String> {\n    Ok(0)\n}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), "src/store.rs");
    assert_eq!(offences[0].line(), 1);
    assert_eq!(offences[0].subject(), Some("read"));
    assert_eq!(offences[0].description(), "`read` fails with a `String`");
    assert_eq!(
        offences[0].correction(),
        "give the crate an error enum its callers and tests can match"
    );
}

#[test]
fn check_workspace_of_a_typed_error_reports_nothing() {
    // Arrange & Act
    let offences = library(
        "pub enum ReadError {\n    Missing,\n}\n\npub fn read() -> Result<u8, ReadError> {\n    Ok(0)\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_tests_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/lib.rs", ""),
        (
            "tests/store_tests.rs",
            "pub fn read() -> Result<u8, String> {\n    Ok(0)\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = TypedErrorsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = TypedErrorsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = TypedErrorsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
