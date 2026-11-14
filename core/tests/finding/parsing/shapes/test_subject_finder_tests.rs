// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Each test's subject: the records of its package it names, and whether it
// calls any behaviour of the package -- in its body, inside the standard
// macros, or through a helper of its own file. A macro it cannot see into
// counts as behaviour, so a test is never accused on a guess.

use stern4rust::finding::parsing::shapes::package_surface::PackageSurface;
use stern4rust::finding::parsing::shapes::test_subject_finder::TestSubjectFinder;
use stern4rust::source_file::SourceFile;

fn surface() -> PackageSurface {
    PackageSurface::of(&[SourceFile::new(
        "src/window.rs",
        "pub struct Window {\n    pub start: u64,\n    pub items: Vec<u64>,\n}\nimpl Window {\n    pub fn new(start: u64, items: Vec<u64>) -> Self { Self { start, items } }\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self, window: Window) -> u64 { window.start }\n}\npub fn compute(window: &Window) -> u64 { window.start }\n",
    )])
}

#[test]
fn find_of_a_helper_alone_finds_nothing() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "fn window() -> Window {\n    Window::new(3, vec![])\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert!(subjects.is_empty());
}

#[test]
fn find_of_a_test_asserting_a_records_default_names_it_without_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn default_is_zero() {\n    let window = Window::default();\n    assert_eq!(window, Window { start: 0, items: Vec::new() });\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].records, vec!["Window".to_string()]);
    assert!(!subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_calling_a_free_function_exercises_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn compute_reads_the_start() {\n    let window = Window::new(3, vec![]);\n    assert_eq!(compute(&window), 3);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert!(subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_calling_a_helper_that_calls_behaviour_exercises_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "fn ran(window: Window) -> u64 {\n    Engine.run(window)\n}\n\n#[test]\nfn run_reads_the_start() {\n    assert_eq!(ran(Window::new(3, vec![])), 3);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert!(subjects[0].exercises_behaviour);
}

// A local method's body is walked as a helper's is, wherever its impl sits --
// here inside an inline module of the test file.
#[test]
fn find_of_a_test_calling_a_local_method_that_calls_behaviour_exercises_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "mod support {\n    pub struct Harness;\n\n    impl Harness {\n        pub fn go(&self) -> u64 {\n            compute(&Window::new(1, vec![]))\n        }\n    }\n}\n\n#[test]\nfn reads() {\n    let window = Window { start: Harness.go(), items: vec![] };\n    assert_eq!(window.start, 1);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert!(subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_calling_a_records_constructor_names_it_without_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn new_keeps_the_start() {\n    let window = Window::new(3, vec![]);\n    assert_eq!(window.start, 3);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].records, vec!["Window".to_string()]);
    assert!(!subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_calling_an_objects_method_inside_an_assert_exercises_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn run_reads_the_start() {\n    assert_eq!(Engine.run(Window::new(3, vec![])), 3);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert!(subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_calling_only_std_on_a_records_field_names_it_without_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn new_keeps_the_items() {\n    let window = Window::new(3, vec![1, 2]);\n    assert_eq!(window.items.len(), 2);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].records, vec!["Window".to_string()]);
    assert!(!subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_cloning_a_record_names_it_without_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn clone_is_equal() {\n    let window = Window::new(3, vec![]);\n    assert_eq!(window.clone(), window);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].records, vec!["Window".to_string()]);
    assert!(!subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_in_an_inline_module_is_found() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "mod nested {\n    #[test]\n    fn default_is_zero() {\n        assert_eq!(Window::default().start, 0);\n    }\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].records, vec!["Window".to_string()]);
    assert!(!subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_invoking_an_unknown_macro_exercises_behaviour() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn scenario_runs() {\n    let window = Window::new(3, vec![]);\n    scenario!(window);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert!(subjects[0].exercises_behaviour);
}

#[test]
fn find_of_a_test_naming_no_record_names_none() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn adds() {\n    assert_eq!(1 + 1, 2);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects.len(), 1);
    assert!(subjects[0].records.is_empty());
}

#[test]
fn find_of_a_test_reports_its_name_and_line() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "\n#[test]\nfn default_is_zero() {\n    assert_eq!(Window::default().start, 0);\n}\n",
    );

    // Act
    let subjects = TestSubjectFinder::new(&surface()).find(&file);

    // Assert
    assert_eq!(subjects[0].name, "default_is_zero");
    assert_eq!(subjects[0].line, 3);
}
