// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Turning a file into the items the structure rule reasons about.
//
// Two things here are easy to get wrong and invisible when wrong. Plain `//`
// comments never reach the syntax tree, so a comment introducing an item has to
// be folded back into that item by hand -- otherwise the blank line before the
// comment reads as a gap and every commented test is an offence. And an `impl`
// block has no name of its own, so it has to borrow the name of the type it
// implements or it sorts as an empty string and drifts to the top.

use stern4rust::finding::section::Section;
use stern4rust::finding::test_file_parser::TestFileParser;
use stern4rust::source_file::SourceFile;

fn items(contents: &str) -> Vec<stern4rust::finding::test_file_item::TestFileItem> {
    TestFileParser::parse(&SourceFile::new("tests/a_tests.rs", contents)).expect("parses")
}

fn sections(contents: &str) -> Vec<Section> {
    items(contents)
        .into_iter()
        .map(|item| item.section)
        .collect()
}

#[test]
fn parse_classifies_a_const_and_a_static_as_constants() {
    // Arrange & Act
    let found = sections("const A: usize = 1;\nstatic B: usize = 2;\n");

    // Assert
    assert_eq!(found, [Section::Constants, Section::Constants]);
}

// Matching the last path segment rather than the whole path is what keeps this
// from having to enumerate test frameworks.
#[test]
fn parse_classifies_a_function_carrying_a_qualified_test_attribute_as_a_test() {
    // Arrange & Act
    let found = sections("#[tokio::test]\nasync fn a() {}\n");

    // Assert
    assert_eq!(found, [Section::Tests]);
}

#[test]
fn parse_classifies_a_function_carrying_the_test_attribute_as_a_test() {
    // Arrange & Act
    let found = sections("#[test]\nfn a() {}\n");

    // Assert
    assert_eq!(found, [Section::Tests]);
}

#[test]
fn parse_classifies_a_plain_function_as_a_helper() {
    // Arrange & Act
    let found = sections("fn a() {}\n");

    // Assert
    assert_eq!(found, [Section::Helpers]);
}

// Helpers are whatever is left, which is what keeps the set closed.
#[test]
fn parse_classifies_a_struct_an_impl_and_a_type_alias_as_helpers() {
    // Arrange & Act
    let found = sections("struct A;\nimpl A {}\ntype B = A;\n");

    // Assert
    assert_eq!(
        found,
        [Section::Helpers, Section::Helpers, Section::Helpers]
    );
}

#[test]
fn parse_classifies_a_use_item_as_an_import() {
    // Arrange & Act
    let found = sections("use alpha::One;\n");

    // Assert
    assert_eq!(found, [Section::Imports]);
}

// Without this the blank line above the comment reads as the gap, and every
// commented test in the workspace becomes an offence.
#[test]
fn parse_folds_a_leading_comment_into_the_item_below_it() {
    // Arrange & Act
    let found = items("fn a() {}\n\n// Why this one matters.\n#[test]\nfn b() {}\n");

    // Assert
    assert_eq!(found[1].first_line, 3);
}

#[test]
fn parse_folds_several_leading_comment_lines_into_the_item_below_them() {
    // Arrange & Act
    let found = items("fn a() {}\n\n// One.\n// Two.\n#[test]\nfn b() {}\n");

    // Assert
    assert_eq!(found[1].first_line, 3);
}

#[test]
fn parse_names_a_constant_by_its_identifier() {
    // Arrange & Act
    let found = items("const SOME_LIMIT: usize = 1;\n");

    // Assert
    assert_eq!(found[0].name, "SOME_LIMIT");
}

#[test]
fn parse_names_a_function_by_its_identifier() {
    // Arrange & Act
    let found = items("fn helper_name() {}\n");

    // Assert
    assert_eq!(found[0].name, "helper_name");
}

// An impl block has no name of its own, so it borrows the type's and sits beside
// the struct rather than drifting to the top of the section.
#[test]
fn parse_names_an_impl_block_after_the_type_it_implements() {
    // Arrange & Act
    let found = items("impl Recorder {}\n");

    // Assert
    assert_eq!(found[0].name, "Recorder");
}

#[test]
fn parse_names_an_import_as_it_was_written() {
    // Arrange & Act
    let found = items("use alpha::One;\n");

    // Assert
    assert!(found[0].name.contains("alpha::One"));
}

// rustc will say so far more clearly, and guessing at a shape from broken source
// would pile noise on top of a compile error.
#[test]
fn parse_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = TestFileParser::parse(&SourceFile::new("tests/a_tests.rs", "fn broken( {\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn parse_of_an_empty_file_returns_no_items() {
    // Arrange & Act
    let found = items("");

    // Assert
    assert!(found.is_empty());
}

#[test]
fn parse_reports_the_line_an_item_ends_on() {
    // Arrange & Act
    let found = items("fn a() {\n    let _ = 1;\n}\n");

    // Assert
    assert_eq!(found[0].last_line, 3);
}

#[test]
fn parse_reports_the_line_an_item_starts_on() {
    // Arrange & Act
    let found = items("\n\nfn a() {}\n");

    // Assert
    assert_eq!(found[0].first_line, 3);
}

// The attribute is part of the test, so the block starts there rather than at
// the `fn`.
#[test]
fn parse_starts_a_test_block_at_its_attribute() {
    // Arrange & Act
    let found = items("#[test]\nfn a() {}\n");

    // Assert
    assert_eq!(found[0].first_line, 1);
}
