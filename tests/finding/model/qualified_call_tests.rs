// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a path-qualified call turns into once it is repaired.
//
// Every such path can be fixed by importing enough of it that at most one
// segment is left at the call site, and the two shapes want different splits.
// `syn::parse_file` has no qualifier worth keeping, so all of it is imported.
// `std::env::args` keeps `env`, because a bare `args()` says less than
// `env::args()` does.

use stern4rust::finding::model::qualified_call::QualifiedCall;

#[test]
fn call_of_a_three_segment_path_keeps_the_module_qualifier() {
    // Arrange & Act
    let call = QualifiedCall::new("std::env::args", 1).call();

    // Assert
    assert_eq!(call, "env::args");
}

#[test]
fn call_of_a_two_segment_path_is_the_function_alone() {
    // Arrange & Act
    let call = QualifiedCall::new("syn::parse_file", 1).call();

    // Assert
    assert_eq!(call, "parse_file");
}

#[test]
fn import_of_a_three_segment_path_stops_before_the_function() {
    // Arrange & Act
    let import = QualifiedCall::new("std::env::args", 1).import();

    // Assert
    assert_eq!(import, "std::env");
}

// `use syn;` would be legal and useless: the call site would be unchanged.
#[test]
fn import_of_a_two_segment_path_is_the_whole_path() {
    // Arrange & Act
    let import = QualifiedCall::new("syn::parse_file", 1).import();

    // Assert
    assert_eq!(import, "syn::parse_file");
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let call = QualifiedCall::new("std::env::args", 42);

    // Assert
    assert_eq!(call.path, "std::env::args");
    assert_eq!(call.line, 42);
}
