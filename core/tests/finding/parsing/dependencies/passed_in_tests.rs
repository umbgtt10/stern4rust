// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Whether a value a method stores came from the caller: a parameter, as it is
// or as `Some(p)`, `Box::new(p)`, `Rc::new(p)`, `Arc::new(p)`, `p.clone()`,
// `Rc::clone(&p)`, `Arc::clone(&p)`, `p.into()` or a parameter's field; `None`
// or an empty vector; or a local bound to any of these, by name or by
// destructuring, a rebound name reading its earlier binding; and what only
// repackages them -- `?`, an `into_*` conversion, or a function of the type's own
// given nothing else. Anything else the method built.

use quote::ToTokens;
use stern4rust::finding::parsing::dependencies::passed_in::PassedIn;
use syn::Expr;
use syn::ItemFn;
use syn::parse_str;

const METHOD: &str = "fn new(port: Box<dyn Port>, deps: Deps) -> Self {\n    let shared = Arc::new(port);\n    let built = StdoutReporter::new();\n    let looped = Box::new(Concrete);\n    let looped = Some(looped);\n    let Parts { store, .. } = deps;\n    let (first, second) = (port, Concrete);\n    let parts = Self::validated(port)?;\n    let parts = parts.into_parts();\n    todo!()\n}";
const SHADOWING: &str =
    "fn new(port: Box<dyn Port>) -> Self {\n    let port = Box::new(Concrete);\n    todo!()\n}";

fn holds(value: &str) -> bool {
    holds_in(METHOD, value)
}

fn holds_in(source: &str, value: &str) -> bool {
    let method: ItemFn = parse_str(source).expect("parses");
    let checked: Expr = parse_str(value).expect("parses");
    PassedIn::of(&method.sig, &method.block, "App").holds(&checked)
}

fn origin(value: &str) -> String {
    let method: ItemFn = parse_str(METHOD).expect("parses");
    let checked: Expr = parse_str(value).expect("parses");
    PassedIn::of(&method.sig, &method.block, "App")
        .origin(&checked)
        .to_token_stream()
        .to_string()
}

#[test]
fn holds_of_a_binding_destructured_from_a_parameter_is_true() {
    // Arrange & Act
    let found = holds("store");

    // Assert
    assert!(found);
}

#[test]
fn holds_of_a_function_given_a_built_value_or_nothing_or_of_another_type_is_false() {
    // Arrange & Act
    let found = (
        holds("Self::validated(StdoutReporter::new())"),
        holds("Self::defaults()"),
        holds("Other::validated(port)"),
    );

    // Assert
    assert_eq!(found, (false, false, false));
}

#[test]
fn holds_of_a_local_bound_to_a_built_value_is_false() {
    // Arrange & Act
    let found = holds("built");

    // Assert
    assert!(!found);
}

#[test]
fn holds_of_a_local_bound_to_a_parameter_is_true() {
    // Arrange & Act
    let found = holds("shared");

    // Assert
    assert!(found);
}

#[test]
fn holds_of_a_local_rebound_from_a_repackaging_of_itself_is_true() {
    // Arrange & Act
    let found = holds("parts");

    // Assert
    assert!(found);
}

#[test]
fn holds_of_a_parameter_in_each_passed_in_shape_is_true() {
    // Arrange
    let shapes = [
        "port",
        "Some(port)",
        "Box::new(port)",
        "Rc::new(port)",
        "Arc::new(port)",
        "port.clone()",
        "Rc::clone(&port)",
        "Arc::clone(&port)",
        "port.into()",
        "Some(Box::new(port))",
    ];

    // Act
    let refused: Vec<&str> = shapes.into_iter().filter(|shape| !holds(shape)).collect();

    // Assert
    assert!(refused.is_empty(), "refused {refused:?}");
}

#[test]
fn holds_of_a_parameter_rebound_to_a_built_value_is_false() {
    // Arrange & Act
    let found = holds_in(SHADOWING, "port");

    // Assert
    assert!(!found);
}

#[test]
fn holds_of_a_parameters_field_is_true() {
    // Arrange & Act
    let found = holds("deps.store");

    // Assert
    assert!(found);
}

#[test]
fn holds_of_a_passed_in_value_behind_a_question_mark_is_true() {
    // Arrange & Act
    let found = holds("deps?");

    // Assert
    assert!(found);
}

#[test]
fn holds_of_a_shadowed_local_bound_to_itself_is_false() {
    // Arrange & Act
    let found = holds("looped");

    // Assert
    assert!(!found);
}

#[test]
fn holds_of_a_value_built_inside_is_false() {
    // Arrange & Act
    let found = (
        holds("Box::new(StdoutReporter::new())"),
        holds("NoOpCoordinator"),
        holds("Registry::current()"),
    );

    // Assert
    assert_eq!(found, (false, false, false));
}

#[test]
fn holds_of_an_into_conversion_of_a_parameter_is_true() {
    // Arrange & Act
    let found = holds("deps.into_parts(3)");

    // Assert
    assert!(found);
}

#[test]
fn holds_of_another_method_call_on_a_parameter_is_false() {
    // Arrange & Act
    let found = holds("deps.reporter()");

    // Assert
    assert!(!found);
}

#[test]
fn holds_of_each_tuple_binding_reads_its_own_element() {
    // Arrange & Act
    let found = (holds("first"), holds("second"));

    // Assert
    assert_eq!(found, (true, false));
}

#[test]
fn holds_of_none_and_an_empty_vector_is_true() {
    // Arrange & Act
    let found = (holds("None"), holds("Vec::new()"), holds("vec![]"));

    // Assert
    assert_eq!(found, (true, true, true));
}

#[test]
fn holds_of_the_types_own_function_given_only_passed_in_values_is_true() {
    // Arrange & Act
    let found = (
        holds("Self::validated(port, deps.store)"),
        holds("App::validated(port)"),
    );

    // Assert
    assert_eq!(found, (true, true));
}

#[test]
fn origin_of_a_destructured_binding_is_what_it_was_destructured_from() {
    // Arrange & Act
    let found = origin("store");

    // Assert
    assert_eq!(found, "deps");
}

#[test]
fn origin_of_a_local_is_its_initialiser() {
    // Arrange & Act
    let found = origin("built");

    // Assert
    assert_eq!(found, "StdoutReporter :: new ()");
}

#[test]
fn origin_of_a_parameter_is_itself() {
    // Arrange & Act
    let found = origin("port");

    // Assert
    assert_eq!(found, "port");
}
