// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A declared type and what its impl blocks hold, and the shape that follows:
// a builder if it has a consuming build, an object if it has any other
// behaviour, a record otherwise.

use stern4rust::finding::parsing::shapes::method_shape::MethodShape;
use stern4rust::finding::parsing::shapes::shaped_type::ShapedType;
use stern4rust::finding::parsing::shapes::type_shape::TypeShape;

fn declared() -> ShapedType {
    ShapedType::new("Widget", "src/widget.rs", 7)
}

#[test]
fn accessors_counts_the_getters_constructors_and_steps() {
    // Arrange
    let widget = declared()
        .with_method("name", MethodShape::Getter)
        .with_method("size", MethodShape::Getter)
        .with_method("new", MethodShape::PlainConstructor)
        .with_method("with_size", MethodShape::Step);

    // Act
    let accessors = widget.accessors();

    // Assert
    assert_eq!(accessors, 4);
}

#[test]
fn constructors_counts_the_plain_constructors() {
    // Arrange
    let widget = declared().with_method("new", MethodShape::PlainConstructor);

    // Act
    let constructors = widget.constructors();

    // Assert
    assert_eq!(constructors, 1);
}

#[test]
fn getters_lists_the_getters_by_name() {
    // Arrange
    let widget = declared()
        .with_method("name", MethodShape::Getter)
        .with_method("render", MethodShape::Behaviour)
        .with_method("size", MethodShape::Getter);

    // Act
    let getters = widget.getters();

    // Assert
    assert_eq!(getters, ["name", "size"]);
}

#[test]
fn line_returns_the_line_of_the_declaration() {
    // Arrange
    let widget = declared();

    // Act
    let line = widget.line();

    // Assert
    assert_eq!(line, 7);
}

#[test]
fn name_returns_the_type_name() {
    // Arrange
    let widget = declared();

    // Act
    let name = widget.name();

    // Assert
    assert_eq!(name, "Widget");
}

#[test]
fn path_returns_the_declaring_file() {
    // Arrange
    let widget = declared();

    // Act
    let path = widget.path();

    // Assert
    assert_eq!(path, "src/widget.rs");
}

#[test]
fn shape_of_a_type_with_a_behaviour_method_is_an_object() {
    // Arrange
    let widget = declared()
        .with_method("name", MethodShape::Getter)
        .with_method("render", MethodShape::Behaviour);

    // Act
    let shape = widget.shape();

    // Assert
    assert_eq!(shape, TypeShape::Object);
}

#[test]
fn shape_of_a_type_with_a_build_is_a_builder() {
    // Arrange
    let builder = declared()
        .with_method("with_size", MethodShape::Behaviour)
        .with_method("build", MethodShape::Build);

    // Act
    let shape = builder.shape();

    // Assert
    assert_eq!(shape, TypeShape::Builder);
}

#[test]
fn shape_of_a_type_with_a_trait_off_the_value_list_is_an_object() {
    // Arrange
    let widget = declared().with_trait("Rule");

    // Act
    let shape = widget.shape();

    // Assert
    assert_eq!(shape, TypeShape::Object);
}

#[test]
fn shape_of_a_type_with_a_value_trait_is_a_record() {
    // Arrange
    let widget = declared().with_trait("Display").with_trait("From");

    // Act
    let shape = widget.shape();

    // Assert
    assert_eq!(shape, TypeShape::Record);
}

#[test]
fn shape_of_a_type_with_accessors_only_is_a_record() {
    // Arrange
    let widget = declared()
        .with_method("name", MethodShape::Getter)
        .with_method("new", MethodShape::PlainConstructor)
        .with_method("with_size", MethodShape::Step);

    // Act
    let shape = widget.shape();

    // Assert
    assert_eq!(shape, TypeShape::Record);
}

#[test]
fn shape_of_a_type_with_no_impl_is_a_record() {
    // Arrange
    let widget = declared();

    // Act
    let shape = widget.shape();

    // Assert
    assert_eq!(shape, TypeShape::Record);
}

#[test]
fn steps_counts_the_with_steps() {
    // Arrange
    let widget = declared()
        .with_method("with_name", MethodShape::Step)
        .with_method("with_size", MethodShape::Step);

    // Act
    let steps = widget.steps();

    // Assert
    assert_eq!(steps, 2);
}
