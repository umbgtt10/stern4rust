// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A file's `use` statements are its list of dependencies. A path written inline
// at the call site is a dependency that never reaches that list.
//
// One imported segment stays legal, and that is the rule's shape rather than a
// concession: `use std::fs` followed by `fs::read_to_string` states the route
// once and still names the module where the reader needs it.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::imported_paths_rule::ImportedPathsRule;
use stern4rust::source_file::SourceFile;

fn check(path: &str, body: &str) -> Vec<Offence> {
    ImportedPathsRule::new().check(&SourceFile::new(path, body))
}

#[test]
fn check_of_a_call_qualified_by_an_imported_module_finds_nothing() {
    // Arrange & Act
    let found = check(
        "src/a.rs",
        "use std::fs;\n\nfn outer() {\n    let _ = fs::read_to_string(\"\");\n}\n",
    );

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// The correction keeps `env` rather than proposing a bare `args()`.
#[test]
fn check_of_a_multi_segment_call_names_the_import_to_add() {
    // Arrange & Act
    let found = check(
        "src/a.rs",
        "fn outer() {\n    let _ = std::env::args();\n}\n",
    );

    // Assert
    assert_eq!(
        found[0].correction,
        "add `use std::env;` and call `env::args`"
    );
}

#[test]
fn check_of_a_path_qualified_call_reports_it() {
    // Arrange & Act
    let found = check(
        "src/a.rs",
        "fn outer() {\n    let _ = syn::parse_file(\"\");\n}\n",
    );

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 2);
    assert_eq!(found[0].rule, "imported-paths");
    assert_eq!(found[0].subject.as_deref(), Some("syn::parse_file"));
    assert_eq!(
        found[0].correction,
        "add `use syn::parse_file;` and call `parse_file`"
    );
}

// Unlike single-implemented-type, tests/ is not exempt: a test file has the
// same reader and the same list of dependencies at its top.
#[test]
fn check_of_a_test_file_reports_it_too() {
    // Arrange & Act
    let found = check(
        "tests/a_tests.rs",
        "#[test]\nfn t() {\n    let _ = syn::parse_file(\"\");\n}\n",
    );

    // Assert
    assert_eq!(found.len(), 1);
}

#[test]
fn check_of_an_unparseable_file_finds_nothing() {
    // Arrange & Act
    let found = check("src/a.rs", "fn broken( {\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// Every call, not the first one in the file: the report is meant to be worked
// through in one pass.
#[test]
fn check_reports_every_offending_call() {
    // Arrange & Act
    let found = check(
        "src/a.rs",
        "fn outer() {\n    let _ = syn::parse_file(\"\");\n    let _ = std::env::args();\n}\n",
    );

    // Assert
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].line, 2);
    assert_eq!(found[1].line, 3);
}

#[test]
fn name_is_imported_paths() {
    // Arrange & Act
    let name = ImportedPathsRule::new().name();

    // Assert
    assert_eq!(name, "imported-paths");
}
