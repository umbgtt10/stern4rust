// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The module names a registry declares.
//
// `pub` is deliberately not required. `module-registry` demands it in `src/`
// and this is a different question: a private `mod name;` compiles that file
// just as well, and being compiled is the whole concern. An inline
// `mod name { ... }` declares no file, so it cannot be what reaches one.

use stern4rust::finding::parsing::modules::module_declaration_finder::ModuleDeclarationFinder;
use stern4rust::source_file::SourceFile;

fn found(contents: &str) -> Vec<String> {
    ModuleDeclarationFinder::find(&SourceFile::new("src/lib.rs", contents)).expect("parses")
}

#[test]
fn directory_of_a_named_file_is_the_folder_beneath_it() {
    // Arrange & Act
    let directory = ModuleDeclarationFinder::directory_of("src/platform.rs");

    // Assert
    assert_eq!(directory, "src/platform/");
}

#[test]
fn directory_of_a_registry_is_its_own_folder() {
    // Arrange & Act
    let directories: Vec<String> = ["src/lib.rs", "src/main.rs", "src/store/mod.rs"]
        .into_iter()
        .map(ModuleDeclarationFinder::directory_of)
        .collect();

    // Assert
    assert_eq!(directories, ["src/", "src/", "src/store/"]);
}

#[test]
fn find_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = ModuleDeclarationFinder::find(&SourceFile::new("src/lib.rs", "pub mod ( broken\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn find_of_a_private_declaration_counts_it() {
    // Arrange & Act
    let names = found("mod widget;\n");

    // Assert
    assert_eq!(names, ["widget"]);
}

#[test]
fn find_of_a_public_declaration_counts_it() {
    // Arrange & Act
    let names = found("pub mod widget;\n");

    // Assert
    assert_eq!(names, ["widget"]);
}

// A registry holding nothing but a header declares nothing, which is a fact
// rather than a failure to parse.
#[test]
fn find_of_a_registry_with_no_declarations_is_empty() {
    // Arrange & Act
    let names = found("// just a header\n");

    // Assert
    assert!(names.is_empty(), "expected none, got {names:?}");
}

// It declares no file, so it cannot be what reaches one.
#[test]
fn find_of_an_inline_module_does_not_count_it() {
    // Arrange & Act
    let names = found("pub mod widget { pub struct W; }\n");

    // Assert
    assert!(names.is_empty(), "expected none, got {names:?}");
}

#[test]
fn find_of_several_declarations_keeps_the_order_they_appear_in() {
    // Arrange & Act
    let names = found("pub mod beta;\npub mod alpha;\n");

    // Assert
    assert_eq!(names, ["beta", "alpha"]);
}
