// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use syn::parse_file;
use xtask::boundary::boundary_rule::BoundaryRule;
use xtask::boundary::technology_use_collector::TechnologyUseCollector;

fn collected(source: &str) -> Vec<String> {
    let syntax = parse_file(source).expect("valid Rust");
    TechnologyUseCollector::collect(&syntax, &rule())
}

fn rule() -> BoundaryRule {
    BoundaryRule {
        package: String::from("core"),
        allowed_crates: vec![],
        crate_root: PathBuf::from("/w/core/src/lib.rs"),
        required_attributes: vec![],
        source_dirs: vec![PathBuf::from("/w/core/src")],
        forbidden_modules: vec![String::from("std::fs"), String::from("std::process")],
        forbidden_macros: vec![String::from("println")],
        forbidden_methods: vec![String::from("exists")],
    }
}

#[test]
fn collect_a_forbidden_method_call_names_it() {
    // Arrange & Act
    let found = collected("fn f(p: &Path) -> bool { p.exists() }");

    // Assert
    assert_eq!(found, vec![String::from(".exists()")]);
}

#[test]
fn collect_a_fully_qualified_forbidden_type_names_its_module() {
    // Arrange & Act
    let found = collected("fn f() { let _ = std::process::Command::new(\"cargo\"); }");

    // Assert
    assert_eq!(found, vec![String::from("std::process")]);
}

#[test]
fn collect_a_grouped_import_of_a_forbidden_module_names_it_once() {
    // Arrange & Act
    let found = collected("use std::{fs, fs::read_to_string, path::Path};");

    // Assert
    assert_eq!(found, vec![String::from("std::fs")]);
}

#[test]
fn collect_a_printing_macro_names_it() {
    // Arrange & Act
    let found = collected("fn f() { println!(\"{}\", 1); }");

    // Assert
    assert_eq!(found, vec![String::from("println!")]);
}

#[test]
fn collect_a_string_literal_mentioning_a_forbidden_module_finds_nothing() {
    // Arrange & Act
    let found = collected("const SOURCE: &str = \"use std::fs; fn f() { println!(); }\";");

    // Assert
    assert!(found.is_empty());
}

#[test]
fn collect_an_allowed_standard_module_finds_nothing() {
    // Arrange & Act
    let found =
        collected("use std::fmt::Write;\nuse std::path::PathBuf;\nuse std::collections::BTreeMap;");

    // Assert
    assert!(found.is_empty());
}

#[test]
fn collect_an_extern_crate_std_names_it() {
    // Arrange & Act
    let found = collected("extern crate alloc;\nextern crate std;");

    // Assert
    assert_eq!(found, vec![String::from("extern crate std")]);
}
