// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every struct in src/ is a record or an object, by its impl blocks across the
// package. A record has no behaviour: its fields are all pub and it has no
// getters, plain constructors or with_ steps. An object has behaviour: its fields
// are all private. One offence per struct, at its name. Enum variants' fields,
// tests/ and #[cfg(test)] modules are left alone.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::types::records_and_objects_rule::RecordsAndObjectsRule;
use stern4rust::source_file::SourceFile;

const OBJECT: &str = "\n\nimpl Term {\n    pub fn bump(&mut self) {}\n}\n";
const RULE: &str = "records-and-objects";

fn check(path: &str, contents: &str) -> Vec<Offence> {
    RecordsAndObjectsRule::new().check_workspace(&[SourceFile::new(path, contents)])
}

fn object(fields: &str) -> String {
    format!("pub struct Term {{\n{fields}}}{OBJECT}")
}

// The rule's subject is the package, since an impl that gives a struct its
// behaviour may sit in another file; a single file says nothing on its own.
#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/term.rs", &object("    pub number: u64,\n"));

    // Act
    let offences = RecordsAndObjectsRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_builder_exposing_a_field_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/term_builder.rs",
        "pub struct TermBuilder {\n    pub number: Option<u64>,\n}\n\nimpl TermBuilder {\n    pub fn build(self) -> Term {\n        Term::new(self.number)\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("TermBuilder"));
}

#[test]
fn check_workspace_of_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/broken.rs", "pub struct Term {\n    pub number: u64,\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_getter_only_record_counts_its_accessors_and_says_what_to_do() {
    // Arrange & Act
    let offences = check(
        "src/report.rs",
        "pub struct Report {\n    total: u32,\n    crappy: u32,\n}\n\nimpl Report {\n    pub fn new(total: u32, crappy: u32) -> Self {\n        Self { total, crappy }\n    }\n\n    pub fn total(&self) -> u32 {\n        self.total\n    }\n\n    pub fn crappy(&self) -> u32 {\n        self.crappy\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].line(), 1);
    assert_eq!(offences[0].subject(), Some("Report"));
    assert_eq!(
        offences[0].description(),
        "`Report` is getter-only -- 2 getters, 1 constructor and no behaviour"
    );
    assert_eq!(
        offences[0].correction(),
        "delete the getters and constructor, make the fields pub and build it by literal"
    );
}

#[test]
fn check_workspace_of_a_getter_only_record_with_a_step_counts_it() {
    // Arrange & Act
    let offences = check(
        "src/export.rs",
        "pub struct Export {\n    pub size: u32,\n}\n\nimpl Export {\n    pub fn size(&self) -> u32 {\n        self.size\n    }\n\n    pub fn with_size(self, size: u32) -> Self {\n        Self { size, ..self }\n    }\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Export` is getter-only -- 1 getter, 1 `with_` step and no behaviour"
    );
    assert_eq!(
        offences[0].correction(),
        "delete the getter and step, make the fields pub and build it by literal"
    );
}

#[test]
fn check_workspace_of_a_record_with_private_fields_and_getters_reports_it_once_as_getter_only() {
    // Arrange & Act
    let offences = check(
        "src/export.rs",
        "pub struct Export {\n    data: Vec<u8>,\n}\n\nimpl Export {\n    pub fn data(&self) -> &[u8] {\n        &self.data\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Export` is getter-only -- 1 getter and no behaviour"
    );
    assert_eq!(
        offences[0].correction(),
        "delete the getter, make the fields pub and build it by literal"
    );
}

#[test]
fn check_workspace_of_a_record_with_private_fields_names_them_and_says_what_to_do() {
    // Arrange & Act
    let offences = check(
        "src/export.rs",
        "pub struct Export {\n    data: Vec<u8>,\n    pub size: u32,\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("Export"));
    assert_eq!(
        offences[0].description(),
        "`Export` is a record with a private field `data`; a record's fields are public"
    );
    assert_eq!(
        offences[0].correction(),
        "make it pub: a struct with no behaviour has no invariant to keep"
    );
}

#[test]
fn check_workspace_of_a_record_with_public_fields_and_a_value_trait_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/line.rs",
        "pub struct Line {\n    pub text: String,\n}\n\nimpl Display for Line {\n    fn fmt(&self, f: &mut Formatter) -> Result {\n        write!(f, \"{}\", self.text)\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_with_two_private_fields_names_both_in_the_plural() {
    // Arrange & Act
    let offences = check(
        "src/export.rs",
        "pub struct Export {\n    data: Vec<u8>,\n    size: u32,\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Export` is a record with private fields `data` and `size`; a record's fields are public"
    );
    assert_eq!(
        offences[0].correction(),
        "make them pub: a struct with no behaviour has no invariant to keep"
    );
}

#[test]
fn check_workspace_of_a_struct_in_a_cfg_test_module_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        "#[cfg(test)]\nmod tests {\n    struct Fake {\n        secret: u64,\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_struct_under_tests_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/support/fake.rs", &object("    pub number: u64,\n"));

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_struct_with_a_defaulting_constructor_reports_it_as_an_object() {
    // Arrange & Act
    let offences = check(
        "src/counter.rs",
        "pub struct Counter {\n    pub count: u32,\n}\n\nimpl Counter {\n    pub fn new() -> Self {\n        Self { count: 0 }\n    }\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Counter` is an object exposing `count`; an object's fields are private"
    );
}

#[test]
fn check_workspace_of_a_unit_struct_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/marker.rs",
        "pub struct Marker;\n\nimpl Marker {\n    pub fn run(&self) {}\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_an_enums_variant_fields_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/message.rs",
        "pub enum Message {\n    Vote { term: u64 },\n    Ack(u64),\n}\n\nimpl Message {\n    pub fn term(&self) -> u64 {\n        0\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_an_object_exposing_a_field_names_it_and_says_what_to_do() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        &object("    pub number: u64,\n    leader: u32,\n"),
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].line(), 1);
    assert_eq!(offences[0].subject(), Some("Term"));
    assert_eq!(
        offences[0].description(),
        "`Term` is an object exposing `number`; an object's fields are private"
    );
    assert_eq!(
        offences[0].correction(),
        "make it private: give a reader a getter and a builder a constructor"
    );
}

#[test]
fn check_workspace_of_an_object_in_an_inline_module_reports_it_at_its_line() {
    // Arrange & Act
    let offences = check(
        "src/outer.rs",
        "mod inner {\n    pub struct Term {\n        pub number: u64,\n    }\n\n    impl Term {\n        pub fn bump(&mut self) {}\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line(), 2);
}

#[test]
fn check_workspace_of_an_object_newtypes_public_field_reports_it_by_position() {
    // Arrange & Act
    let offences = check(
        "src/height.rs",
        "pub struct Height(pub u64);\n\nimpl Height {\n    pub fn next(&self) -> Self {\n        Self(self.0 + 1)\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Height` is an object exposing `0`; an object's fields are private"
    );
}

#[test]
fn check_workspace_of_an_object_shaped_by_an_impl_in_another_file_reports_its_field() {
    // Arrange
    let files = [
        SourceFile::new(
            "src/adapter.rs",
            "pub struct Adapter {\n    pub store: Store,\n}\n",
        ),
        SourceFile::new(
            "src/adapter_port.rs",
            "impl Port for Adapter {\n    fn send(&self) {}\n}\n",
        ),
    ];

    // Act
    let offences = RecordsAndObjectsRule::new().check_workspace(&files);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file(), "src/adapter.rs");
    assert_eq!(offences[0].subject(), Some("Adapter"));
}

#[test]
fn check_workspace_of_an_object_with_a_pub_crate_field_reports_it() {
    // Arrange & Act
    let offences = check("src/term.rs", &object("    pub(crate) number: u64,\n"));

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_workspace_of_an_object_with_a_pub_in_path_field_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        &object("    pub(in crate::raft) number: u64,\n"),
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_workspace_of_an_object_with_a_pub_super_field_reports_it() {
    // Arrange & Act
    let offences = check("src/term.rs", &object("    pub(super) number: u64,\n"));

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_workspace_of_an_object_with_many_fields_names_two_and_counts_the_rest() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        &object("    pub a: u8,\n    pub b: u8,\n    pub c: u8,\n    pub d: u8,\n    pub e: u8,\n"),
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Term` is an object exposing `a`, `b` and 3 more fields; an object's fields are private"
    );
    assert_eq!(
        offences[0].correction(),
        "make them private: give a reader a getter and a builder a constructor"
    );
}

#[test]
fn check_workspace_of_an_object_with_private_fields_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        &object("    number: u64,\n    pub(self) leader: u32,\n"),
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_an_object_with_three_fields_names_each() {
    // Arrange & Act
    let offences = check(
        "src/term.rs",
        &object("    pub a: u8,\n    pub b: u8,\n    pub c: u8,\n"),
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Term` is an object exposing `a`, `b` and `c`; an object's fields are private"
    );
}

#[test]
fn check_workspace_of_an_object_with_two_fields_names_both() {
    // Arrange & Act
    let offences = check("src/term.rs", &object("    pub a: u8,\n    pub b: u8,\n"));

    // Assert
    assert_eq!(
        offences[0].description(),
        "`Term` is an object exposing `a` and `b`; an object's fields are private"
    );
}

#[test]
fn explanation_shows_a_getter_only_record_and_the_record_it_should_be() {
    // Arrange & Act
    let explanation = RecordsAndObjectsRule::new().explanation();

    // Assert
    assert_eq!(explanation.name(), RULE);
    assert!(explanation.breaks().contains("pub fn number(&self) -> u64"));
    assert!(explanation.instead().contains("pub number: u64"));
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = RecordsAndObjectsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = RecordsAndObjectsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = RecordsAndObjectsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
