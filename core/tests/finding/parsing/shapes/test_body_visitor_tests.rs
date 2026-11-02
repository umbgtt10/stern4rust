// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One test's walk. Calls are judged by what they reach: a record's own
// constructor is not behaviour, an object's method is, and so is a free function
// of the package, reached directly, through a path, through a helper or a local
// method of the test's own file, or inside a standard macro's arguments.

use std::collections::BTreeMap;
use stern4rust::finding::parsing::shapes::package_surface::PackageSurface;
use stern4rust::finding::parsing::shapes::test_body_visitor::TestBodyVisitor;
use stern4rust::finding::parsing::unit_test_finder::UnitTestFinder;
use stern4rust::source_file::SourceFile;
use syn::Block;
use syn::ImplItem;
use syn::Item;
use syn::ItemFn;
use syn::parse_file;

type Split<'a> = (
    &'a ItemFn,
    BTreeMap<String, &'a Block>,
    BTreeMap<String, &'a Block>,
);

fn split(items: &[Item]) -> Split<'_> {
    let functions: Vec<&ItemFn> = items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(function) => Some(function),
            _ => None,
        })
        .collect();
    let test = functions
        .iter()
        .find(|function| function.attrs.iter().any(UnitTestFinder::is_test))
        .copied()
        .expect("one test");
    let helpers = functions
        .iter()
        .filter(|function| !function.attrs.iter().any(UnitTestFinder::is_test))
        .map(|function| (function.sig.ident.to_string(), function.block.as_ref()))
        .collect();
    let methods = items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(block) => Some(block),
            _ => None,
        })
        .flat_map(|block| block.items.iter())
        .filter_map(|member| match member {
            ImplItem::Fn(method) => Some((method.sig.ident.to_string(), &method.block)),
            _ => None,
        })
        .collect();
    (test, helpers, methods)
}

fn surface() -> PackageSurface {
    PackageSurface::of(&[SourceFile::new(
        "src/widget.rs",
        "pub struct Window {\n    pub start: u64,\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> u64 { 1 }\n}\npub fn compute(seed: u64) -> u64 { seed }\n",
    )])
}

#[test]
fn examine_of_a_call_qualified_by_a_record_is_not_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn built() {\n    let window = Window::default();\n    assert_eq!(window.start, 0);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert_eq!(subject.records, vec!["Window".to_string()]);
    assert!(!subject.exercises_behaviour);
}

#[test]
fn examine_of_a_call_qualified_by_an_object_is_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn ran() {\n    let engine = Engine::default();\n    assert_eq!(engine.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_a_free_function_through_a_module_path_is_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn computed() {\n    let window = Window { start: crate::widget::compute(1) };\n    assert_eq!(window.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_a_hand_written_trait_method_through_a_path_is_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn converts() {\n    let window = Window { start: 1 };\n    assert_eq!(Span::from(window).start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = PackageSurface::of(&[SourceFile::new(
        "src/widget.rs",
        "pub struct Window {\n    pub start: u64,\n}\nimpl From<Window> for Span {\n    fn from(window: Window) -> Self {\n        Span { start: window.start }\n    }\n}\n",
    )]);

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_a_helper_calling_behaviour_is_behaviour() {
    // Arrange
    let parsed = parse_file("fn ran() -> u64 {\n    Engine.run()\n}\n\n#[test]\nfn reads() {\n    let window = Window { start: ran() };\n    assert_eq!(window.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_a_local_method_calling_behaviour_is_behaviour() {
    // Arrange
    let parsed = parse_file("struct Harness;\nimpl Harness {\n    fn go(&self) -> u64 {\n        compute(1)\n    }\n}\n\n#[test]\nfn reads() {\n    let window = Window { start: Harness.go() };\n    assert_eq!(window.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_a_macro_its_walk_cannot_see_is_behaviour() {
    // Arrange
    let parsed = parse_file(
        "#[test]\nfn reads() {\n    let window = Window { start: 1 };\n    scenario!(window);\n}\n",
    )
    .expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_a_record_inside_a_pattern_names_it() {
    // Arrange
    let parsed = parse_file("#[test]\nfn matched() {\n    let window = Window { start: 1 };\n    assert!(matches!(window, Window { start: 1 } if true));\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert_eq!(subject.records, vec!["Window".to_string()]);
    assert!(!subject.exercises_behaviour);
}

#[test]
fn examine_of_a_recursive_helper_ends() {
    // Arrange
    let parsed = parse_file("fn deeper(n: u64) -> u64 {\n    deeper(n)\n}\n\n#[test]\nfn reads() {\n    let window = Window { start: deeper(1) };\n    assert_eq!(window.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert_eq!(subject.records, vec!["Window".to_string()]);
    assert!(!subject.exercises_behaviour);
}

#[test]
fn examine_of_a_struct_literal_names_the_record() {
    // Arrange
    let parsed = parse_file("#[test]\nfn built() {\n    let window = Window { start: 1 };\n    assert_eq!(window.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert_eq!(subject.records, vec!["Window".to_string()]);
    assert!(!subject.exercises_behaviour);
}

#[test]
fn examine_of_behaviour_beside_a_guard_is_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn guarded() {\n    let window = Window { start: 1 };\n    assert!(matches!(compute(window.start), x if x > 0));\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_behaviour_inside_a_vec_repeat_is_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn reads() {\n    let starts = vec![compute(1); 2];\n    assert_eq!(Window { start: starts[0] }.start, 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_of_behaviour_inside_an_assert_is_behaviour() {
    // Arrange
    let parsed = parse_file("#[test]\nfn reads() {\n    let window = Window { start: 1 };\n    assert_eq!(compute(window.start), 1);\n}\n").expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert!(subject.exercises_behaviour);
}

#[test]
fn examine_reports_the_tests_name_and_line() {
    // Arrange
    let parsed =
        parse_file("\n#[test]\nfn built() {\n    assert_eq!(Window { start: 1 }.start, 1);\n}\n")
            .expect("valid test file");
    let (test, helpers, methods) = split(&parsed.items);
    let surface = surface();

    // Act
    let subject = TestBodyVisitor::new(&surface, &helpers, &methods).examine(test);

    // Assert
    assert_eq!(subject.name, "built");
    assert_eq!(subject.line, 3);
}
