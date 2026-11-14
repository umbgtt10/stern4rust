// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
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
fn call_of_a_five_segment_path_keeps_its_last_two() {
    // Arrange & Act
    let call = QualifiedCall::new("alpha::beta::gamma::delta::run", 1).call();

    // Assert
    assert_eq!(call, "delta::run");
}

// A macro is called with its bang, kept on the name the correction leaves.
#[test]
fn call_of_a_macro_ends_in_its_bang() {
    // Arrange & Act
    let calls = [
        QualifiedCall::macro_invocation("std::vec", 1).call(),
        QualifiedCall::macro_invocation("core::arch::global_asm", 1).call(),
    ];

    // Assert
    assert_eq!(calls, ["vec!", "arch::global_asm!"]);
}

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

// `use syn;` would be legal and useless: the call site would be unchanged.
// The macro's own name is imported as a function's is, and with it
// whatever else the path names -- `use std::vec;` brings the module too.
#[test]
fn import_of_a_macro_is_its_path_split_as_a_function_s() {
    // Arrange & Act
    let imports = [
        QualifiedCall::macro_invocation("std::vec", 1).import(),
        QualifiedCall::macro_invocation("core::arch::global_asm", 1).import(),
    ];

    // Assert
    assert_eq!(imports, ["std::vec", "core::arch"]);
}

#[test]
fn import_of_a_three_segment_path_stops_before_the_function() {
    // Arrange & Act
    let import = QualifiedCall::new("std::env::args", 1).import();

    // Assert
    assert_eq!(import, "std::env");
}

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
    assert_eq!(call.path(), "std::env::args");
    assert_eq!(call.line(), 42);
}
