// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Calls reached through a path the file never imported.
//
// A call is fine when it is unqualified, when it is qualified by a type, or when
// it is qualified by exactly one segment this file brought into scope with a
// `use`. Everything else is a path standing in for an import: `syn::parse_file`
// works without saying where `syn` came from, and `std::env::args` spells out a
// route that an import states once at the top.
//
// The distinction is between `use std::fs;` followed by `fs::read_to_string` --
// one imported segment, and idiomatic -- and `syn::parse_file` with nothing
// importing `syn` at all.

use stern4rust::finding::parsing::qualified_call_finder::QualifiedCallFinder;
use stern4rust::source_file::SourceFile;

fn found(body: &str) -> Vec<stern4rust::finding::model::qualified_call::QualifiedCall> {
    QualifiedCallFinder::find(&SourceFile::new("src/subject.rs", body)).expect("parses")
}

fn paths(body: &str) -> Vec<String> {
    found(body).into_iter().map(|call| call.path).collect()
}

// A call inside a function body is still a call.
#[test]
fn find_descends_into_function_bodies() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    let _ = syn::parse_file(\"\");\n}\n");

    // Assert
    assert_eq!(paths, ["syn::parse_file"]);
}

// A macro is not a function, and its path obeys different rules.
#[test]
fn find_ignores_a_macro_invocation() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    let _ = serde_json::json!({});\n}\n");

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

// A module whose name merely starts like a primitive is still a module.
#[test]
fn find_of_a_call_qualified_by_a_module_named_like_a_primitive_returns_the_path() {
    // Arrange & Act
    let paths = paths(
        "fn outer() {
    let _ = u64_helpers::decode([0; 8]);
}
",
    );

    // Assert
    assert_eq!(paths, ["u64_helpers::decode"]);
}

#[test]
fn find_of_a_call_qualified_by_a_non_integer_primitive_returns_nothing() {
    // Arrange & Act
    let paths: Vec<String> = ["f32", "f64", "bool", "char", "str"]
        .iter()
        .flat_map(|name| {
            paths(&format!(
                "fn outer() {{
    let _ = {name}::from_str(\"\");
}}
"
            ))
        })
        .collect();

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

// A primitive is a type, and the case convention cannot see that: `u64` opens
// lowercase, so it read as a module and the correction offered was
// `use u64::from_le_bytes;` -- which is not Rust. The names are fixed by the
// language rather than guessed at, so this one lowercase set can be known.
#[test]
fn find_of_a_call_qualified_by_a_primitive_type_returns_nothing() {
    // Arrange & Act
    let paths = paths(
        "fn outer() {
    let _ = u64::from_le_bytes([0; 8]);
}
",
    );

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

// One imported segment is the idiomatic form and states where the function
// came from at the call site.
#[test]
fn find_of_a_call_qualified_by_an_imported_module_returns_nothing() {
    // Arrange & Act
    let paths = paths("use std::fs;\n\nfn outer() {\n    let _ = fs::read_to_string(\"\");\n}\n");

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

#[test]
fn find_of_a_call_qualified_by_an_unimported_crate_returns_it() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    let _ = syn::parse_file(\"\");\n}\n");

    // Assert
    assert_eq!(paths, ["syn::parse_file"]);
}

#[test]
fn find_of_a_call_qualified_by_each_integer_primitive_returns_nothing() {
    // Arrange & Act
    let paths: Vec<String> = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    ]
    .iter()
    .flat_map(|name| {
        paths(&format!(
            "fn outer() {{
    let _ = {name}::from_le_bytes([0; 8]);
}}
"
        ))
    })
    .collect();

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

#[test]
fn find_of_a_crate_rooted_call_returns_it() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    crate::helpers::assist();\n}\n");

    // Assert
    assert_eq!(paths, ["crate::helpers::assist"]);
}

#[test]
fn find_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = QualifiedCallFinder::find(&SourceFile::new("src/a.rs", "fn broken( {\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn find_of_a_grouped_import_covers_each_name() {
    // Arrange & Act
    let paths =
        paths("use std::{fs, io};\n\nfn outer() {\n    let _ = fs::read_to_string(\"\");\n}\n");

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

// Two segments is a route the file spells out at every call site rather than
// stating once at the top.
#[test]
fn find_of_a_multi_segment_call_returns_it() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    let _ = std::env::args();\n}\n");

    // Assert
    assert_eq!(paths, ["std::env::args"]);
}

#[test]
fn find_of_a_renamed_import_uses_the_new_name() {
    // Arrange & Act
    let paths = paths(
        "use std::fs as filesystem;\n\nfn outer() {\n    let _ = filesystem::read_to_string(\"\");\n}\n",
    );

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

#[test]
fn find_of_a_self_qualified_call_returns_nothing() {
    // Arrange & Act
    let paths = paths("struct A;\nimpl A {\n    fn outer() {\n        Self::inner();\n    }\n}\n");

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

// A type qualifier is not a path standing in for an import: the type itself was
// imported, and Widget::new says which type is being constructed.
#[test]
fn find_of_a_type_qualified_call_returns_nothing() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    let _ = Widget::new();\n}\n");

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

#[test]
fn find_of_an_unqualified_call_returns_nothing() {
    // Arrange & Act
    let paths = paths("fn outer() {\n    helper();\n}\n");

    // Assert
    assert!(paths.is_empty(), "expected none, got {paths:?}");
}

#[test]
fn find_reports_the_line_of_the_call() {
    // Arrange & Act
    let found = found("fn outer() {\n\n    let _ = syn::parse_file(\"\");\n}\n");

    // Assert
    assert_eq!(found[0].line, 3);
}
