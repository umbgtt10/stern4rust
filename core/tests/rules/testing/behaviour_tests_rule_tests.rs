// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A test exercises behaviour of its package. One that builds a record and
// calls nothing but its accessors and derived traits tests the compiler: that a
// derived `Default` gives zeros, that a clone equals its original.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::testing::behaviour_tests_rule::BehaviourTestsRule;
use stern4rust::source_file::SourceFile;

fn check(files: &[(&str, &str)]) -> Vec<Offence> {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, body)| SourceFile::new(path, body))
        .collect();
    BehaviourTestsRule::new().check_workspace(&sources)
}

#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "#[test]\nfn default_is_zero() {}\n",
    );

    // Act
    let found = BehaviourTestsRule::new().check(&file);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn check_workspace_of_a_function_in_src_is_not_judged() {
    // Arrange & Act
    let found = check(&[
        (
            "src/window.rs",
            "#[derive(Default, PartialEq, Debug)]\npub struct Window {\n    pub start: u64,\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self, window: &Window) -> u64 { window.start }\n}\n",
        ),
        (
            "src/checks.rs",
            "#[test]\nfn default_is_zero() {\n    assert_eq!(Window::default().start, 0);\n}\n",
        ),
    ]);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn check_workspace_of_a_record_test_names_the_record_in_its_description() {
    // Arrange & Act
    let found = check(&[
        (
            "src/window.rs",
            "#[derive(Default, PartialEq, Debug)]\npub struct Window {\n    pub start: u64,\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self, window: &Window) -> u64 { window.start }\n}\n",
        ),
        (
            "tests/window_tests.rs",
            "#[test]\nfn default_is_zero() {\n    assert_eq!(Window::default().start, 0);\n}\n",
        ),
    ]);

    // Assert
    assert!(found[0].description().contains("`Window`"));
}

#[test]
fn check_workspace_of_a_test_converting_through_a_hand_written_from_reports_nothing() {
    // Arrange
    let sources = [
        SourceFile::new(
            "src/args.rs",
            "pub struct Args {\n    pub jobs: u32,\n}\nimpl From<Args> for Config {\n    fn from(args: Args) -> Self {\n        Config { jobs: args.jobs }\n    }\n}\n",
        ),
        SourceFile::new(
            "tests/args_tests.rs",
            "#[test]\nfn from_carries_the_jobs() {\n    assert_eq!(Config::from(Args { jobs: 2 }).jobs, 2);\n}\n",
        ),
    ];

    // Act
    let found = BehaviourTestsRule::new().check_workspace(&sources);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn check_workspace_of_a_test_exercising_behaviour_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        (
            "src/window.rs",
            "#[derive(Default, PartialEq, Debug)]\npub struct Window {\n    pub start: u64,\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self, window: &Window) -> u64 { window.start }\n}\n",
        ),
        (
            "tests/engine_tests.rs",
            "#[test]\nfn run_reads_the_start() {\n    assert_eq!(Engine.run(&Window::default()), 0);\n}\n",
        ),
    ]);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn check_workspace_of_a_test_naming_no_record_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        (
            "src/window.rs",
            "#[derive(Default, PartialEq, Debug)]\npub struct Window {\n    pub start: u64,\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self, window: &Window) -> u64 { window.start }\n}\n",
        ),
        (
            "tests/sums_tests.rs",
            "#[test]\nfn adds() {\n    assert_eq!(1 + 1, 2);\n}\n",
        ),
    ]);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn check_workspace_of_a_test_of_a_record_alone_reports_it() {
    // Arrange & Act
    let found = check(&[
        (
            "src/window.rs",
            "#[derive(Default, PartialEq, Debug)]\npub struct Window {\n    pub start: u64,\n}\npub struct Engine;\nimpl Engine {\n    pub fn run(&self, window: &Window) -> u64 { window.start }\n}\n",
        ),
        (
            "tests/window_tests.rs",
            "#[test]\nfn default_is_zero() {\n    assert_eq!(Window::default().start, 0);\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule(), "behaviour-tests");
    assert_eq!(found[0].subject(), Some("default_is_zero"));
    assert_eq!(found[0].file(), "tests/window_tests.rs");
}

#[test]
fn explanation_names_the_rule() {
    // Arrange & Act
    let explanation = BehaviourTestsRule::new().explanation();

    // Assert
    assert_eq!(explanation.name, "behaviour-tests");
}

#[test]
fn name_is_behaviour_tests() {
    // Arrange & Act
    let name = BehaviourTestsRule::new().name();

    // Assert
    assert_eq!(name, "behaviour-tests");
}
