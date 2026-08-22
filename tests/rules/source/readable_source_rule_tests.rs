// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The rule that exists because silence is indistinguishable from success.
//
// Every other rule that parses gives up quietly on source it cannot read,
// trusting rustc to say so more clearly. That is right for a file somebody is
// actively editing and wrong for a file nobody is looking at: a corrupted file
// disappears from the report entirely and the package looks cleaner than it is.
// This happened during development -- a test file became a run of NUL bytes and
// the tool reported one fewer offence than the tree contained, with no
// indication anything had been skipped.

use stern4rust::rule::Rule;
use stern4rust::rules::source::readable_source_rule::ReadableSourceRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "readable-source";

fn check(contents: &str) -> Vec<stern4rust::reporting::offence::Offence> {
    ReadableSourceRule::new().check(&SourceFile::new("src/subject.rs", contents))
}

// The case that motivated the rule, exactly as it arrived: a file whose bytes
// are all NUL reads as empty in a diff and as clean in every parsing rule.
#[test]
fn check_a_file_of_nul_bytes_reports_it() {
    // Arrange
    let corrupted = "\0".repeat(41);

    // Act
    let offences = check(&corrupted);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
}

#[test]
fn check_a_file_that_does_not_parse_reports_it() {
    // Arrange & Act
    let offences = check("fn broken( {\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("does not parse"),
        "got {}",
        offences[0].description
    );
}

#[test]
fn check_a_file_that_parses_reports_nothing() {
    // Arrange & Act
    let offences = check("pub struct Subject;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// An empty file is valid Rust. It may break other rules; it does not break this
// one.
#[test]
fn check_an_empty_file_reports_nothing() {
    // Arrange & Act
    let offences = check("");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_names_the_file_it_judged() {
    // Arrange & Act
    let offences = check("fn broken( {\n");

    // Assert
    assert_eq!(offences[0].file, "src/subject.rs");
}

// The line rustc would also point at, so the two reports agree rather than
// sending a reader to two different places.
#[test]
fn check_reports_the_line_the_parse_failed_on() {
    // Arrange & Act
    let offences = check("pub struct Fine;\n\npub struct Also;\n\nfn broken( {\n");

    // Assert
    assert!(offences[0].line >= 5, "got line {}", offences[0].line);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = ReadableSourceRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
