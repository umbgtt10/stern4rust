// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Whether a type is a seam: a trait object, an `impl Trait`, or a type parameter
// bounded by a trait outside the value traits -- as it is, or wrapped in a Box,
// an Rc, an Arc, an Option, a Vec or a reference. A concrete type is not one.

use stern4rust::finding::parsing::dependencies::seam_types::SeamTypes;
use syn::ItemStruct;
use syn::Type;
use syn::parse_str;

fn seam(declaration: &str, ty: &str) -> bool {
    let declared: ItemStruct = parse_str(declaration).expect("parses");
    let checked: Type = parse_str(ty).expect("parses");
    SeamTypes::of(&[&declared.generics]).is_seam(&checked)
}

#[test]
fn is_seam_of_a_boxed_trait_object_is_true() {
    // Arrange & Act
    let found = seam("struct App {}", "Box<dyn Reporter>");

    // Assert
    assert!(found);
}

#[test]
fn is_seam_of_a_concrete_type_is_false() {
    // Arrange & Act
    let found = (
        seam("struct App {}", "Box<Store>"),
        seam("struct App {}", "String"),
    );

    // Assert
    assert_eq!(found, (false, false));
}

#[test]
fn is_seam_of_a_reference_to_a_trait_object_is_true() {
    // Arrange & Act
    let found = seam("struct App<'a> {}", "&'a dyn Port");

    // Assert
    assert!(found);
}

#[test]
fn is_seam_of_a_shared_optional_trait_object_is_true() {
    // Arrange & Act
    let found = seam("struct App {}", "Option<Arc<dyn Port + Send>>");

    // Assert
    assert!(found);
}

#[test]
fn is_seam_of_a_type_parameter_bounded_by_a_trait_is_true() {
    // Arrange & Act
    let found = seam("struct App<P: Port> {}", "P");

    // Assert
    assert!(found);
}

#[test]
fn is_seam_of_a_type_parameter_bounded_in_a_where_clause_is_true() {
    // Arrange & Act
    let found = seam("struct App<P> where P: Port + Send {}", "Rc<P>");

    // Assert
    assert!(found);
}

#[test]
fn is_seam_of_a_type_parameter_bounded_only_by_value_traits_is_false() {
    // Arrange & Act
    let found = seam("struct Wrapper<T: Clone + Debug> {}", "T");

    // Assert
    assert!(!found);
}

#[test]
fn is_seam_of_a_vector_of_boxed_trait_objects_is_true() {
    // Arrange & Act
    let found = seam("struct Registry {}", "Vec<Box<dyn Rule>>");

    // Assert
    assert!(found);
}

#[test]
fn is_seam_of_an_impl_trait_is_true() {
    // Arrange & Act
    let found = seam("struct App {}", "impl Port");

    // Assert
    assert!(found);
}
