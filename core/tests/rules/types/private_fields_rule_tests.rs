// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A struct's fields are private. Every field declared `pub`, `pub(crate)`, `pub(super)` or `pub(in ...)` in src/
// is reported -- once per struct, at its name, naming the fields -- and a tuple struct's fields alike. Enum
// variants' fields, tests/ and `#[cfg(test)]` modules are left alone.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::types::private_fields_rule::PrivateFieldsRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "private-fields";

fn check(path: &str, contents: &str) -> Vec<Offence> {
    PrivateFieldsRule::new().check(&SourceFile::new(path, contents))
}

#[test]
fn check_names_the_fields_and_says_what_to_do() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        "pub struct Term {\n    pub number: u64,\n    leader: u32,\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].line(), 1);
    assert_eq!(offences[0].subject(), Some("Term"));
    assert_eq!(
        offences[0].description(),
        "`Term` exposes `number`; a struct's fields are private"
    );
    assert_eq!(
        offences[0].correction(),
        "make it private: give a reader a getter and a builder a constructor"
    );
}

#[test]
fn check_of_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/broken.rs", "pub struct Term {\n    pub number: u64,\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_of_a_newtypes_public_field_reports_it_by_position() {
    // Arrange & Act
    let offences = check("src/height.rs", "pub struct Height(pub u64);\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Height` exposes `0`; a struct's fields are private"
    );
}

#[test]
fn check_of_a_pub_crate_field_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/state.rs",
        "pub struct State {\n    pub(crate) handle: u32,\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`State` exposes `handle`; a struct's fields are private"
    );
}

#[test]
fn check_of_a_pub_in_path_field_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/store/state.rs",
        "pub struct State {\n    pub(in crate::store) handle: u32,\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_of_a_pub_super_field_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/store/state.rs",
        "pub struct State {\n    pub(super) handle: u32,\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_of_a_struct_in_a_cfg_test_module_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        "#[cfg(test)]\nmod tests {\n    struct Fixture {\n        pub number: u64,\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_of_a_struct_in_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        "mod inner {\n    pub struct Term {\n        pub number: u64,\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line(), 2);
}

#[test]
fn check_of_a_struct_under_tests_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/support/fixture.rs",
        "pub struct Fixture {\n    pub number: u64,\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_of_a_unit_struct_reports_nothing() {
    // Arrange & Act
    let offences = check("src/marker.rs", "pub struct Marker;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_of_an_enums_variant_fields_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/message.rs",
        "pub enum Message {\n    Vote { term: u64 },\n    Ping(u64),\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_of_many_fields_names_two_and_counts_the_rest() {
    // Arrange & Act
    let offences = check(
        "src/config.rs",
        "pub struct Config {\n    pub baseline: u8,\n    pub config_file: u8,\n    pub fix: bool,\n    pub format: u8,\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Config` exposes `baseline`, `config_file` and 2 more fields; a struct's fields are private"
    );
    assert_eq!(
        offences[0].correction(),
        "make them private: give a reader a getter and a builder a constructor"
    );
}

#[test]
fn check_of_private_fields_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        "pub struct Term {\n    number: u64,\n}\n\npub struct Height(u64);\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_of_three_fields_names_each() {
    // Arrange & Act
    let offences = check(
        "src/span.rs",
        "pub struct Span {\n    pub start: u8,\n    pub end: u8,\n    pub line: u8,\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Span` exposes `start`, `end` and `line`; a struct's fields are private"
    );
}

#[test]
fn check_of_two_fields_names_both() {
    // Arrange & Act
    let offences = check(
        "src/pair.rs",
        "pub struct Pair {\n    pub left: u8,\n    pub right: u8,\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Pair` exposes `left` and `right`; a struct's fields are private"
    );
}

// The rule's subject is a file, and it says so rather than inheriting a silence nobody chose.
#[test]
fn check_workspace_reports_nothing() {
    // Arrange
    let files = vec![SourceFile::new(
        "src/term.rs",
        "pub struct Term {\n    pub number: u64,\n}\n",
    )];

    // Act
    let offences = PrivateFieldsRule::new().check_workspace(&files);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn explanation_shows_a_public_field_and_the_getter_put_in_its_place() {
    // Arrange & Act
    let explanation = PrivateFieldsRule::new().explanation();

    // Assert
    assert_eq!(explanation.name(), RULE);
    assert!(explanation.breaks().contains("pub number: u64"));
    assert!(
        explanation
            .instead()
            .contains("pub fn number(&self) -> u64")
    );
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = PrivateFieldsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = PrivateFieldsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = PrivateFieldsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
