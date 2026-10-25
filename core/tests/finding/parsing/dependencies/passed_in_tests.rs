// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Whether a value a method stores came from the caller: a parameter, as it is
// or as `Some(p)`, `Box::new(p)`, `Rc::new(p)`, `Arc::new(p)`, `p.clone()`,
// `Rc::clone(&p)`, `Arc::clone(&p)`, `p.into()` or a parameter's field; `None`
// or an empty vector; or a local bound to any of these. Anything else the method
// built.

use quote::ToTokens;
use stern4rust::finding::parsing::dependencies::passed_in::PassedIn;
use syn::Expr;
use syn::ItemFn;
use syn::parse_str;

const METHOD: &str = "fn new(port: Box<dyn Port>, deps: Deps) -> Self {\n    let shared = Arc::new(port);\n    let built = StdoutReporter::new();\n    let looped = Box::new(Concrete);\n    let looped = Some(looped);\n    todo!()\n}";

fn holds(value: &str) -> bool {
    let method: ItemFn = parse_str(METHOD).expect("parses");
    let checked: Expr = parse_str(value).expect("parses");
    PassedIn::of(&method.sig, &method.block).holds(&checked)
}

fn origin(value: &str) -> String {
    let method: ItemFn = parse_str(METHOD).expect("parses");
    let checked: Expr = parse_str(value).expect("parses");
    PassedIn::of(&method.sig, &method.block)
        .origin(&checked)
        .to_token_stream()
        .to_string()
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
fn holds_of_a_parameters_field_is_true() {
    // Arrange & Act
    let found = holds("deps.store");

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
fn holds_of_none_and_an_empty_vector_is_true() {
    // Arrange & Act
    let found = (holds("None"), holds("Vec::new()"), holds("vec![]"));

    // Assert
    assert_eq!(found, (true, true, true));
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
