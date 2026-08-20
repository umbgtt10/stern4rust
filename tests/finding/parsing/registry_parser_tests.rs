// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Finding what does not belong in a registry file, and saying what it is.
//
// The point of this parser is the naming. A registry offence that reports only
// "something here is not a declaration" is true of the whole file and actionable
// nowhere in it -- and when a file holds four strays it produces four rows that
// are byte-identical. Each stray has to arrive with its own line and its own
// name before the report is worth reading.

use stern4rust::finding::model::registry_item::RegistryItem;
use stern4rust::finding::model::registry_policy::RegistryPolicy;
use stern4rust::finding::parsing::registry_parser::RegistryParser;
use stern4rust::source_file::SourceFile;

fn labels(body: &str) -> Vec<String> {
    strays(body).into_iter().map(|stray| stray.label).collect()
}

fn strays(body: &str) -> Vec<RegistryItem> {
    RegistryParser::strays(
        &SourceFile::new("tests/all_tests.rs", body),
        RegistryPolicy::tests(),
    )
    .expect("parses")
}

#[test]
fn strays_names_a_constant_with_its_identifier() {
    // Arrange & Act
    let found = labels("const LIMIT: usize = 1;\n");

    // Assert
    assert_eq!(found, ["the constant `LIMIT`"]);
}

#[test]
fn strays_names_a_function_with_its_identifier() {
    // Arrange & Act
    let found = labels("fn helper() {}\n");

    // Assert
    assert_eq!(found, ["the function `helper`"]);
}

#[test]
fn strays_names_a_static_with_its_identifier() {
    // Arrange & Act
    let found = labels("static LIMIT: usize = 1;\n");

    // Assert
    assert_eq!(found, ["the static `LIMIT`"]);
}

#[test]
fn strays_names_a_struct_with_its_identifier() {
    // Arrange & Act
    let found = labels("struct Recorder;\n");

    // Assert
    assert_eq!(found, ["the struct `Recorder`"]);
}

#[test]
fn strays_names_a_type_alias_with_its_identifier() {
    // Arrange & Act
    let found = labels("type Pair = (usize, usize);\n");

    // Assert
    assert_eq!(found, ["the type alias `Pair`"]);
}

// An impl block has no identifier of its own, so it falls back to the line as
// written rather than being left as a bare "item".
#[test]
fn strays_names_an_impl_block_by_the_line_as_written() {
    // Arrange & Act
    let found = labels("struct Recorder;\nimpl Recorder {}\n");

    // Assert
    assert_eq!(found[1], "the impl block `impl Recorder {}`");
}

// An import has no identifier either, and the line as written is what a reader
// would search the file for.
#[test]
fn strays_names_an_import_by_the_line_as_written() {
    // Arrange & Act
    let found = labels("use std::fmt::Debug;\n");

    // Assert
    assert_eq!(found, ["the import `use std::fmt::Debug;`"]);
}

// The distinction the rule exists for: `mod name;` points at a file, `mod name
// { ... }` is code living in the one file a reader scans expecting a list.
#[test]
fn strays_names_an_inline_module_with_its_identifier() {
    // Arrange & Act
    let found = labels("mod inner { }\n");

    // Assert
    assert_eq!(found, ["the inline module `inner`"]);
}

// rustc will say so far more clearly, and guessing at a shape from broken source
// would pile noise on top of a compile error.
#[test]
fn strays_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = RegistryParser::strays(
        &SourceFile::new("tests/all_tests.rs", "mod broken {\n"),
        RegistryPolicy::tests(),
    );

    // Assert
    assert!(found.is_none());
}

// A declaration is a declaration whether or not it is `pub`. The rule exists to
// catch a file that is never compiled, and a private `mod name;` compiles it
// just as well.
#[test]
fn strays_of_a_private_mod_declaration_returns_nothing() {
    // Arrange & Act
    let found = strays("mod alpha_tests;\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn strays_of_a_registry_of_only_declarations_returns_nothing() {
    // Arrange & Act
    let found = strays("pub mod alpha_tests;\npub mod beta_tests;\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn strays_of_an_empty_file_returns_nothing() {
    // Arrange & Act
    let found = strays("");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// The defect this parser was written for. Four strays used to produce four rows
// that were byte-identical, every one of them pointing at line 1.
#[test]
fn strays_reports_each_stray_at_its_own_line() {
    // Arrange
    let body =
        "use std::fmt;\n\nconst LIMIT: usize = 1;\n\nfn helper() {}\n\npub mod alpha_tests;\n";

    // Act
    let found = strays(body);

    // Assert
    assert_eq!(found.len(), 3);
    assert_eq!(
        found.iter().map(|stray| stray.line).collect::<Vec<usize>>(),
        [1, 3, 5]
    );
}
