// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where a method body fills a field of its type: each field of a struct literal
// naming `Self` or the type, each position of a tuple struct's constructor, each
// call to another associated function of the type with what it is handed, and
// each `self.field = ...`. Nothing about any other type.

use stern4rust::finding::parsing::dependencies::built_text::BuiltText;
use stern4rust::finding::parsing::dependencies::dependency_fills::DependencyFills;
use syn::ItemFn;
use syn::parse_str;

fn method(source: &str) -> ItemFn {
    parse_str(source).expect("parses")
}

#[test]
fn assignments_of_a_method_name_each_field_of_self_it_assigns() {
    // Arrange
    let found = method(
        "fn reset(&mut self) {\n    self.flash = Registry::current();\n    other.size = 1;\n}",
    );

    // Act
    let assigned = DependencyFills::assignments(&found.block);

    // Assert
    assert_eq!(assigned.len(), 1);
    assert_eq!(assigned[0].field, "flash");
    assert_eq!(BuiltText::of(&assigned[0].value), "Registry::current()");
    assert_eq!(assigned[0].line, 2);
}

#[test]
fn delegations_of_a_call_to_another_function_of_the_type_name_it_and_its_arguments() {
    // Arrange
    let found = method("fn new() -> Self {\n    Self::with_deps(Box::new(Resolver::new()), 3)\n}");

    // Act
    let delegated = DependencyFills::delegations(&found.block, "App");

    // Assert
    assert_eq!(delegated.len(), 1);
    assert_eq!(delegated[0].target, "with_deps");
    assert_eq!(delegated[0].arguments.len(), 2);
}

#[test]
fn delegations_of_a_call_to_another_types_function_are_left_out() {
    // Arrange
    let found = method("fn new() -> Self {\n    Other::with_deps(Box::new(Resolver::new()))\n}");

    // Act
    let delegated = DependencyFills::delegations(&found.block, "App");

    // Assert
    assert!(delegated.is_empty());
}

#[test]
fn fills_of_a_literal_naming_the_type_and_of_a_tuple_constructor_name_each_field() {
    // Arrange
    let found = method(
        "fn new(a: A, b: B) -> Self {\n    let first = App { reporter: a };\n    Self(first, b)\n}",
    );

    // Act
    let filled: Vec<String> = DependencyFills::fills(&found.block, "App")
        .into_iter()
        .map(|fill| fill.field)
        .collect();

    // Assert
    assert_eq!(filled, ["reporter", "0", "1"]);
}

#[test]
fn fills_of_a_struct_literal_name_each_field_its_value_and_its_line() {
    // Arrange
    let found = method(
        "fn new() -> Self {\n    Self {\n        reporter: Box::new(Stdout::new()),\n        size: 1,\n    }\n}",
    );

    // Act
    let filled = DependencyFills::fills(&found.block, "App");

    // Assert
    assert_eq!(filled.len(), 2);
    assert_eq!(filled[0].field, "reporter");
    assert_eq!(BuiltText::of(&filled[0].value), "Stdout");
    assert_eq!(filled[0].line, 3);
    assert_eq!(filled[1].field, "size");
}

#[test]
fn fills_of_another_types_literal_are_left_out() {
    // Arrange
    let found = method(
        "fn new() -> Self {\n    let other = Other { reporter: Box::new(Stdout::new()) };\n    todo!()\n}",
    );

    // Act
    let filled = DependencyFills::fills(&found.block, "App");

    // Assert
    assert!(filled.is_empty());
}
