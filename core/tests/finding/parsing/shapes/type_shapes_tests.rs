// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every struct and enum a set of files declares, shaped by the impl blocks the
// same files hold for it -- wherever they are, since an adapter's impl of its
// port may sit in another file than the adapter.

use stern4rust::finding::parsing::shapes::type_shape::TypeShape;
use stern4rust::finding::parsing::shapes::type_shapes::TypeShapes;
use stern4rust::source_file::SourceFile;
use syn::parse_file;

fn shape_in(source: &str, name: &str) -> Option<TypeShape> {
    let items = parse_file(source).expect("parses").items;
    TypeShapes::in_items("src/widget.rs", &items)
        .get(name)
        .map(|shaped| shaped.shape())
}

#[test]
fn get_of_a_name_nothing_declares_returns_none() {
    // Arrange
    let items = parse_file("impl Display for Foreign {\n    fn fmt(&self) {}\n}\n")
        .expect("parses")
        .items;

    // Act
    let found = TypeShapes::in_items("src/widget.rs", &items)
        .get("Foreign")
        .is_none();

    // Assert
    assert!(found);
}

#[test]
fn in_items_finds_a_type_in_an_inline_module() {
    // Arrange & Act
    let shape = shape_in(
        "mod inner {\n    pub struct Widget;\n    impl Widget {\n        pub fn run(&mut self) {}\n    }\n}\n",
        "Widget",
    );

    // Assert
    assert_eq!(shape, Some(TypeShape::Object));
}

#[test]
fn in_items_ignores_impls_in_a_cfg_test_module() {
    // Arrange & Act
    let shape = shape_in(
        "pub struct Widget {\n    pub size: u32,\n}\n\n#[cfg(test)]\nmod tests {\n    impl super::Widget {\n        fn probe(&mut self) {}\n    }\n}\n",
        "Widget",
    );

    // Assert
    assert_eq!(shape, Some(TypeShape::Record));
}

#[test]
fn in_items_shapes_a_record_with_a_display_impl_as_a_record() {
    // Arrange & Act
    let shape = shape_in(
        "pub struct Widget {\n    pub size: u32,\n}\n\nimpl Display for Widget {\n    fn fmt(&self, f: &mut Formatter) -> Result {\n        write!(f, \"{}\", self.size)\n    }\n}\n",
        "Widget",
    );

    // Assert
    assert_eq!(shape, Some(TypeShape::Record));
}

#[test]
fn in_items_shapes_an_enum_with_a_method_as_an_object() {
    // Arrange & Act
    let shape = shape_in(
        "pub enum Kind {\n    A,\n    B,\n}\n\nimpl Kind {\n    pub fn name(self) -> &'static str {\n        match self {\n            Self::A => \"a\",\n            Self::B => \"b\",\n        }\n    }\n}\n",
        "Kind",
    );

    // Assert
    assert_eq!(shape, Some(TypeShape::Object));
}

#[test]
fn of_shapes_a_type_by_an_impl_in_another_file() {
    // Arrange
    let files = [
        SourceFile::new(
            "src/adapter.rs",
            "pub struct Adapter {\n    store: Store,\n}\n",
        ),
        SourceFile::new(
            "src/port_impl.rs",
            "impl Port for Adapter {\n    fn send(&self) {}\n}\n",
        ),
    ];

    // Act
    let shape = TypeShapes::of(&files)
        .get("Adapter")
        .map(|shaped| shaped.shape());

    // Assert
    assert_eq!(shape, Some(TypeShape::Object));
}

#[test]
fn of_skips_a_file_that_does_not_parse() {
    // Arrange
    let files = [
        SourceFile::new("src/broken.rs", "pub struct {"),
        SourceFile::new("src/widget.rs", "pub struct Widget;\n"),
    ];

    // Act
    let names: Vec<String> = TypeShapes::of(&files)
        .types()
        .iter()
        .map(|shaped| shaped.name().to_string())
        .collect();

    // Assert
    assert_eq!(names, ["Widget"]);
}

#[test]
fn types_lists_every_declared_struct_and_enum() {
    // Arrange
    let items = parse_file("pub struct A;\npub enum B {\n    X,\n}\npub trait C {}\n")
        .expect("parses")
        .items;

    // Act
    let names: Vec<String> = TypeShapes::in_items("src/widget.rs", &items)
        .types()
        .iter()
        .map(|shaped| shaped.name().to_string())
        .collect();

    // Assert
    assert_eq!(names, ["A", "B"]);
}
