// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A function in an inherent impl is `pub` or private. `pub(crate)`, `pub(super)`,
// `pub(self)` and `pub(in ...)` are each reported at the method's name, with a
// correction read from the package: private where nothing outside its file
// calls it, `pub` with a test where something does, and the type's spread where
// its impl sits in more than one file. tests/ is exempt.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::public_or_private_rule::PublicOrPrivateRule;
use stern4rust::source_file::SourceFile;

const ELECTION: &str = "pub struct RaftProtocol;\n\nimpl RaftProtocol {\n    pub(crate) fn election(&self, term: u64) -> u64 {\n        term\n    }\n}\n";
const RULE: &str = "public-or-private";

fn check_workspace(files: &[(&str, &str)]) -> Vec<Offence> {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(file, contents)| SourceFile::new(file, contents))
        .collect();
    PublicOrPrivateRule::new().check_workspace(&files)
}

// This rule answers the tree question only, and says so rather than inheriting
// a silence nobody chose.
#[test]
fn check_of_a_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/raft.rs", ELECTION);

    // Act
    let offences = PublicOrPrivateRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_matches_a_call_on_its_arity_too() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/raft.rs", ELECTION),
        (
            "src/router.rs",
            "pub fn route(protocol: &RaftProtocol) {\n    protocol.election();\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences[0].correction, "make it private: `fn`");
}

#[test]
fn check_workspace_of_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "src/broken.rs",
        "impl RaftProtocol {\n    pub(crate) fn election(\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_method_called_from_another_file_offers_pub_and_a_test() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/raft.rs", ELECTION),
        (
            "src/router.rs",
            "pub fn route(protocol: &RaftProtocol) -> u64 {\n    protocol.election(3)\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(
        offences[0].correction,
        "make it `pub` and give it a test, or move the caller's need behind the type's public \
         contract"
    );
}

#[test]
fn check_workspace_of_a_method_called_only_in_its_own_file_offers_private() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "src/raft.rs",
        "pub struct RaftProtocol;\n\nimpl RaftProtocol {\n    pub(crate) fn election(&self, term: u64) -> u64 {\n        term\n    }\n\n    \
         pub fn step(&self) -> u64 {\n        self.election(1)\n    }\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].correction, "make it private: `fn`");
}

#[test]
fn check_workspace_of_a_method_in_a_spread_impl_names_implemented_in_one_file() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/raft.rs", ELECTION),
        (
            "src/raft_timers.rs",
            "impl RaftProtocol {\n    pub fn tick(&self) -> u64 {\n        self.election(1)\n    }\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(
        offences[0].correction,
        "implement `RaftProtocol` in one file, as `implemented-in-one-file` asks: the restriction \
         serves its spread over 2 files"
    );
}

#[test]
fn check_workspace_of_a_pub_crate_method_reports_it_at_its_name() {
    // Arrange & Act
    let offences = check_workspace(&[("src/raft.rs", ELECTION)]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "src/raft.rs");
    assert_eq!(offences[0].line, 4);
    assert_eq!(offences[0].subject.as_deref(), Some("election"));
    assert_eq!(
        offences[0].description,
        "`election` is `pub(crate)` in `impl RaftProtocol`; a method is `pub` or private"
    );
}

// `pub(self)` already means private, whatever calls it.
#[test]
fn check_workspace_of_a_pub_self_method_offers_private() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "src/raft.rs",
        "pub struct RaftProtocol;\n\nimpl RaftProtocol {\n    pub(self) fn election(&self) {}\n}\n",
    )]);

    // Assert
    assert_eq!(offences[0].correction, "make it private: `fn`");
}

#[test]
fn check_workspace_of_pub_and_private_methods_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "src/store.rs",
        "pub struct Store;\n\nimpl Store {\n    pub fn len(&self) -> usize {\n        self.count()\n    }\n\n    \
         fn count(&self) -> usize {\n        0\n    }\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_restricted_methods_in_tests_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "tests/support/raft_fake.rs",
        "pub struct RaftFake;\n\nimpl RaftFake {\n    pub(crate) fn election(&self) {}\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_reports_each_restricted_method() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "src/store.rs",
        "pub struct Store;\n\nimpl Store {\n    pub(crate) fn grow(&self) {}\n\n    pub(super) fn shrink(&self) {}\n}\n",
    )]);

    // Assert
    let subjects: Vec<Option<&str>> = offences
        .iter()
        .map(|offence| offence.subject.as_deref())
        .collect();
    assert_eq!(subjects, vec![Some("grow"), Some("shrink")]);
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = PublicOrPrivateRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = PublicOrPrivateRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = PublicOrPrivateRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
