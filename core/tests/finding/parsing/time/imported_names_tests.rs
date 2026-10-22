// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a name a file uses comes from, read off its `use` items: a single import, a group, a rename, `self`
// in a group, and a glob, which may supply any name.

use stern4rust::finding::parsing::time::imported_names::ImportedNames;
use syn::parse_file;

fn names(source: &str) -> ImportedNames {
    ImportedNames::of(&parse_file(source).expect("parses").items)
}

fn written(path: &str) -> Vec<String> {
    path.split("::").map(String::from).collect()
}

#[test]
fn resolve_of_a_crate_path_keeps_it_as_written() {
    // Arrange
    let names = names("fn f() {}\n");

    // Act
    let candidates = names.resolve(&written("std::thread::sleep"));

    // Assert
    assert_eq!(candidates, ["std::thread::sleep"]);
}

#[test]
fn resolve_of_a_glob_import_tries_the_globbed_module() {
    // Arrange
    let names = names("use std::time::*;\n");

    // Act
    let candidates = names.resolve(&written("Instant::now"));

    // Assert
    assert!(
        candidates.contains(&String::from("std::time::Instant::now")),
        "{candidates:?}"
    );
}

#[test]
fn resolve_of_a_grouped_import_reads_each_member() {
    // Arrange
    let names = names("use std::time::{Duration, Instant};\n");

    // Act
    let candidates = names.resolve(&written("Instant::now"));

    // Assert
    assert_eq!(candidates, ["std::time::Instant::now"]);
}

#[test]
fn resolve_of_a_module_imported_as_self_reads_through_it() {
    // Arrange
    let names = names("use std::thread::{self};\n");

    // Act
    let candidates = names.resolve(&written("thread::sleep"));

    // Assert
    assert_eq!(candidates, ["std::thread::sleep"]);
}

#[test]
fn resolve_of_a_renamed_import_reads_the_original() {
    // Arrange
    let names = names("use std::thread::sleep as nap;\n");

    // Act
    let candidates = names.resolve(&written("nap"));

    // Assert
    assert_eq!(candidates, ["std::thread::sleep"]);
}

#[test]
fn resolve_reads_an_import_inside_a_function_body() {
    // Arrange
    let names = names("fn wait() {\n    use tokio::time::sleep;\n}\n");

    // Act
    let candidates = names.resolve(&written("sleep"));

    // Assert
    assert_eq!(candidates, ["tokio::time::sleep"]);
}
