// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The standard library's macros whose arguments are expressions, read as the
// code they are; every other macro left to its own expansion.

use quote::ToTokens;
use stern4rust::finding::parsing::shapes::standard_macro::StandardMacro;
use syn::Expr;
use syn::ExprMacro;
use syn::Macro;
use syn::parse_str;

fn listed(expressions: Option<Vec<Expr>>) -> Option<Vec<String>> {
    expressions.map(|expressions| {
        expressions
            .iter()
            .map(|expression| expression.to_token_stream().to_string())
            .collect()
    })
}

fn macro_of(text: &str) -> Macro {
    parse_str::<ExprMacro>(text).expect("a macro").mac
}

fn rendered(text: &str) -> Option<Vec<String>> {
    let node = macro_of(text);
    listed(
        StandardMacro::of(&node)
            .expect("a standard macro")
            .arguments(),
    )
}

fn rendered_expressions(text: &str) -> Option<Vec<String>> {
    let node = macro_of(text);
    listed(
        StandardMacro::of(&node)
            .expect("a standard macro")
            .expressions(),
    )
}

#[test]
fn arguments_of_a_body_of_expressions_returns_each_of_them() {
    // Arrange & Act
    let arguments = rendered("assert_eq!(left, right, \"differ\")");

    // Assert
    assert_eq!(
        arguments,
        Some(vec![
            String::from("left"),
            String::from("right"),
            String::from("\"differ\"")
        ])
    );
}

// A pattern binding by reference is no expression: nothing is guessed at.
#[test]
fn arguments_of_a_body_that_is_no_list_of_expressions_returns_none() {
    // Arrange & Act
    let arguments = rendered("matches!(value, ref inner)");

    // Assert
    assert_eq!(arguments, None);
}

// `vec![element; count]` is the one standard body that is not a list.
#[test]
fn arguments_of_a_repeated_element_returns_the_element_and_its_count() {
    // Arrange & Act
    let arguments = rendered("vec![zero(); 3]");

    // Assert
    assert_eq!(
        arguments,
        Some(vec![String::from("zero ()"), String::from("3")])
    );
}

#[test]
fn expressions_of_a_macro_other_than_matches_returns_its_arguments() {
    // Arrange & Act
    let expressions = rendered_expressions("assert_eq!(left, right)");

    // Assert
    assert_eq!(
        expressions,
        Some(vec![String::from("left"), String::from("right")])
    );
}

// `matches!`'s second argument is a pattern, not an expression: what is
// matched and the guard are the expressions it holds.
#[test]
fn expressions_of_matches_returns_what_it_matches_and_its_guard() {
    // Arrange & Act
    let expressions = rendered_expressions("matches!(value, Some(inner) if inner > limit)");

    // Assert
    assert_eq!(
        expressions,
        Some(vec![String::from("value"), String::from("inner > limit")])
    );
}

#[test]
fn of_a_macro_of_the_standard_family_returns_it() {
    // Arrange
    let node = macro_of("format!(\"{}\", value)");

    // Act
    let standard = StandardMacro::of(&node);

    // Assert
    assert!(standard.is_some());
}

// Its expansion is its author's, and nothing says its body is code.
#[test]
fn of_a_macro_outside_the_standard_family_returns_none() {
    // Arrange
    let node = macro_of("quote!(std::env::args())");

    // Act
    let standard = StandardMacro::of(&node);

    // Assert
    assert!(standard.is_none());
}

// Named by its last segment, as `std::assert!` is the same macro.
#[test]
fn of_a_standard_macro_reached_through_a_path_returns_it() {
    // Arrange
    let node = macro_of("std::assert!(ready)");

    // Act
    let standard = StandardMacro::of(&node);

    // Assert
    assert!(standard.is_some());
}
