// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a package offers its tests: the records, which hold data and nothing to
// test, and the behaviour -- objects' methods, trait methods written by hand,
// trait default methods and free functions -- a test is for.

use stern4rust::finding::parsing::shapes::package_surface::PackageSurface;
use stern4rust::source_file::SourceFile;

fn surface(source: &str) -> PackageSurface {
    PackageSurface::of(&[SourceFile::new("src/widget.rs", source)])
}

#[test]
fn has_records_of_a_record_returns_true() {
    // Arrange
    let surface = surface("pub struct Window {\n    pub start: u64,\n}\n");

    // Act
    let result = surface.has_records();

    // Assert
    assert!(result);
}

#[test]
fn has_records_of_objects_only_returns_false() {
    // Arrange
    let surface =
        surface("pub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> u32 { 1 }\n}\n");

    // Act
    let result = surface.has_records();

    // Assert
    assert!(!result);
}

#[test]
fn is_behaviour_method_of_a_records_getter_returns_false() {
    // Arrange
    let surface = surface(
        "pub struct Window {\n    start: u64,\n}\nimpl Window {\n    pub fn start(&self) -> u64 { self.start }\n}\n",
    );

    // Act
    let result = surface.is_behaviour_method("start");

    // Assert
    assert!(!result);
}

#[test]
fn is_behaviour_method_of_a_trait_default_method_returns_true() {
    // Arrange
    let surface = surface("pub trait Greeter {\n    fn greet(&self) -> u32 { 1 }\n}\n");

    // Act
    let result = surface.is_behaviour_method("greet");

    // Assert
    assert!(result);
}

#[test]
fn is_behaviour_method_of_a_trait_impl_method_returns_true() {
    // Arrange
    let surface = surface(
        "pub struct Window;\nimpl Display for Window {\n    fn fmt(&self, f: &mut Formatter) -> Result { Ok(()) }\n}\n",
    );

    // Act
    let result = surface.is_behaviour_method("fmt");

    // Assert
    assert!(result);
}

#[test]
fn is_behaviour_method_of_an_objects_method_returns_true() {
    // Arrange
    let surface =
        surface("pub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> u32 { 1 }\n}\n");

    // Act
    let result = surface.is_behaviour_method("run");

    // Assert
    assert!(result);
}

#[test]
fn is_free_function_of_a_method_returns_false() {
    // Arrange
    let surface =
        surface("pub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> u32 { 1 }\n}\n");

    // Act
    let result = surface.is_free_function("run");

    // Assert
    assert!(!result);
}

#[test]
fn is_free_function_of_a_top_level_function_returns_true() {
    // Arrange
    let surface = surface("pub fn compute(seed: u64) -> u64 { seed + 1 }\n");

    // Act
    let result = surface.is_free_function("compute");

    // Assert
    assert!(result);
}

#[test]
fn is_object_of_a_record_returns_false() {
    // Arrange
    let surface = surface("pub struct Window {\n    pub start: u64,\n}\n");

    // Act
    let result = surface.is_object("Window");

    // Assert
    assert!(!result);
}

#[test]
fn is_object_of_an_object_returns_true() {
    // Arrange
    let surface =
        surface("pub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> u32 { 1 }\n}\n");

    // Act
    let result = surface.is_object("Engine");

    // Assert
    assert!(result);
}

#[test]
fn is_record_of_a_data_enum_returns_true() {
    // Arrange
    let surface = surface("pub enum Status {\n    Active,\n    Parked(u32),\n}\n");

    // Act
    let result = surface.is_record("Status");

    // Assert
    assert!(result);
}

#[test]
fn is_record_of_a_derived_record_returns_true() {
    // Arrange
    let surface = surface(
        "#[derive(Debug, Clone, Default, PartialEq)]\npub struct Window {\n    pub start: u64,\n}\n",
    );

    // Act
    let result = surface.is_record("Window");

    // Assert
    assert!(result);
}

#[test]
fn is_record_of_a_record_with_a_hand_written_display_returns_false() {
    // Arrange
    let surface = surface(
        "pub struct Window {\n    pub start: u64,\n}\nimpl Display for Window {\n    fn fmt(&self, f: &mut Formatter) -> Result { write!(f, \"{}\", self.start) }\n}\n",
    );

    // Act
    let result = surface.is_record("Window");

    // Assert
    assert!(!result);
}

#[test]
fn is_record_of_a_record_with_a_plain_constructor_returns_true() {
    // Arrange
    let surface = surface(
        "pub struct Window {\n    pub start: u64,\n}\nimpl Window {\n    pub fn new(start: u64) -> Self { Self { start } }\n}\n",
    );

    // Act
    let result = surface.is_record("Window");

    // Assert
    assert!(result);
}

#[test]
fn is_record_of_a_type_declared_in_a_cfg_test_module_returns_false() {
    // Arrange
    let surface = surface(
        "#[cfg(test)]\nmod tests {\n    pub struct Fixture {\n        pub start: u64,\n    }\n}\n",
    );

    // Act
    let result = surface.is_record("Fixture");

    // Assert
    assert!(!result);
}

#[test]
fn is_record_of_an_object_returns_false() {
    // Arrange
    let surface =
        surface("pub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> u32 { 1 }\n}\n");

    // Act
    let result = surface.is_record("Engine");

    // Assert
    assert!(!result);
}
