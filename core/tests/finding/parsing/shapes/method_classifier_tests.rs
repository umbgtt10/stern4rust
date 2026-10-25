// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What one method of an impl block is, for telling a record from an object:
// a getter, a plain constructor and a with_ step only hold a record's values;
// a consuming build marks a builder; anything else is behaviour.

use stern4rust::finding::parsing::shapes::method_classifier::MethodClassifier;
use stern4rust::finding::parsing::shapes::method_shape::MethodShape;
use syn::ImplItemFn;
use syn::parse_str;

fn shape(source: &str) -> MethodShape {
    let method: ImplItemFn = parse_str(source).expect("parses");
    MethodClassifier::shape(&method, "Widget")
}

#[test]
fn shape_of_a_build_consuming_self_and_returning_another_type_is_build() {
    // Arrange & Act
    let shape =
        shape("pub fn build(self) -> Result<Node, BuildError> {\n    Ok(Node::new(self.parts))\n}");

    // Assert
    assert_eq!(shape, MethodShape::Build);
}

#[test]
fn shape_of_a_build_returning_its_own_type_is_behaviour() {
    // Arrange & Act
    let shape = shape("pub fn build(self) -> Self {\n    self\n}");

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_constructor_filling_a_default_is_behaviour() {
    // Arrange & Act
    let shape =
        shape("pub fn new(name: String) -> Self {\n    Self { name, items: Vec::new() }\n}");

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_constructor_returning_a_result_is_behaviour() {
    // Arrange & Act
    let shape =
        shape("pub fn new(name: String) -> Result<Self, Error> {\n    Ok(Self { name })\n}");

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_constructor_storing_its_parameters_is_a_plain_constructor() {
    // Arrange
    let sources = [
        "pub fn new(name: String, size: u32) -> Self {\n    Self { name, size }\n}",
        "pub const fn new(name: String) -> Widget {\n    Widget { name: name }\n}",
        "pub fn new(value: u64) -> Self {\n    Self(value)\n}",
        "pub fn new() -> Self {\n    Self\n}",
    ];

    // Act
    let shapes: Vec<MethodShape> = sources.iter().map(|source| shape(source)).collect();

    // Assert
    assert_eq!(shapes, [MethodShape::PlainConstructor; 4]);
}

#[test]
fn shape_of_a_method_computing_from_a_field_is_behaviour() {
    // Arrange & Act
    let shape = shape("pub fn count(&self) -> usize {\n    self.items.len()\n}");

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_method_reading_one_field_is_a_getter() {
    // Arrange
    let sources = [
        "pub fn size(&self) -> u32 {\n    self.size\n}",
        "pub fn name(&self) -> &str {\n    &self.name\n}",
        "pub fn path(&self) -> &Path {\n    &*self.path\n}",
        "pub fn label(&self) -> Option<&str> {\n    self.label.as_deref()\n}",
        "pub fn tags(&self) -> Vec<String> {\n    (self.tags.clone())\n}",
        "pub fn first(&self) -> u8 {\n    self.0\n}",
    ];

    // Act
    let shapes: Vec<MethodShape> = sources.iter().map(|source| shape(source)).collect();

    // Assert
    assert_eq!(shapes, [MethodShape::Getter; 6]);
}

#[test]
fn shape_of_a_method_taking_a_parameter_is_behaviour() {
    // Arrange & Act
    let shape = shape("pub fn item(&self, at: usize) -> u32 {\n    self.items[at]\n}");

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_mut_self_method_is_behaviour() {
    // Arrange & Act
    let shape = shape("pub fn size(&mut self) -> u32 {\n    self.size\n}");

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_step_with_logic_is_behaviour() {
    // Arrange & Act
    let shape = shape(
        "pub fn with_size(mut self, size: u32) -> Self {\n    self.size = Some(size);\n    self\n}",
    );

    // Assert
    assert_eq!(shape, MethodShape::Behaviour);
}

#[test]
fn shape_of_a_with_step_replacing_one_field_is_a_step() {
    // Arrange & Act
    let shape = shape("pub fn with_size(self, size: u32) -> Self {\n    Self { size, ..self }\n}");

    // Assert
    assert_eq!(shape, MethodShape::Step);
}
