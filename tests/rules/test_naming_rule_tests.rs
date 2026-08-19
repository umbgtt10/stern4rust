// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A test is named `<method>_<conditions>_<result>`.
//
// The name and nothing else. Earlier versions tried to verify that the leading
// part was the method actually under test -- through the body, through the test
// file's helpers, and against the mirrored source -- and all three produced
// confident wrong answers on correct code. `tested-public-api` answers that
// question from the declared entry points instead.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::test_naming_rule::TestNamingRule;
use stern4rust::source_file::SourceFile;

fn check(body: &str) -> Vec<Offence> {
    TestNamingRule::new().check(&SourceFile::new("tests/a_tests.rs", body))
}

#[test]
fn check_of_a_helper_function_reports_nothing() {
    // Arrange & Act
    let found = check("fn helper_of_a_kind() {\n    let _ = 1;\n}\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A name whose method part contains underscores still has three parts overall.
#[test]
fn check_of_a_long_name_reports_nothing() {
    // Arrange & Act
    let found = check(
        "#[test]\nfn suggested_file_of_a_multi_word_name_separates_them() {\n    let _ = 1;\n}\n",
    );

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn check_of_a_one_part_name_reports_it() {
    // Arrange & Act
    let found = check("#[test]\nfn works() {\n    let _ = 1;\n}\n");

    // Assert
    assert_eq!(found.len(), 1);
}

#[test]
fn check_of_a_source_file_reports_nothing() {
    // Arrange & Act
    let found =
        TestNamingRule::new().check(&SourceFile::new("src/a.rs", "#[test]\nfn works() {}\n"));

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// The shape the rule exists for: a method, a condition and a result.
#[test]
fn check_of_a_three_part_name_reports_nothing() {
    // Arrange & Act
    let found = check("#[test]\nfn find_of_nothing_returns_none() {\n    let _ = 1;\n}\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn check_of_a_two_part_name_reports_it() {
    // Arrange & Act
    let found = check("#[test]\nfn find_works() {\n    let _ = 1;\n}\n");

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "test-naming");
    assert_eq!(found[0].subject.as_deref(), Some("find_works"));
    assert!(found[0].description.contains("fewer than 3 parts"));
}

#[test]
fn check_of_an_unparseable_file_reports_nothing() {
    // Arrange & Act
    let found = check("fn broken( {\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn name_is_test_naming() {
    // Arrange & Act
    let name = TestNamingRule::new().name();

    // Assert
    assert_eq!(name, "test-naming");
}
