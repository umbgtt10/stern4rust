// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The types a file both declares and gives behaviour to.
//
// Two halves have to meet for a type to count. It must be declared here, so an
// `impl Display for SomeoneElsesType` does not make this file that type's home.
// And it must carry at least one impl block, so a file may hold as many plain
// data declarations as it likes -- a handful of payload structs with no
// behaviour is one subject, not five.
//
// A trait impl counts as much as an inherent one. Both are behaviour, and a
// reader looking for what a type does opens the file either way.

use stern4rust::finding::parsing::implemented_type_finder::ImplementedTypeFinder;
use stern4rust::source_file::SourceFile;

fn found(body: &str) -> Vec<stern4rust::finding::model::implemented_type::ImplementedType> {
    ImplementedTypeFinder::find(&SourceFile::new("src/subject.rs", body)).expect("parses")
}

fn names(body: &str) -> Vec<String> {
    found(body).into_iter().map(|found| found.name).collect()
}

// A reader looking for what a type does opens the file either way.
#[test]
fn find_counts_a_trait_impl() {
    // Arrange & Act
    let names = names("pub struct A;\n\nimpl Display for A {}\n");

    // Assert
    assert_eq!(names, ["A"]);
}

#[test]
fn find_counts_a_type_once_however_many_impl_blocks_it_has() {
    // Arrange & Act
    let names =
        names("pub struct A;\n\nimpl A {}\n\nimpl Display for A {}\n\nimpl Debug for A {}\n");

    // Assert
    assert_eq!(names, ["A"]);
}

// A nested type with behaviour is still a second subject in the same file.
#[test]
fn find_descends_into_an_inline_module() {
    // Arrange & Act
    let names =
        names("pub struct A;\nimpl A {}\n\nmod detail {\n    pub struct B;\n    impl B {}\n}\n");

    // Assert
    assert_eq!(names, ["A", "B"]);
}

#[test]
fn find_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = ImplementedTypeFinder::find(&SourceFile::new("src/a.rs", "struct A {\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn find_of_a_struct_with_an_impl_returns_it() {
    // Arrange & Act
    let names = names("pub struct Widget;\n\nimpl Widget {\n    pub fn new() {}\n}\n");

    // Assert
    assert_eq!(names, ["Widget"]);
}

// Plain data is not a subject. A file may hold as many such declarations as it
// likes.
#[test]
fn find_of_a_type_without_an_impl_returns_nothing() {
    // Arrange & Act
    let names = names("pub struct A;\npub struct B;\npub enum C {}\n");

    // Assert
    assert!(names.is_empty(), "expected none, got {names:?}");
}

#[test]
fn find_of_an_enum_with_an_impl_returns_it() {
    // Arrange & Act
    let names = names("pub enum Verdict {\n    Clean,\n}\n\nimpl Verdict {}\n");

    // Assert
    assert_eq!(names, ["Verdict"]);
}

// Implementing a foreign trait for a foreign type does not make this file that
// type's home, so it is not a subject of this file.
#[test]
fn find_of_an_impl_for_a_type_declared_elsewhere_returns_nothing() {
    // Arrange & Act
    let names = names("impl Display for SomeoneElsesType {}\n");

    // Assert
    assert!(names.is_empty(), "expected none, got {names:?}");
}

#[test]
fn find_of_two_types_with_impls_returns_both() {
    // Arrange & Act
    let names = names("pub struct A;\nimpl A {}\n\npub struct B;\nimpl B {}\n");

    // Assert
    assert_eq!(names, ["A", "B"]);
}

// The declaration, not the impl block: that is the line a reader moves.
#[test]
fn find_reports_the_line_the_type_is_declared_on() {
    // Arrange & Act
    let found = found("pub struct A;\nimpl A {}\n\npub struct B;\nimpl B {}\n");

    // Assert
    assert_eq!(found[0].line, 1);
    assert_eq!(found[1].line, 4);
}
