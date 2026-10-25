// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a method built, named the way an offence names it: a type for its
// constructor or a unit value, a function by its path, a chain by its calls,
// each without its arguments, and the wrappers that only carry it left off.

use stern4rust::finding::parsing::dependencies::built_text::BuiltText;
use syn::Expr;
use syn::parse_str;

fn text(value: &str) -> String {
    let built: Expr = parse_str(value).expect("parses");
    BuiltText::of(&built)
}

#[test]
fn of_a_boxed_constructor_call_names_the_type() {
    // Arrange & Act
    let found = text("Box::new(StdoutReporter::new())");

    // Assert
    assert_eq!(found, "StdoutReporter");
}

#[test]
fn of_a_function_call_names_its_path() {
    // Arrange & Act
    let found = text("Registry::current()");

    // Assert
    assert_eq!(found, "Registry::current()");
}

#[test]
fn of_a_macro_names_it_without_its_tokens() {
    // Arrange & Act
    let found = text("vec![Box::new(A::new())]");

    // Assert
    assert_eq!(found, "vec![..]");
}

#[test]
fn of_a_method_chain_names_each_call_without_its_arguments() {
    // Arrange & Act
    let found = text("Self::all(config).into_iter().filter(|rule| rule.on()).collect()");

    // Assert
    assert_eq!(found, "Self::all().into_iter().filter().collect()");
}

#[test]
fn of_a_struct_literal_names_its_type() {
    // Arrange & Act
    let found = text("Some(Store { size: 1 })");

    // Assert
    assert_eq!(found, "Store");
}

#[test]
fn of_a_unit_value_names_it() {
    // Arrange & Act
    let found = text("Arc::new(NoOpCoordinator)");

    // Assert
    assert_eq!(found, "NoOpCoordinator");
}

#[test]
fn of_anything_else_is_its_source_without_spaces() {
    // Arrange & Act
    let found = text("if ready { a } else { b }");

    // Assert
    assert_eq!(found, "ifready{a}else{b}");
}
